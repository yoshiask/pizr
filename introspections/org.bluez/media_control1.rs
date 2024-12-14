//! # D-Bus interface proxy for: `org.bluez.MediaControl1`
//!
//! This code was generated by `zbus-xmlgen` `5.0.1` from D-Bus introspection data.
//! Source: `org.bluez.device1.xml`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the [Writing a client proxy] section of the zbus
//! documentation.
//!
//! This type implements the [D-Bus standard interfaces], (`org.freedesktop.DBus.*`) for which the
//! following zbus API can be used:
//!
//! * [`zbus::fdo::IntrospectableProxy`]
//! * [`zbus::fdo::PropertiesProxy`]
//!
//! Consequently `zbus-xmlgen` did not generate code for the above interfaces.
//!
//! [Writing a client proxy]: https://dbus2.github.io/zbus/client.html
//! [D-Bus standard interfaces]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces,
use zbus::proxy;
#[proxy(interface = "org.bluez.MediaControl1", assume_defaults = true)]
pub trait MediaControl1 {
    /// FastForward method
    fn fast_forward(&self) -> zbus::Result<()>;

    /// Next method
    fn next(&self) -> zbus::Result<()>;

    /// Pause method
    fn pause(&self) -> zbus::Result<()>;

    /// Play method
    fn play(&self) -> zbus::Result<()>;

    /// Previous method
    fn previous(&self) -> zbus::Result<()>;

    /// Rewind method
    fn rewind(&self) -> zbus::Result<()>;

    /// Stop method
    fn stop(&self) -> zbus::Result<()>;

    /// VolumeDown method
    fn volume_down(&self) -> zbus::Result<()>;

    /// VolumeUp method
    fn volume_up(&self) -> zbus::Result<()>;

    /// Connected property
    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;

    /// Player property
    #[zbus(property)]
    fn player(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}
