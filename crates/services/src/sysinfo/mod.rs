use futures_signals::signal::Mutable;
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::time::interval;

#[derive(Debug, Clone)]
pub struct SysInfoData {
    pub cpu_usage: f32,
    pub memory_total: u64,
    pub memory_used: u64,
}

#[derive(Debug, Clone)]
pub struct SysInfoSubscriber {
    pub data: Arc<Mutable<SysInfoData>>,
    refresh_rate_ms: u64,
}

impl SysInfoSubscriber {
    pub fn new(refresh_rate_ms: u64) -> Arc<Self> {
        let data = Arc::new(Mutable::new(SysInfoData {
            cpu_usage: 0.0,
            memory_total: 0,
            memory_used: 0,
        }));
        let this = Arc::new(Self {
            data: data.clone(),
            refresh_rate_ms,
        });
        Self::spawn(this.clone());
        this
    }

    fn spawn(this: Arc<Self>) {
        tokio::spawn(async move {
            let mut sys = System::new_all();
            let mut ticker = interval(Duration::from_millis(this.refresh_rate_ms));
            loop {
                ticker.tick().await;
                sys.refresh_cpu_all();
                sys.refresh_memory();
                let cpu_usage =
                    sys.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / sys.cpus().len() as f32;
                let memory_total = sys.total_memory();
                let memory_used = sys.used_memory();
                this.data.set(SysInfoData {
                    cpu_usage,
                    memory_total,
                    memory_used,
                });
            }
        });
    }
}
