#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;

use std::error::Error;

use zbus::{Connection};
use zbus::fdo::{IntrospectableProxy};

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
    println!();

    let hci0 = adapter1::Adapter1Proxy::builder(&connection)
        .destination("org.bluez")?
        .interface("org.bluez.Adapter1")?
        .path("/org/bluez/hci0")?
        .build()
        .await?;

    let controller_name = hci0.name().await?;
    println!("Controller: {}", controller_name);

    Ok(())
}
