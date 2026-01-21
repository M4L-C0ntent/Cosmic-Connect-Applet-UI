// cosmic-connect-applet/src/plugins/sms/mod.rs

// #[allow(dead_code)] = Placeholder for code that will be used once features are fully integrated

mod app;
mod dbus;
mod emoji;
mod messages;
mod models;
mod utils;
mod views;

pub use app::SmsWindow;

/// Run the SMS window application
#[allow(dead_code)]
pub fn run(device_id: String, device_name: String) -> cosmic::iced::Result {
    let settings = cosmic::app::Settings::default();
    cosmic::app::run::<SmsWindow>(settings, (device_id, device_name))
}