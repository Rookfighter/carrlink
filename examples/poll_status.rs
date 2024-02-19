use std::{io, time::Duration};

use btleplug::{
    api::{Central as _, Manager as _},
    platform::{Adapter, Manager},
};
use carrlink::ControlUnit;
use tokio;

async fn find_adapter() -> btleplug::Result<Adapter> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    match adapter_list.first() {
        Some(adapter) => Ok(adapter.to_owned()),
        None => Err(btleplug::Error::DeviceNotFound),
    }
}

async fn find_control_unit(adapter: Adapter) -> carrlink::Result<ControlUnit> {
    loop {
        match carrlink::discover_first_ble(&adapter, Duration::from_secs(5)).await? {
            Some(control_unit) => return Ok(control_unit),
            None => println!("No control unit found"),
        };
    }
}

#[tokio::main()]
async fn main() -> io::Result<()> {
    println!("Search adapter ...");

    let adapter = find_adapter().await.unwrap();

    println!(
        "Search control unit on adapter {} ...",
        adapter.adapter_info().await.unwrap()
    );

    let mut control_unit = find_control_unit(adapter).await.unwrap();

    println!("Connect to control unit");
    control_unit.connect().await.unwrap();

    println!("Looping");
    loop {
        println!("Fetch status");
        match control_unit.get_status().await {
            Ok(status) => println!("{:?}", status),
            Err(error) => println!("error: {}", error),
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // control_unit.disconnect().await.unwrap();

    Ok(())
}
