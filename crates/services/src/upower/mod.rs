pub mod dbus;

use anyhow::Result;
use dbus::UPowerProxy;
use zbus::fdo::PropertiesProxy;
use zbus::proxy::CacheProperties;

pub use dbus::BatteryState;

pub async fn create_upower_proxy() -> Result<PropertiesProxy<'static>> {
    let dbus = Box::pin(zbus::Connection::system()).await?;

    let device_proxy = UPowerProxy::new(&dbus).await?;

    let display_device = device_proxy.get_display_device().await?;

    let path = display_device.inner().path();

    let proxy = PropertiesProxy::builder(&dbus)
        .destination("org.freedesktop.UPower")
        .expect("failed to set proxy destination address")
        .path(path)
        .expect("failed to set proxy path")
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    Ok(proxy)
}
