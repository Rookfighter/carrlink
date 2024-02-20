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

    loop {
        println!("Button (enter / countdown / esc / brake / speed / fuel / code / exit");
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;

        match &answer[..answer.len()-1] {
            "enter"=> control_unit.press_enter().await.unwrap(),
            "countdown"=> { control_unit.press_enter().await.unwrap(); control_unit.press_enter().await.unwrap()},
            "esc"=> control_unit.press_esc().await.unwrap(),
            "speed"=> control_unit.press_speed().await.unwrap(),
            "brake"=> control_unit.press_brake().await.unwrap(),
            "fuel"=> control_unit.press_fuel().await.unwrap(),
            "code"=> control_unit.press_code().await.unwrap(),
            "exit"=> break,
            _ => println!("Invalid command: {}", answer),
        }
    }

    Ok(())
}

