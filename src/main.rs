#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;
#[path = "../introspections/org.bluez/bluez.rs"] mod bluez;

use std::error::Error;

use zbus::{Connection};
use zbus::fdo::{IntrospectableProxy};
use crate::bluez::{BLUEZ_PATH_ROOT, BLUEZ_SERVICE};

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;

    let bluez_intro = IntrospectableProxy::builder(&connection)
	    .destination(BLUEZ_SERVICE)?
        .path(BLUEZ_PATH_ROOT)?
        .build()
        .await?;
    println!("Built bluez");

    let bluez_introspection = bluez_intro.introspect().await?;
    println!("Introspection:");
    println!("{}", bluez_introspection);
    println!();

    let hci0 = adapter1::Adapter1Proxy::builder(&connection)
        .destination(BLUEZ_SERVICE)?
        .interface(format!("{}.{}", BLUEZ_SERVICE, "Adapter1"))?
        .path(format!("{}/{}", BLUEZ_PATH_ROOT, "hci0"))?
        .build()
        .await?;

    let controller_name = hci0.name().await?;
    println!("Controller: {}", controller_name);

    Ok(())
}
