use futures_util::StreamExt;
use services::upower::{self, BatteryState};
use ui::prelude::*;

#[derive(Debug, Clone)]
struct Properties {
    percentage: f64,
    state: BatteryState,
    time_to_full: i64,
    time_to_empty: i64,
}

#[derive(Debug, Clone)]
enum Message {
    Update(Properties),
}

impl EventEmitter<Message> for Upower {}

pub struct Upower {
    label: String,
    icon: Icons,
    icon_color: Option<Rgba>,
}

impl Upower {
    // async fn connect() {
    //     let display_proxy = create_upower_proxy().await.unwrap();
    //
    //     let device_interface_name =
    //         zbus::names::InterfaceName::from_static_str("org.freedesktop.UPower.Device")
    //             .expect("failed to create zbus InterfaceName");
    //
    //     let mut prop_changed_stream = display_proxy.receive_properties_changed().await.unwrap();
    //     while let Some(signal) = prop_changed_stream.next().await {
    //         let args = signal.args().expect("Invalid signal arguments");
    //         if args.interface_name != device_interface_name {
    //             continue;
    //         }
    //
    //         for (name, changed_value) in args.changed_properties {
    //             match name {
    //                 "Percentage" => println!(
    //                     "Percentage: {}",
    //                     changed_value
    //                         .downcast_ref::<f64>()
    //                         .expect("expected Percentage to be f64")
    //                 ),
    //                 "IconName" => println!(
    //                     "IconName: {:?}",
    //                     changed_value
    //                         .downcast_ref::<&str>()
    //                         .expect("expected IconName to be &str")
    //                 ),
    //                 "State" => println!(
    //                     "State: {:?}",
    //                     changed_value
    //                         .downcast_ref::<u32>()
    //                         .expect("expected State to be u32")
    //                 ),
    //                 "TimeToFull" => println!(
    //                     "TimeToFull: {:?}",
    //                     changed_value
    //                         .downcast_ref::<i64>()
    //                         .expect("expected TimeToFull to be u32")
    //                 ),
    //                 "TimeToEmpty" => println!(
    //                     "TimeToEmpty: {:?}",
    //                     changed_value
    //                         .downcast_ref::<i64>()
    //                         .expect("expected TimeToEmpty to be u32")
    //                 ),
    //                 _ => {}
    //             }
    //         }
    //     }
    // }

    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        // cx.spawn(async move |this, cx| {});
        // let proxy = upower::create_upower_proxy().await.unwrap();

        // let properties = proxy.get_all(upower::DEVICE_INTERFACE_NAME).await?;

        // let percentage = properties["Percentage"]
        //     .downcast_ref::<f64>()
        //     .expect("expected percentage: f64 in HashMap of all properties");
        // let icon_name = properties["IconName"]
        //     .downcast_ref::<&str>()
        //     .expect("expected IconName: str in HashMap of all properties")
        //     .to_string();
        // let state = u32_to_battery_state(
        //     properties["State"]
        //         .downcast_ref::<u32>()
        //         .expect("expected State: u32 in HashMap of all properties"),
        // )
        // .unwrap_or(BatteryState::Unknown);
        // let time_to_full = properties["TimeToFull"]
        //     .downcast_ref::<i64>()
        //     .expect("expected TimeToFull: i64 in HashMap of all properties");
        // let time_to_empty = properties["TimeToEmpty"]
        //     .downcast_ref::<i64>()
        //     .expect("expected TimeToEmpty: i64 in HashMap of all properties");
        // let mut properties = UpowerProperties {
        //     percentage,
        //     icon_name: icon_name.clone(),
        //     state,
        //     time_to_full,
        //     time_to_empty,
        // };
        cx.new(|cx| {
            let label = "50".to_string();
            let icon = Icons::BatteryCharging;
            Upower {
                label,
                icon,
                icon_color: None,
            }
        })
    }
}

impl Render for Upower {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .child(Icon::new(self.icon).when_some(self.icon_color, |this, color| this.color(color)))
            .child(self.label.to_string())
    }
}
