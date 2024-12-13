#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;

use std::collections::HashMap;
use std::error::Error;

use zbus::{zvariant::Value, proxy, Connection, ObjectServer};
use zbus::fdo::{ManagedObjects, ObjectManager, IntrospectableProxy};

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let bluez_intro = IntrospectableProxy::builder(&connection)
        .path("/org/bluez")?
	    .destination("org.bluez")?
        .build()
        .await?;
    println!("Built bluez");

    let bluez_introspection = bluez_intro.introspect().await?;
    println!("Introspection:");
    println!("{}", bluez_introspection);

    // let managed_objects = bluez_intro.introspect().await?;
    // for object_path in managed_objects.keys() {
    //     println!("Managed object: {}", object_path);
    // }

    Ok(())

    // let proxy = adapter1::Adapter1Proxy::new(&connection).await?;
    // let name = proxy.name().await?;
    // println!("{}", name);
    //
    // Ok(())
}
