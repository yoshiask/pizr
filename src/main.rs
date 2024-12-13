#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;
#[path = "../introspections/org.bluez/bluez.rs"] mod bluez;

use std::error::Error;

use zbus::{Connection};
use crate::bluez::{BLUEZ_PATH_ROOT, BLUEZ_SERVICE};

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let hci0 = adapter1::Adapter1Proxy::builder(&connection)
        .destination(BLUEZ_SERVICE)?
        .interface(format!("{}.{}", BLUEZ_SERVICE, "Adapter1"))?
        .path(format!("{}/{}", BLUEZ_PATH_ROOT, "hci0"))?
        .build()
        .await?;

    let controller_name = hci0.name().await?;
    println!("Controller: {}", controller_name);

    let controller_powered = hci0.powered().await?;
    println!("Powered?: {}", controller_powered);

    // Power on BT controller
    hci0.set_powered(true).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    if hci0.powered().await? {
        println!("Successfully powered hci0");
    }
    else {
        println!("Failed to power on hci0");
    }

    Ok(())
}
