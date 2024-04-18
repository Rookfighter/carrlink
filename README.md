# carrlink

`carrlink` is a rust library for interfacing with the Carrera control unit.
The library provides the following functionality

* connect to control unit via bluetooth and -serial connection-
* configure fuel level, brake and speed level of the cars
* manipulate position and lap tower
* read lap times

# Usage

For getting started with `carrlink` using bluetooth you simply need to select your bluetooth adapter, search for a neabry control unit and start right off communicating with.

For more examples, have a look at the `examples/` directory.

```rs
use std::{io, time::Duration};

use btleplug::{
    api::{Central as _, Manager as _},
    platform::{Adapter, Manager},
};
use carrlink::{BackendBLE, ControlUnit};
use tokio;

async fn find_adapter() -> btleplug::Result<Adapter> {
    let manager = Manager::new().await?;
    let adapter_list = manager.adapters().await?;
    match adapter_list.first() {
        Some(adapter) => Ok(adapter.to_owned()),
        None => Err(btleplug::Error::DeviceNotFound),
    }
}

async fn find_control_unit(adapter: Adapter) -> carrlink::Result<ControlUnit<BackendBLE>> {
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

    let version = control_unit.get_version().await.unwrap();
    println!("CU version: {}", version);

    println!("Disconnect from control unit");
    control_unit.disconnect().await.unwrap();

    Ok(())
}
```

# License

`carrlink` is licensed under the [MIT License](https://github.com/Rookfighter/carrlink/blob/main/LICENSE)

Carrera® and Carrera AppConnect® are registered trademarks of Carrera
Toys GmbH.

`carrlink` is not an official Carrera® product, and is not
affiliated with or endorsed by Carrera Toys GmbH.

`carrlink` took inspiration from the Python library [`carreralib`](https://github.com/tkem/carreralib).
