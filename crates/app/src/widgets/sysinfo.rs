use futures::StreamExt;
use futures_signals::signal::SignalExt;
use services::sysinfo::SysInfoSubscriber;
use std::sync::Arc;
use ui::prelude::*;

pub struct SysInfoWidget {
    sysinfo: Arc<SysInfoSubscriber>,
    cpu_usage: f32,
}

impl SysInfoWidget {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        let sysinfo = SysInfoSubscriber::new(1000); // 1s 刷新
        let entity = cx.new(|_| Self {
            sysinfo: sysinfo.clone(),
            cpu_usage: 0.0,
        });

        // 订阅数据变化
        let weak = entity.downgrade();
        cx.spawn(async move |_, cx: &mut AsyncApp| {
            let mut signal_stream = sysinfo.data.signal_cloned().to_stream();
            while let Some(data) = signal_stream.next().await {
                if let Some(this) = weak.upgrade() {
                    cx.update_entity(&this, |state, _| {
                        state.cpu_usage = data.cpu_usage;
                    })
                    .ok();
                }
            }
        })
        .detach();

        entity
    }
}

impl Render for SysInfoWidget {
    fn render(
        &mut self,
        _window: &mut ui::Window,
        _cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        h_flex()
            .gap_1()
            .child(format!("CPU: {:.1}%", self.cpu_usage))
    }
}
