//! Module which implements a bluetooth low energy backend with routines for
//! connecting, disconnecting and sending requests.

use std::time::{Duration, Instant};

use crate::{ControlUnit, Error};
use btleplug::api::{
    Central as _, CentralEvent, Characteristic, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Peripheral};
use futures::stream::StreamExt;
use uuid::{uuid, Uuid};

const SERVICE_UUID: Uuid = uuid!("39df7777-b1b4-b90b-57f1-7144ae4e4a6a");
const NOTIFY_UUID: Uuid = uuid!("39df9999-b1b4-b90b-57f1-7144ae4e4a6a");
const OUTPUT_UUID: Uuid = uuid!("39df8888-b1b4-b90b-57f1-7144ae4e4a6a");

impl From<btleplug::Error> for crate::Error {
    fn from(value: btleplug::Error) -> Self {
        match value {
            btleplug::Error::DeviceNotFound => crate::Error::DeviceNotFound,
            btleplug::Error::PermissionDenied => crate::Error::PermissionDenied,
            btleplug::Error::NotConnected => crate::Error::NotConnected,
            btleplug::Error::Other(contained) => crate::Error::Other(contained),
            btleplug::Error::NotSupported(msg) => crate::Error::NotSupported(msg),
            btleplug::Error::TimedOut(_) => crate::Error::TimedOut,
            btleplug::Error::RuntimeError(msg) => crate::Error::RuntimeError(msg),
            _ => crate::Error::Other(Box::new(value)),
        }
    }
}

struct EndpointsBLE {
    notify_char: Characteristic,
    output_char: Characteristic,
}

pub struct BackendBLE {
    peripheral: Peripheral,
    endpoints: Option<EndpointsBLE>,
}

fn as_timeout_error<E>(_: E) -> Error {
    Error::TimedOut
}

impl BackendBLE {
    pub fn new(peripheral: Peripheral) -> BackendBLE {
        BackendBLE {
            peripheral,
            endpoints: None,
        }
    }
    /// Connects the backend with the configured peripheral.
    pub async fn connect(&mut self, timeout: Duration) -> crate::Result<()> {
        let ret = tokio::time::timeout(timeout.clone(), self.connect_internal()).await;
        Ok(ret.map_err(as_timeout_error)??)
    }

    pub async fn disconnect(&mut self, timeout: Duration) -> crate::Result<()> {
        let ret = tokio::time::timeout(timeout.clone(), self.disconnect_internal()).await;
        Ok(ret.map_err(as_timeout_error)??)
    }

    pub async fn request(&mut self, data: &[u8], timeout: Duration) -> crate::Result<Vec<u8>> {
        let ret = tokio::time::timeout(timeout.clone(), self.request_internal(data)).await;
        Ok(ret.map_err(as_timeout_error)??)
    }

    pub async fn is_connected(&self) -> crate::Result<bool> {
        Ok(self.peripheral.is_connected().await? && self.endpoints.is_some())
    }

    async fn connect_internal(&mut self) -> btleplug::Result<()> {
        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;
        }

        self.peripheral.discover_services().await?;

        let service = match self
            .peripheral
            .services()
            .iter()
            .find(|s| s.uuid == SERVICE_UUID)
        {
            Some(s) => Ok(s.clone()),
            None => Err(btleplug::Error::NoSuchCharacteristic),
        }?;

        let notify_char = match service
            .characteristics
            .iter()
            .find(|c| c.uuid == NOTIFY_UUID)
        {
            Some(c) => Ok(c.clone()),
            None => Err(btleplug::Error::NoSuchCharacteristic),
        }?;

        let output_char = match service
            .characteristics
            .iter()
            .find(|c| c.service_uuid == SERVICE_UUID && c.uuid == OUTPUT_UUID)
        {
            Some(c) => Ok(c.clone()),
            None => Err(btleplug::Error::NoSuchCharacteristic),
        }?;

        self.peripheral.subscribe(&notify_char).await?;

        self.endpoints = Some(EndpointsBLE {
            output_char,
            notify_char,
        });

        Ok(())
    }

    async fn disconnect_internal(&mut self) -> btleplug::Result<()> {
        match &self.endpoints {
            Some(endpoints) => self.peripheral.unsubscribe(&endpoints.notify_char).await?,
            None => (),
        }

        self.endpoints = None;

        if self.peripheral.is_connected().await? {
            self.peripheral.disconnect().await?;
        }

        Ok(())
    }

    async fn request_internal(&mut self, data: &[u8]) -> btleplug::Result<Vec<u8>> {
        match &self.endpoints {
            None => Err(btleplug::Error::NotConnected),
            Some(endpoints) => {
                self.peripheral
                    .write(&endpoints.output_char, data, WriteType::WithoutResponse)
                    .await?;
                let mut notify_stream = self.peripheral.notifications().await?.take(1);
                match notify_stream.next().await {
                    Some(in_data) => {
                        let mut result = in_data.value;
                        // BLE data is mostly tailored by a $ and they miss the command character
                        // bring this data buffer into a common format
                        if !result.is_empty() && *result.last().unwrap() == b'$' {
                            result.truncate(result.len() - 1);
                            result.splice(0..0, [*data.first().unwrap()]);
                        }

                        Ok(result)
                    }
                    None => Err(btleplug::Error::RuntimeError("no response".to_owned())),
                }
            }
        }
    }
}

async fn is_control_unit(peripheral: &Peripheral) -> btleplug::Result<bool> {
    match peripheral.properties().await? {
        Some(properties) => match properties.local_name {
            Some(name) => Ok(name == "Control_Unit"),
            None => Ok(false),
        },
        None => Ok(false),
    }
}

/// Searches for a control unit bluetooth device in the range of the given adapter and returns the first instance.
/// Returns the found control unit if any was available, otherwise none on timeout or an error when any error occurs.
pub async fn discover_first_ble(
    adapter: &Adapter,
    timeout: Duration,
) -> crate::Result<Option<ControlUnit>> {
    Ok(discover_first_ble_internal(&adapter, timeout).await?)
}

async fn discover_first_ble_internal(
    adapter: &Adapter,
    timeout: Duration,
) -> btleplug::Result<Option<ControlUnit>> {
    let start = Instant::now();
    adapter.start_scan(ScanFilter::default()).await?;
    let mut events = adapter.events().await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(peripheral_id) => {
                let peripheral = adapter.peripheral(&peripheral_id).await?;
                if is_control_unit(&peripheral).await? {
                    adapter.stop_scan().await?;
                    return Ok(Some(ControlUnit::new(BackendBLE::new(peripheral))));
                }
            }
            _ => (),
        }

        if start.elapsed() > timeout {
            break;
        }
    }

    adapter.stop_scan().await?;
    Ok(None)
}
