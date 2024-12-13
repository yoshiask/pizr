#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;

use std::collections::HashMap;
use std::error::Error;

use zbus::{zvariant::Value, proxy, Connection, fdo::ObjectManagerProxy, ObjectServer};
use zbus::fdo::{ManagedObjects, ObjectManager};

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;
    let object_manager = ObjectManagerProxy::builder(&connection)
        .path("/org/bluez")?
	.destination("org.bluez")?
        .build()
        .await?;
    println!("Built object manager");

    let managed_objects = object_manager.get_managed_objects().await?;
    for object_path in managed_objects.keys() {
        println!("Managed object: {}", object_path);
    }

    Ok(())

    // let proxy = adapter1::Adapter1Proxy::new(&connection).await?;
    // let name = proxy.name().await?;
    // println!("{}", name);
    //
    // Ok(())
}
