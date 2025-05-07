pub mod dbus;

use anyhow::Ok;
use futures_signals::signal::{Mutable, MutableSignalCloned};
use futures_util::StreamExt;
use futures_util::stream::select_all;
use tracing::{debug, info};

pub use dbus::BatteryState;
use zbus::Connection;

use self::dbus::{DeviceProxy, Upower};

#[derive(Debug, Clone)]
pub struct UpowerData {
    pub percentage: f64,
    pub state: BatteryState,
    pub time_to_full: f64,
    pub time_to_empty: f64,
}

impl UpowerData {
    async fn init(conn: &Connection) -> anyhow::Result<Self> {
        let upower = Upower::new(conn).await?;
        let device = upower.get_display_device().await?;
        let device_proxy = DeviceProxy::builder(conn)
            .path(device.inner().path())?
            .build()
            .await?;

        let percentage = device_proxy.percentage().await?;
        let state = device.state().await?;
        let time_to_full = device_proxy.energy_full().await?;
        let time_to_empty = device_proxy.energy_empty().await?;

        Ok(Self {
            percentage,
            state,
            time_to_full,
            time_to_empty,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Subscriber {
    data: Mutable<UpowerData>,
    conn: Connection,
}

impl Subscriber {
    pub async fn new() -> anyhow::Result<Self> {
        let conn = Connection::system().await?;
        let data = UpowerData::init(&conn).await?;
        let data = Mutable::new(data);

        let conn_for_task = conn.clone();
        let data_for_task = data.clone();

        tokio::spawn(async move {
            let sub = Self {
                data: data_for_task,
                conn: conn_for_task,
            };
            sub.run().await.unwrap();
        });

        Ok(Self { data, conn })
    }

    async fn run(&self) -> anyhow::Result<()> {
        info!("Upower subscriber start");

        let upower = Upower::new(&self.conn).await?;
        let device = upower.get_display_device().await?;
        let device_proxy = DeviceProxy::builder(&self.conn)
            .path(device.inner().path())?
            .build()
            .await?;

        let percentages = device_proxy
            .receive_percentage_changed()
            .await
            .then(|v| async move {
                let value = v.get().await.unwrap_or_default();

                debug!("percentage changed: {}", value);
                self.data.lock_mut().percentage = value;
            })
            .boxed();

        let states = device_proxy
            .receive_state_changed()
            .await
            .then(|v| async move {
                let value = v.get().await.unwrap_or(BatteryState::Unknown);

                debug!("state changed: {:?}", value);
                self.data.lock_mut().state = value;
            })
            .boxed();

        let time_to_full = device_proxy
            .receive_energy_full_changed()
            .await
            .then(|v| async move {
                let value = v.get().await.unwrap_or_default();

                debug!("time to full changed: {:?}", value);
                self.data.lock_mut().time_to_full = value;
            })
            .boxed();

        let time_to_empty = device_proxy
            .receive_energy_empty_changed()
            .await
            .then(|v| async move {
                let value = v.get().await.unwrap_or_default();

                debug!("time to empty changed: {:?}", value);
                self.data.lock_mut().time_to_empty = value;
            })
            .boxed();

        let mut events = select_all(vec![percentages, states, time_to_full, time_to_empty]);

        while (events.next().await).is_some() {}

        Ok(())
    }

    pub fn subscribe(&self) -> MutableSignalCloned<UpowerData> {
        self.data.signal_cloned()
    }
}
