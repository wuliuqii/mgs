use std::collections::HashMap;
use std::ops::Deref;

use anyhow::Result;
use futures_util::StreamExt;
use itertools::Itertools;
use zbus::zvariant::OwnedObjectPath;

use super::active_connection::ActiveConnection;
use super::dbus::access_point::AccessPointProxy;
use super::dbus::active_connection::ActiveConnectionProxy;
use super::dbus::device::DeviceProxy;
use super::dbus::device::wired::WiredDeviceProxy;
use super::dbus::device::wireless::WirelessDeviceProxy;
use super::dbus::nm::NetworkManagerProxy;
use super::dbus::statistics::StatisticsProxy;
use super::device::{DeviceState, DeviceType};
use super::{AccessPoint, ActiveConnectionInfo, NetworkStatistics};

#[derive(Debug)]
pub struct NetworkManager<'a>(NetworkManagerProxy<'a>);

impl<'a> Deref for NetworkManager<'a> {
    type Target = NetworkManagerProxy<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> NetworkManager<'a> {
    pub async fn new(connection: &'a zbus::Connection) -> zbus::Result<NetworkManager<'a>> {
        NetworkManagerProxy::new(connection).await.map(Self)
    }

    pub async fn active_connections(&self) -> Result<Vec<ActiveConnectionInfo>> {
        let active_connections = self.0.active_connections().await?;
        let mut ac_proxies: Vec<ActiveConnection<'a>> =
            Vec::with_capacity(active_connections.len());
        for active_connection in active_connections {
            let active_connection = ActiveConnectionProxy::builder(self.inner().connection())
                .path(active_connection)?
                .build()
                .await?;
            ac_proxies.push(active_connection.into());
        }

        let mut info = Vec::<ActiveConnectionInfo>::with_capacity(ac_proxies.len());
        for connection in ac_proxies {
            for device in connection.devices().await.unwrap_or_default() {
                if connection.vpn().await.unwrap_or_default() {
                    info.push(ActiveConnectionInfo::Vpn {
                        name: connection.id().await?,
                        object_path: connection.inner().path().to_owned().into(),
                    });
                    continue;
                }

                let device_proxy = DeviceProxy::builder(self.inner().connection())
                    .path(device)?
                    .build()
                    .await?;

                match device_proxy.device_type().await.map(DeviceType::from).ok() {
                    Some(DeviceType::Ethernet) => {
                        let wired_device = WiredDeviceProxy::builder(self.inner().connection())
                            .path(device_proxy.inner().path())?
                            .build()
                            .await?;

                        info.push(ActiveConnectionInfo::Wired {
                            name: connection.id().await?,
                            speed: wired_device.speed().await?,
                        });
                    }
                    Some(DeviceType::Wifi) => {
                        let wireless_device =
                            WirelessDeviceProxy::builder(self.inner().connection())
                                .path(device_proxy.inner().path())?
                                .build()
                                .await?;

                        if let Ok(access_point) = wireless_device.active_access_point().await {
                            let access_point = AccessPointProxy::builder(self.inner().connection())
                                .path(access_point)?
                                .build()
                                .await?;

                            info.push(ActiveConnectionInfo::WiFi {
                                id: connection.id().await?,
                                name: String::from_utf8_lossy(&access_point.ssid().await?)
                                    .into_owned(),
                                strength: access_point.strength().await.unwrap_or_default(),
                                device: device_proxy.inner().path().to_string(),
                            });
                        }
                    }
                    Some(DeviceType::WireGuard) => {
                        info.push(ActiveConnectionInfo::Vpn {
                            name: connection.id().await?,
                            object_path: connection.inner().path().to_owned().into(),
                        });
                    }
                    _ => {}
                }
            }
        }

        info.sort_by(|a, b| {
            let helper = |conn: &ActiveConnectionInfo| match conn {
                ActiveConnectionInfo::Vpn { name, .. } => format!("0{name}"),
                ActiveConnectionInfo::Wired { name, .. } => format!("1{name}"),
                ActiveConnectionInfo::WiFi { name, .. } => format!("2{name}"),
            };
            helper(a).cmp(&helper(b))
        });

        Ok(info)
    }

    pub async fn wireless_devices(&self) -> Result<Vec<OwnedObjectPath>> {
        let devices = self.devices().await?;
        let mut wireless_devices = Vec::new();
        for device in devices {
            let device_proxy = DeviceProxy::builder(self.inner().connection())
                .path(&device)?
                .build()
                .await?;

            if matches!(
                device_proxy.device_type().await.map(DeviceType::from),
                Ok(DeviceType::Wifi)
            ) {
                wireless_devices.push(device);
            }
        }

        Ok(wireless_devices)
    }

    pub async fn network_statistics(&self) -> Result<Vec<NetworkStatistics>> {
        let devices = self.devices().await?;
        let mut network_statistics = Vec::new();
        for device in devices {
            let device_proxy = DeviceProxy::builder(self.inner().connection())
                .path(&device)?
                .build()
                .await?;

            if matches!(
                device_proxy.device_type().await.map(DeviceType::from),
                Ok(DeviceType::Wifi)
            ) {
                let staticstics_proxy = StatisticsProxy::builder(self.inner().connection())
                    .path(&device)?
                    .build()
                    .await?;
                let tx = staticstics_proxy.tx_bytes().await?;
                let rx = staticstics_proxy.rx_bytes().await?;
                let timestamp = chrono::Utc::now().timestamp();
                network_statistics.push(NetworkStatistics {
                    prev_rx: rx,
                    prev_tx: tx,
                    prev_rx_time: timestamp,
                    prev_tx_time: timestamp,
                    tx,
                    rx,
                    rx_time: timestamp,
                    tx_time: timestamp,
                    device: device_proxy.inner().path().to_string(),
                });
            }
        }

        Ok(network_statistics)
    }

    pub async fn wireless_access_points(&self) -> Result<Vec<AccessPoint>> {
        let wireless_devices = self.wireless_devices().await?;
        let wireless_ap_futures: Vec<_> = wireless_devices
            .iter()
            .map(|path| async move {
                let device_proxy = DeviceProxy::builder(self.inner().connection())
                    .path(path)?
                    .build()
                    .await?;
                let wireless_device = WirelessDeviceProxy::builder(self.inner().connection())
                    .path(path)?
                    .build()
                    .await?;
                wireless_device.request_scan(HashMap::new()).await?;
                let mut scan_changed = wireless_device.receive_last_scan_changed().await;
                if let Some(t) = scan_changed.next().await {
                    if let Ok(-1) = t.get().await {
                        return Ok(Default::default());
                    }
                }
                let access_points = wireless_device.get_access_points().await?;
                let state = device_proxy
                    .cached_state()
                    .unwrap_or_default()
                    .map(DeviceState::from)
                    .unwrap_or_else(|| DeviceState::Unknown);

                let mut aps = HashMap::<String, AccessPoint>::new();
                for ap in access_points {
                    let ap_proxy = AccessPointProxy::builder(self.inner().connection())
                        .path(ap)?
                        .build()
                        .await?;

                    let ssid = String::from_utf8_lossy(&ap_proxy.ssid().await?).into_owned();
                    let public = ap_proxy.flags().await.unwrap_or_default() == 0;
                    let strength = ap_proxy.strength().await?;
                    if let Some(access_point) = aps.get(&ssid) {
                        if access_point.strength > strength {
                            continue;
                        }
                    }

                    aps.insert(
                        ssid.clone(),
                        AccessPoint {
                            ssid,
                            strength,
                            state,
                            public,
                            working: false,
                            path: ap_proxy.inner().path().to_owned(),
                            device_path: device_proxy.inner().path().to_owned(),
                        },
                    );
                }

                let aps = aps
                    .into_values()
                    .sorted_by(|a, b| a.strength.cmp(&b.strength))
                    .collect();

                Ok(aps)
            })
            .collect();

        let mut wireless_access_points = Vec::with_capacity(wireless_ap_futures.len());

        for f in wireless_ap_futures {
            let mut access_points: anyhow::Result<Vec<AccessPoint>> = f.await;
            if let Ok(access_points) = &mut access_points {
                wireless_access_points.append(access_points);
            }
        }

        wireless_access_points.sort_by(|a, b| b.strength.cmp(&a.strength));

        Ok(wireless_access_points)
    }
}
