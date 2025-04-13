use anyhow::Ok;
use futures_signals::signal::{Mutable, MutableSignalCloned};
use futures_util::StreamExt;
use futures_util::stream::select_all;
use tracing::{debug, info};
use zbus::Connection;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};

use self::dbus::access_point::AccessPointProxy;
use self::dbus::statistics::StatisticsProxy;
use self::device::DeviceState;
use self::nm::NetworkManager;

pub mod access_point;
pub mod active_connection;
pub mod dbus;
pub mod device;
pub mod nm;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AccessPoint {
    pub ssid: String,
    pub strength: u8,
    pub state: DeviceState,
    pub public: bool,
    pub working: bool,
    pub path: ObjectPath<'static>,
    pub device_path: ObjectPath<'static>,
}

#[derive(Debug, Clone)]
pub enum ActiveConnectionInfo {
    Wired {
        name: String,
        speed: u32,
    },
    WiFi {
        id: String,
        name: String,
        strength: u8,
        device: String,
    },
    Vpn {
        name: String,
        object_path: OwnedObjectPath,
    },
}

impl ActiveConnectionInfo {
    pub fn name(&self) -> String {
        match &self {
            Self::Wired { name, .. } => name.clone(),
            Self::WiFi { name, .. } => name.clone(),
            Self::Vpn { name, .. } => name.clone(),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityState {
    None,
    Portal,
    Loss,
    Full,
    #[default]
    Unknown,
}

impl From<u32> for ConnectivityState {
    fn from(state: u32) -> ConnectivityState {
        match state {
            1 => ConnectivityState::None,
            2 => ConnectivityState::Portal,
            3 => ConnectivityState::Loss,
            4 => ConnectivityState::Full,
            _ => ConnectivityState::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkStatistics {
    prev_tx: u64,
    prev_rx: u64,
    prev_tx_time: i64,
    prev_rx_time: i64,
    tx: u64,
    rx: u64,
    tx_time: i64,
    rx_time: i64,
    pub device: String,
}

impl NetworkStatistics {
    pub fn rx_speed(&self) -> f64 {
        let elapsed = self.rx_time - self.prev_rx_time;
        if elapsed == 0 {
            0.0
        } else {
            (self.rx - self.prev_rx) as f64 / elapsed as f64
        }
    }

    pub fn tx_speed(&self) -> f64 {
        let elapsed = self.tx_time - self.prev_tx_time;
        if elapsed == 0 {
            0.0
        } else {
            (self.tx - self.prev_tx) as f64 / elapsed as f64
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkData {
    pub wifi_enabled: bool,
    pub active_connections: Vec<ActiveConnectionInfo>,
    pub wireless_access_points: Vec<AccessPoint>,
    pub connectivity: ConnectivityState,
    pub network_statistics: Vec<NetworkStatistics>,
}

impl NetworkData {
    async fn init(conn: &Connection) -> anyhow::Result<Self> {
        let nm = NetworkManager::new(conn).await?;
        let wifi_enabled = nm.wireless_enabled().await?;
        let connectivity = nm.connectivity().await?.into();
        let active_connections = nm.active_connections().await?;
        let wireless_access_points = nm.wireless_access_points().await?;
        let network_statistics = nm.network_statistics().await?;

        Ok(Self {
            wifi_enabled,
            connectivity,
            active_connections,
            wireless_access_points,
            network_statistics,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Subscriber {
    data: Mutable<NetworkData>,
    conn: Connection,
    refresh_rate_ms: u32,
}

impl Subscriber {
    pub async fn new() -> anyhow::Result<Self> {
        let conn = Connection::system().await?;
        let data = Mutable::new(NetworkData::init(&conn).await?);

        Ok(Self {
            data,
            conn,
            refresh_rate_ms: 1000,
        })
    }

    pub fn set_refresh_rate_ms(&mut self, refresh_rate_ms: u32) {
        self.refresh_rate_ms = refresh_rate_ms;
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        info!("NetworkManager subscriber start");

        let nm = NetworkManager::new(&self.conn).await?;

        let wireless_enabled = nm
            .receive_wireless_enabled_changed()
            .await
            .then(|v| async move {
                let value = v.get().await.unwrap_or_default();

                debug!("WiFi enabled changed: {}", value);
                self.data.lock_mut().wifi_enabled = value;
            })
            .boxed();

        let connectivity_changed = nm
            .receive_connectivity_changed()
            .await
            .then(|val| async move {
                let value = val.get().await.unwrap_or_default().into();

                debug!("Connectivity changed: {:?}", value);
                self.data.lock_mut().connectivity = value;
            })
            .boxed();

        let active_connections = nm
            .receive_active_connections_changed()
            .await
            .then(|_| async move {
                let nm = NetworkManager::new(&self.conn).await.unwrap();
                let value = nm.active_connections().await.unwrap_or_default();

                debug!("Active connections changed: {:?}", value);
                self.data.lock_mut().active_connections = value;
            })
            .boxed();

        let wireless_ac = nm.wireless_access_points().await?;

        let mut strength_changes = Vec::with_capacity(wireless_ac.len());
        for ap in wireless_ac {
            let ssid = ap.ssid;
            let ap_proxy = AccessPointProxy::builder(&self.conn)
                .path(ap.path.clone())?
                .build()
                .await?;

            strength_changes.push(
                ap_proxy
                    .receive_strength_changed()
                    .await
                    .then(move |val| {
                        let ssid = ssid.clone();
                        async move {
                            let value = val.get().await.unwrap_or_default();

                            debug!("Strength changed value: {}, {}", &ssid, value);
                            {
                                let mut data = self.data.lock_mut();
                                if let Some(ap) = data
                                    .wireless_access_points
                                    .iter_mut()
                                    .find(|ap| ap.ssid == ssid)
                                {
                                    ap.strength = value;
                                }
                            }

                            {
                                let mut data = self.data.lock_mut();
                                if let Some(ActiveConnectionInfo::WiFi { strength, .. }) = data
                                    .active_connections
                                    .iter_mut()
                                    .find(|ac| ac.name() == ssid)
                                {
                                    *strength = value;
                                }
                            }
                        }
                    })
                    .boxed(),
            );
        }

        let devices = nm.devices().await?;
        let mut statistics_changes = Vec::new();
        for device in devices {
            let device_string = device.to_string();
            let statstics_proxy = StatisticsProxy::builder(&self.conn)
                .path(device.clone())?
                .build()
                .await?;
            statstics_proxy
                .set_refresh_rate_ms(self.refresh_rate_ms)
                .await
                .unwrap();

            let device_clone = device_string.clone();
            statistics_changes.push(
                statstics_proxy
                    .receive_rx_bytes_changed()
                    .await
                    .then(move |val| {
                        let device_str = device_clone.clone();
                        async move {
                            let value = val.get().await.unwrap_or_default();
                            debug!("Rx bytes changed value: {}", value);
                            let mut data = self.data.lock_mut();
                            for stat in data.network_statistics.iter_mut() {
                                if stat.device == device_str {
                                    stat.prev_rx = stat.rx;
                                    stat.prev_rx_time = stat.rx_time;
                                    stat.rx = value;
                                    stat.rx_time = chrono::Utc::now().timestamp();
                                }
                            }
                        }
                    })
                    .boxed(),
            );

            statistics_changes.push(
                statstics_proxy
                    .receive_tx_bytes_changed()
                    .await
                    .then(move |val| {
                        let device_str = device_string.clone();
                        async move {
                            let value = val.get().await.unwrap_or_default();
                            debug!("Tx bytes changed value: {}", value);
                            let mut data = self.data.lock_mut();
                            for stat in data.network_statistics.iter_mut() {
                                if stat.device == device_str {
                                    stat.prev_tx = stat.tx;
                                    stat.prev_tx_time = stat.tx_time;
                                    stat.tx = value;
                                    stat.tx_time = chrono::Utc::now().timestamp();
                                }
                            }
                        }
                    })
                    .boxed(),
            );
        }

        let mut events = select_all(vec![
            wireless_enabled,
            connectivity_changed,
            active_connections,
        ]);

        for stream in strength_changes {
            events.push(stream);
        }

        for stream in statistics_changes {
            events.push(stream);
        }

        while let Some(_) = events.next().await {}

        Ok(())
    }

    pub fn subscribe(&self) -> MutableSignalCloned<NetworkData> {
        self.data.signal_cloned()
    }
}
