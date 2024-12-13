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

    let controller_powered = hci0.powered().await?;
    if !controller_powered {
        // Power on BT controller
        hci0.set_powered(true).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        if hci0.powered().await? {
            println!("Successfully powered hci0");
        }
        else {
            return Err(Box::from("Failed power on hci0"));
        }
    }

    let controller_discoverable = hci0.discoverable().await?;
    if !controller_discoverable {
        // Make device discoverable
        hci0.set_discoverable(true).await?;
        hci0.set_pairable(true).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        if hci0.discoverable().await? {
            let controller_name = hci0.name().await?;
            println!("pizr is discoverable as '{}'", controller_name);
        }
        else {
            return Err(Box::from("Failed make hci0 discoverable"));
        }
    }

    // let controller_scan = hci0.discovering().await?;
    // if !controller_scan {
    //     // Start scanning for devices
    //     hci0.start_discovery().await?;
    //     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    //
    //     if hci0.discovering().await? {
    //         println!("Searching for devices...");
    //     }
    //     else {
    //         return Err(Box::from("Failed to scan for devices"));
    //     }
    // }

    Ok(())
}
