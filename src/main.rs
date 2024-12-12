#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;

use std::collections::HashMap;
use std::error::Error;

use zbus::{zvariant::Value, proxy, Connection, fdo::ObjectManagerProxy};

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::system().await?;
    let object_manager = ObjectManagerProxy::get_managed_objects();


    let proxy = adapter1::Adapter1Proxy::new(&connection).await?;
    let name = proxy.name().await?;
    println!("{}", name);

    Ok(())
}