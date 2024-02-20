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
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3
    {
        println!("usage: set_speed <player> <level>");
        return Ok(());
    }



    println!("Search adapter ...");
    let adapter = find_adapter().await.unwrap();

    println!(
        "Search control unit on adapter {} ...",
        adapter.adapter_info().await.unwrap()
    );

    let mut control_unit = find_control_unit(adapter).await.unwrap();

    println!("Connect to control unit");
    control_unit.connect().await.unwrap();

    let player = args[1].parse::<usize>().unwrap();
    let level =  args[2].parse::<usize>().unwrap();
    println!("Set speed of player #{} to {}", player, level);
    control_unit.set_speed_level(player, level).await;

    println!("Disconnect from control unit");
    control_unit.disconnect().await.unwrap();

    Ok(())
}
