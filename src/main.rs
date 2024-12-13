#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;
#[path = "../introspections/org.bluez/bluez.rs"] mod bluez;

use std::error::Error;

use zbus::{Connection};
use zbus::export::futures_util::{pin_mut, select, StreamExt};
use zbus::fdo::{ObjectManager, ObjectManagerProxy};
use crate::bluez::{BLUEZ_PATH_ROOT, BLUEZ_SERVICE};

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let hci0_path = format!("{}/{}", BLUEZ_PATH_ROOT, "hci0");

    let hci0 = adapter1::Adapter1Proxy::builder(&connection)
        .destination(BLUEZ_SERVICE)?
        .interface(format!("{}.{}", BLUEZ_SERVICE, "Adapter1"))?
        .path(hci0_path)?
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

    // Listen for new devices
    let objman = ObjectManagerProxy::builder(&connection)
        .interface("org.freedesktop.DBus.ObjectManager")?
        .build()
        .await?;

    let mut added_interfaces_stream = objman.receive_interfaces_added().await?;
    let added_devices_stream = added_interfaces_stream.filter_map(move |signal| async move {
        let args = signal.args().ok()?;
        return if args.object_path.starts_with(concat!(hci0_path, "/")) {
            let device = args.interfaces_and_properties.get("org.bluez.Device1")?;
            let address: String = device.get("Address")?.try_into().ok()?;
            Some(address)
        } else {
            None
        }
    });

    pin_mut!(added_devices_stream);
    while let Some(added_device) = added_devices_stream.next().await {
        println!("{:?}", added_device);
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

pub async fn stream_device_events<'b>(&'b self) -> zbus::Result<impl futures_util::Stream<Item = DiscoveredDeviceEvent> + 'b> {
    let added_objects = self.object_manager.receive_interfaces_added().await?;
    let removed_objects = self.object_manager.receive_interfaces_removed().await?;

    let added_devices = added_objects.filter_map(move |signal: InterfacesAdded| async move {
        let args = signal.args().ok()?;
        if self.is_device_path(&args.object_path) {
            let device = args.interfaces_and_properties.get("org.bluez.Device1")?;
            let address: String = device.get("Address")?.try_into().ok()?;
            let services = Vec::<_>::try_from(device.get("UUIDs")?.try_to_owned().unwrap()).ok()?.into_iter().collect();
            let connected: bool = device.get("Connected")?.try_into().ok()?;
            let rssi: Option<i16> = device.get("RSSI").and_then(|v| v.try_into().ok());
            Some(DiscoveredDeviceEvent::DeviceAdded(DiscoveredDevice {
                path: args.object_path.into(),
                address,
                services,
                connected,
                rssi
            }))
        } else { None }
    });

    let removed_devices = removed_objects.filter_map(move |signal| async move {
        let args = signal.args().ok()?;
        // if this is a device, and one of the removed interfaces was Device1
        if self.is_device_path(&args.object_path) && args.interfaces.contains(&"org.bluez.Device1") {
            Some(DiscoveredDeviceEvent::DeviceRemoved(args.object_path.into()))
        } else { None }
    });

    Ok(select(added_devices, removed_devices))
}
