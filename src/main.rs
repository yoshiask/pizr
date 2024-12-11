#[path = "../introspections/org.bluez/adapter1.rs"] mod adapter1;

use std::collections::HashMap;
use std::error::Error;

use zbus::{zvariant::Value, proxy, Connection};

// Although we use `tokio` here, you can use any async runtime of choice.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let connection = Connection::session().await?;

    let proxy = adapter1::Adapter1Proxy::new(&connection).await?;
    let name = proxy.name().await?;
    println!("{}", name);

    Ok(())
}