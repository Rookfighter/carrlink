//! Module which implements a bluetooth low energy backend with routines for
//! connecting, disconnecting and sending requests.

use crate::ControlUnit;
use btleplug::api::{
    Central as _, CentralEvent, Characteristic, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Peripheral};
use futures::stream::StreamExt;
use uuid::{uuid, Uuid};

// const SERVICE_UUID: Uuid = uuid!("39df7777-b1b4-b90b-57f1-7144ae4e4a6a");
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
            btleplug::Error::TimedOut(duration) => crate::Error::TimedOut(duration),
            btleplug::Error::RuntimeError(msg) => crate::Error::RuntimeError(msg),
            _ => crate::Error::Other(Box::new(value)),
        }
    }
}

pub struct BackendBLE {
    peripheral: Peripheral,
    is_subscribed: bool,
    notify_char: Option<Characteristic>,
    output_char: Option<Characteristic>,
}

impl BackendBLE {
    pub fn new(peripheral: Peripheral) -> BackendBLE {
        BackendBLE {
            peripheral,
            is_subscribed: false,
            notify_char: None,
            output_char: None,
        }
    }
    /// Connects the backend with the configured peripheral.
    pub async fn connect(&mut self) -> crate::Result<()> {
        Ok(self.connect_internal().await?)
    }

    pub async fn disconnect(&mut self) -> crate::Result<()> {
        Ok(self.disconnect_internal().await?)
    }

    pub async fn request(&mut self, data: &[u8]) -> crate::Result<Vec<u8>> {
        Ok(self.request_internal(data).await?)
    }

    pub async fn is_connected(&self) -> crate::Result<bool> {
        Ok(self.peripheral.is_connected().await? && self.is_subscribed)
    }

    async fn connect_internal(&mut self) -> btleplug::Result<()> {
        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;

            self.peripheral.discover_services().await?;
        }

        let chars = self.peripheral.characteristics();

        self.notify_char = Some(
            match chars.iter().find(|c| c.uuid == NOTIFY_UUID) {
                Some(c) => Ok(c),
                None => Err(btleplug::Error::NoSuchCharacteristic),
            }?
            .clone(),
        );

        self.output_char = Some(
            match chars.iter().find(|c| c.uuid == OUTPUT_UUID) {
                Some(c) => Ok(c),
                None => Err(btleplug::Error::NoSuchCharacteristic),
            }?
            .clone(),
        );

        if !self.is_subscribed {
            self.peripheral
                .subscribe(&self.notify_char.as_ref().unwrap())
                .await?;
            self.is_subscribed = true;
        }

        Ok(())
    }

    async fn disconnect_internal(&mut self) -> btleplug::Result<()> {
        if self.is_subscribed {
            match self.notify_char.as_ref() {
                Some(value) => self.peripheral.unsubscribe(&value).await?,
                None => (),
            }
            self.is_subscribed = false;
        }

        if self.peripheral.is_connected().await? {
            self.peripheral.disconnect().await?;
        }

        Ok(())
    }

    async fn request_internal(&mut self, data: &[u8]) -> btleplug::Result<Vec<u8>> {
        let char = match self.output_char.as_mut() {
            Some(char) => Ok(char),
            None => Err(btleplug::Error::NotConnected),
        }?;

        self.peripheral
            .write(&char, data, WriteType::WithResponse)
            .await?;

        let mut notify_stream = self.peripheral.notifications().await?.take(1);

        match notify_stream.next().await {
            Some(in_data) => Ok(in_data.value),
            None => Err(btleplug::Error::RuntimeError("no response".to_owned())),
        }
    }
}

async fn is_control_unit(peripheral: &Peripheral) -> btleplug::Result<bool> {
    match peripheral.properties().await? {
        Some(properties) => match properties.local_name {
            Some(name) => Ok(name == "Control Unit"),
            None => Ok(false),
        },
        None => Ok(false),
    }
}

pub async fn discover_first_ble(adapter: &Adapter) -> crate::Result<Option<ControlUnit>> {
    Ok(discover_first_ble_internal(&adapter).await?)
}

/// Searches for a control unit bluetooth device in the range of the given adapter and returns the first instance.
async fn discover_first_ble_internal(adapter: &Adapter) -> btleplug::Result<Option<ControlUnit>> {
    adapter.start_scan(ScanFilter::default()).await?;
    let mut events = adapter.events().await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) => {
                let peripheral = adapter.add_peripheral(&id).await?;
                if is_control_unit(&peripheral).await? {
                    adapter.stop_scan().await?;
                    return Ok(Some(ControlUnit::new(BackendBLE::new(peripheral))));
                }
            }
            _ => continue,
        }
    }

    adapter.stop_scan().await?;
    Ok(None)
}
