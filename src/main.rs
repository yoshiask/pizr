#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;
#[path = "../introspections/org.bluez/device1.rs"] mod device1;
#[path = "../introspections/org.bluez/media_control1.rs"] mod mediacontrol1;
#[path = "../introspections/org.bluez/client1.rs"] mod obexclient1;
#[path = "../introspections/org.bluez/phonebook_access1.rs"] mod phonebookaccess1;
#[path = "../introspections/org.bluez/transfer1.rs"] mod transfer1;
#[path = "../introspections/org.bluez/bluez.rs"] mod bluez;

#[path = "usercontact.rs"] mod usercontact;

use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::str::FromStr;
use regex::Regex;
use zbus::names::InterfaceName;
use zbus::{Connection};
use zbus::export::futures_util::{pin_mut, StreamExt};
use zbus::fdo::{ObjectManagerProxy, PropertiesProxy};
use zbus::zvariant::{ObjectPath};
use crate::bluez::{BLUEZ_OBEX_PATH_ROOT, BLUEZ_OBEX_SERVICE, BLUEZ_PATH_ROOT, BLUEZ_SERVICE};

fn input<T: FromStr>() -> Result<T, <T as FromStr>::Err> {
    let mut input: String = String::with_capacity(64); 
    
    std::io::stdin()
    .read_line(&mut input)
    .expect("Input could not be read");
    
    input.parse()
}

/// Waits for a matching interface to be added to the DBus object with the specified path.
async fn wait_for_interface<'a, O, I>(connection: &Connection, obj_path: O, interface_name: I) -> Result<(), Box<dyn Error>>
    where
        O: TryInto<ObjectPath<'a>>,
        O::Error: Into<zbus::Error>,
        I: TryInto<InterfaceName<'a>>,
        I::Error: Into<zbus::Error>,
{
    let interface_name = &interface_name.try_into().map_err(Into::into)?;
    let obj_path = &obj_path.try_into().map_err(Into::into)?;
    
    let device = PropertiesProxy::builder(connection)
        .destination(BLUEZ_SERVICE)?
        .interface("org.freedesktop.DBus.Properties")?
        .path(obj_path)?
        .build()
        .await?;

    while device.get_all(interface_name.clone()).await.is_err() {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let system_bus = Connection::system().await?;

    let hci0_path = format!("{BLUEZ_PATH_ROOT}/hci0");

    let hci0 = adapter1::Adapter1Proxy::builder(&system_bus)
        .destination(BLUEZ_SERVICE)?
        .interface(format!("{BLUEZ_SERVICE}.Adapter1"))?
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
            println!("pizr is discoverable as '{controller_name}'");
        }
        else {
            return Err(Box::from("Failed make hci0 discoverable"));
        }
    }

    // Get an ObjectManager to get the first paired device
    let objman = ObjectManagerProxy::builder(&system_bus)
        .destination(BLUEZ_SERVICE)?
        .interface("org.freedesktop.DBus.ObjectManager")?
        .path("/")?
        .build()
        .await?;

    // Regex to match only immediate children of hci0, which should be devices
    let hci0_path_pattern = format!("^{hci0_path}/([^/]+)$");
    let hci0_path_re = Regex::new(hci0_path_pattern.as_str()).unwrap();

    // Check for existing paired devices
    let mut device_path: Option<ObjectPath> = None;
    let bluez_objects = objman.get_managed_objects().await?;
    for bluez_object in bluez_objects {
        let mut bluez_obj_path = bluez_object.0;
        if !hci0_path_re.is_match(bluez_obj_path.as_str()) {
            continue;
        }

        bluez_obj_path = bluez_obj_path.to_owned();
        device_path = Some(bluez_obj_path.into());
    }

    if !device_path.is_some() {
        // Listen for new devices
        println!("No devices found, waiting to pair...");

        let added_interfaces_stream = objman.receive_interfaces_added().await?;
        let added_devices_stream = added_interfaces_stream.filter_map(move |signal| {
        let hci0_path_re_onadd = hci0_path_re.clone();
        async move {
            let args = signal.args().ok()?;
            return if hci0_path_re_onadd.is_match(args.object_path.as_str()) {
                Some(args.object_path.into_owned())
            } else {
                None
            }
        }
        });

        pin_mut!(added_devices_stream);
        while let Some(added_device_path) = added_devices_stream.next().await {
            println!("Paired with {:?}", added_device_path);
            device_path = Some(added_device_path);
            break;
        }
    }

    println!("Found paired device: {:?}", device_path);

    // Get device instance
    let device_path = device_path.unwrap();
    let device = device1::Device1Proxy::builder(&system_bus)
        .destination(BLUEZ_SERVICE)?
        .interface(format!("{BLUEZ_SERVICE}.Device1"))?
        .path(device_path.clone())? // Clone because device proxy takes ownership
        .build()
        .await?;

    let device_name = device.alias().await?;
    println!("Device '{device_name}' is available");

    let device_connected = device.connected().await?;
    if !device_connected {
        device.connect().await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let device_connected = device.connected().await
            .unwrap_or(false);

        if !device_connected {
            return Err(Box::from(format!("Failed to connect to '{device_name}'")));
        }
    }

    println!("Connected to '{device_name}'");

    println!("Waiting for MediaControl1 interface...");
    wait_for_interface(&system_bus, &device_path, format!("{BLUEZ_SERVICE}.MediaControl1")).await?;
    println!("Got interface!");

    let media_control = mediacontrol1::MediaControl1Proxy::builder(&system_bus)
        .destination(BLUEZ_SERVICE)?
        .interface(format!("{BLUEZ_SERVICE}.MediaControl1"))?
        .path(device_path)?
        .build()
        .await?;

    // Wait for media controller to connect
    while !media_control.connected().await.unwrap_or(false) {
        println!("Waiting for media controller to connect...");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("Player is connected");

    // ** OBEX-based connections ** //
    let session_bus = Connection::session().await?;

    let obex_client = obexclient1::Client1Proxy::builder(&session_bus)
        .destination(BLUEZ_OBEX_SERVICE)?
        .interface(format!("{BLUEZ_OBEX_SERVICE}.Client1"))?
        .path(BLUEZ_OBEX_PATH_ROOT)?
        .build()
        .await?;

    let obex_pbap_remote_address = device.address().await?;
    let obex_pbap_target = zbus::zvariant::Value::from("pbap");
    let obex_pbap_args = HashMap::from([
        ("Target", &obex_pbap_target),
    ]);
    let pbap_path = obex_client.create_session(&obex_pbap_remote_address, obex_pbap_args).await?;

    // Get phonebook object
    let pb_access = phonebookaccess1::PhonebookAccess1Proxy::builder(&session_bus)
        .destination(BLUEZ_OBEX_SERVICE)?
        .interface(format!("{BLUEZ_OBEX_SERVICE}.PhonebookAccess1"))?
        .path(pbap_path)?
        .build()
        .await?;

    // See options in https://github.com/RadiusNetworks/bluez/blob/master/doc/obex-api.txt#L331
    pb_access.select("int", "pb").await?;

    print!("Name? ");
    io::stdout().flush()?;
    let query = input::<String>()?;

    let pb_search_results = pb_access.search("name", &query, HashMap::new()).await?;
    let selected_pb_entry = pb_search_results.first()
        .ok_or("No contacts found")?;
    println!("Found contact: {} - {}", selected_pb_entry.0, selected_pb_entry.1);

    // Target must be an absolute path
    let select_contact_vcard = pb_access.pull(&selected_pb_entry.0, "/home/yoshiask/Downloads/test.vcf", HashMap::new()).await?;
    
    let vcard_transfer = transfer1::Transfer1Proxy::builder(&session_bus)
        .destination(BLUEZ_OBEX_SERVICE)?
        .interface(format!("{BLUEZ_OBEX_SERVICE}.Transfer1"))?
        .path(select_contact_vcard.0.clone())?
        .build()
        .await?;

    loop {
        let transfer_status = vcard_transfer.status().await;
        
        println!("{}", transfer_status.expect("Transfer error"));

        if vcard_transfer.status().await? == "complete" {
            println!("Saved to {}", vcard_transfer.filename().await?);
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(250)).await;
    }

    // Read and parse vCard file
    let vcard = usercontact::parse_vcard_from_file(vcard_transfer.filename().await?.as_str())
        .ok_or("Failed to parse vCard")?;

    println!("    {}", vcard.formatted_name.unwrap());

    if let Some(preferred_tel) = vcard.perferred_tel {
        println!("    Preferred phone number: {}", preferred_tel);
    }
    
    if let Some(preferred_email) = vcard.perferred_email {
        println!("    Preferred email: {}", preferred_email);
    }

    // Get all contacts
    // let pb_favorites = pb_access.list(HashMap::new()).await?;
    // pb_favorites.iter().for_each(|(name, number)| {
    //     println!("{name} - {number}");
    // });

    let _: String = input::<String>()?;

    // Clean up OBEX session
    obex_client.remove_session(pb_access.inner().path()).await?;

    Ok(())
}
