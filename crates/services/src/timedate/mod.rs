pub mod dbus;

use anyhow::Result;
use dbus::TimeDateProxy;
use std::sync::Arc;
use zbus::fdo::PropertiesProxy;
use zbus::proxy::CacheProperties;

pub async fn create_timedate_proxy() -> Result<Arc<PropertiesProxy<'static>>> {
    let dbus = Box::pin(zbus::Connection::system()).await?;

    let device_proxy = TimeDateProxy::new(&dbus).await?;

    let proxy = PropertiesProxy::builder(&dbus)
        .destination("org.freedesktop.timedate1")
        .expect("failed to set proxy destination address")
        .path(device_proxy.inner().path())
        .expect("failed to set proxy path")
        .cache_properties(CacheProperties::No)
        .build()
        .await?;

    Ok(Arc::new(proxy))
}
