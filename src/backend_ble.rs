//! Module which implements a bluetooth low energy backend with routines for
//! connecting, disconnecting and sending requests.

use btleplug::api::{
    Central as _, CentralEvent, Characteristic, Peripheral as _, ScanFilter, WriteType,
};
use btleplug::platform::{Adapter, Peripheral};
use futures::stream::StreamExt;
use uuid::{uuid, Uuid};

use super::Error;

// const SERVICE_UUID: Uuid = uuid!("39df7777-b1b4-b90b-57f1-7144ae4e4a6a");
const NOTIFY_UUID: Uuid = uuid!("39df9999-b1b4-b90b-57f1-7144ae4e4a6a");
const OUTPUT_UUID: Uuid = uuid!("39df8888-b1b4-b90b-57f1-7144ae4e4a6a");

fn convert_btleplug_error(error: btleplug::Error) -> Error {
    match error {
        btleplug::Error::DeviceNotFound => Error::DeviceNotFound,
        btleplug::Error::PermissionDenied => Error::PermissionDenied,
        btleplug::Error::NotConnected => Error::NotConnected,
        btleplug::Error::UnexpectedCallback => {
            Error::RuntimeError("btleplug::UnexpectedCallback".to_owned())
        }
        btleplug::Error::UnexpectedCharacteristic => {
            Error::RuntimeError("btleplug::UnexpectedCharacteristic".to_owned())
        }
        btleplug::Error::NoSuchCharacteristic => {
            Error::RuntimeError("btleplug::NoSuchCharacteristic".to_owned())
        }
        btleplug::Error::NotSupported(msg) => Error::NotSupported(msg),
        btleplug::Error::TimedOut(duration) => Error::TimedOut(duration),
        btleplug::Error::Uuid(_) => Error::RuntimeError("btleplug::UUID".to_owned()),
        btleplug::Error::InvalidBDAddr(_) => {
            Error::RuntimeError("btleplug::InvalidBDAddr".to_owned())
        }
        btleplug::Error::RuntimeError(msg) => Error::RuntimeError(msg),
        btleplug::Error::Other(_) => Error::Other,
    }
}

pub struct BackendBLE {
    peripheral: Peripheral,
    is_subscribed: bool,
    notify_char: Characteristic,
    output_char: Characteristic,
}

impl BackendBLE {
    /// Connects the backend with the configured peripheral.
    pub async fn connect(&mut self) -> Result<(), Error> {
        self.connect_internal()
            .await
            .map_err(convert_btleplug_error)
    }

    pub async fn disconnect(&mut self) -> Result<(), Error> {
        self.disconnect_internal()
            .await
            .map_err(convert_btleplug_error)
    }

    pub async fn request(&mut self, data: &[u8]) -> Result<Vec<u8>, Error> {
        self.request_internal(data)
            .await
            .map_err(convert_btleplug_error)
    }

    pub async fn is_connected(&self) -> Result<bool, Error> {
        self.peripheral
            .is_connected()
            .await
            .map(|is_connected| is_connected && self.is_subscribed)
            .map_err(convert_btleplug_error)
    }

    async fn connect_internal(&mut self) -> Result<(), btleplug::Error> {
        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;

            self.peripheral.discover_services().await?;
        }

        let chars = self.peripheral.characteristics();

        self.notify_char = match chars.iter().find(|c| c.uuid == NOTIFY_UUID) {
            Some(c) => Ok(c),
            None => Err(btleplug::Error::NoSuchCharacteristic),
        }?
        .clone();

        self.output_char = match chars.iter().find(|c| c.uuid == OUTPUT_UUID) {
            Some(c) => Ok(c),
            None => Err(btleplug::Error::NoSuchCharacteristic),
        }?
        .clone();

        if !self.is_subscribed {
            self.peripheral.subscribe(&self.notify_char).await?;
            self.is_subscribed = true;
        }

        Ok(())
    }

    async fn disconnect_internal(&mut self) -> Result<(), btleplug::Error> {
        if self.is_subscribed {
            self.peripheral.unsubscribe(&self.notify_char).await?;
            self.is_subscribed = false;
        }

        if self.peripheral.is_connected().await? {
            self.peripheral.disconnect().await?;
        }

        Ok(())
    }

    async fn request_internal(&mut self, data: &[u8]) -> Result<Vec<u8>, btleplug::Error> {
        self.peripheral
            .write(&self.output_char, data, WriteType::WithResponse)
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

/// Searches for a control unit bluetooth device in the range of the given adapter and returns the first instance.
pub async fn discover_first_ble(adapter: &Adapter) -> btleplug::Result<Option<Peripheral>> {
    adapter.start_scan(ScanFilter::default()).await?;
    let mut events = adapter.events().await?;

    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) => {
                let peripheral = adapter.add_peripheral(&id).await?;
                if is_control_unit(&peripheral).await? {
                    adapter.stop_scan().await?;
                    return Ok(Some(peripheral));
                }
            }
            _ => continue,
        }
    }

    adapter.stop_scan().await?;
    Ok(None)
}
