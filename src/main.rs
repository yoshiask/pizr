#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;
#[path = "../introspections/org.bluez/device1.rs"] mod device1;
#[path = "../introspections/org.bluez/bluez.rs"] mod bluez;

use std::error::Error;
use zbus::{Connection};
use zbus::export::futures_util::{pin_mut, StreamExt};
use zbus::fdo::{ObjectManagerProxy};
use zbus::zvariant::{ObjectPath};
use crate::bluez::{BLUEZ_PATH_ROOT, BLUEZ_SERVICE};

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let hci0_path = format!("{}/{}", BLUEZ_PATH_ROOT, "hci0");

    let hci0 = adapter1::Adapter1Proxy::builder(&connection)
        .destination(BLUEZ_SERVICE)?
        .interface(format!("{}.{}", BLUEZ_SERVICE, "Adapter1"))?
        .path(hci0_path.as_str())?
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

    // Get an ObjectManager to get the first paired device
    let objman = ObjectManagerProxy::builder(&connection)
        .destination(BLUEZ_SERVICE)?
        .interface("org.freedesktop.DBus.ObjectManager")?
        .path("/")?
        .build()
        .await?;

    // Check for existing paired devices
    let mut device_path: Option<ObjectPath> = None;
    let hci0_path_prefix = format!("{hci0_path}/");
    let bluez_objects = objman.get_managed_objects().await?;
    for bluez_object in bluez_objects {
        let mut bluez_obj_path = bluez_object.0;
        if !bluez_obj_path.starts_with(hci0_path_prefix.as_str()) {
            continue;
        }

        bluez_obj_path = bluez_obj_path.to_owned();
        device_path = Some(bluez_obj_path.into());
    }

    if !device_path.is_some() {
        // Listen for new devices
        println!("No devices found, waiting to pair...");

        let hci0_path_prefix_str = hci0_path_prefix.as_str();
        let added_interfaces_stream = objman.receive_interfaces_added().await?;
        let added_devices_stream = added_interfaces_stream.filter_map(move |signal| async move {
            let args = signal.args().ok()?;
            return if args.object_path.starts_with(hci0_path_prefix_str) {
                Some(args.object_path.into_owned())
            } else {
                None
            }
        });

        pin_mut!(added_devices_stream);
        while let Some(added_device_path) = added_devices_stream.next().await {
            println!("Paired with {:?}", added_device_path);
            device_path = Some(added_device_path);
            break;
        }
    }

    // Get device instance
    let device_path_str = device_path.unwrap();
    let device = device1::Device1Proxy::builder(&connection)
        .destination(BLUEZ_SERVICE)?
        .interface(format!("{}.{}", BLUEZ_SERVICE, "Device1"))?
        .path(device_path_str.as_str())?
        .build()
        .await?;

    let device_name = device.name().await?;
    println!("Device '{device_name}' is available");

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
