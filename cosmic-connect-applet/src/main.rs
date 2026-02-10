// cosmic-connect-applet/src/main.rs

use cosmic::app::Core;
use cosmic::iced::window::Id as SurfaceId;
use cosmic::iced::Subscription;
use cosmic::{widget, Element, Task};
use std::collections::HashMap;
use tokio::sync::Mutex as TokioMutex;
use futures::StreamExt;

// Import from the library instead of declaring modules
use cosmic_connect_applet::backend;
use cosmic_connect_applet::messages::Message;
use cosmic_connect_applet::models::Device;
use cosmic_connect_applet::portal;
use cosmic_connect_applet::ui;

lazy_static::lazy_static! {
    static ref LAST_PAIR_TIME: TokioMutex<Option<std::time::Instant>> 
        = TokioMutex::new(None);
}

pub struct KdeConnectApplet {
    core: Core,
    popup: Option<SurfaceId>,
    devices: HashMap<String, Device>,
    expanded_device: Option<String>,
}

impl cosmic::Application for KdeConnectApplet {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "io.github.M4LC0ntent.CosmicConnect";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        tokio::spawn(async {
            if let Err(e) = backend::initialize().await {
                eprintln!("Failed to initialize backend: {:?}", e);
            }
        });

        let app = KdeConnectApplet {
            core,
            popup: None,
            devices: HashMap::new(),
            expanded_device: None,
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: SurfaceId) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::TogglePopup => {
                if let Some(p) = self.popup.take() {
                    return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(p);
                } else {
                    let new_id = SurfaceId::unique();
                    self.popup.replace(new_id);

                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = cosmic::iced::Limits::NONE
                        .max_width(400.0)
                        .min_width(300.0)
                        .max_height(600.0);

                    return Task::batch(vec![
                        cosmic::iced::platform_specific::shell::commands::popup::get_popup(popup_settings),
                        Task::perform(backend::fetch_devices(), |devices| {
                            cosmic::Action::App(Message::DevicesUpdated(devices))
                        })
                    ]);
                }
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::RefreshDevices => {
                return Task::perform(backend::fetch_devices(), |devices| {
                    cosmic::Action::App(Message::DevicesUpdated(devices))
                });
            }
            Message::DevicesUpdated(devices) => {
                self.devices.clear();
                for device in devices {
                    self.devices.insert(device.id.clone(), device);
                }
            }
            Message::DelayedRefresh => {
                eprintln!("=== Delayed Refresh Triggered ===");
                return Task::perform(backend::fetch_devices(), |devices| {
                    cosmic::Action::App(Message::DevicesUpdated(devices))
                });
            }
            Message::ToggleDeviceMenu(ref device_id) => {
                if self.expanded_device.as_ref() == Some(device_id) {
                    self.expanded_device = None;
                } else {
                    self.expanded_device = Some(device_id.clone());
                }
            }
            Message::PingDevice(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        backend::ping_device(id).await.ok();
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::RingDevice(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        backend::ring_device(id).await.ok();
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::BrowseDevice(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        backend::browse_device_filesystem(id).await.ok();
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::PairDevice(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        backend::pair_device(id).await.ok();
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::UnpairDevice(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        backend::unpair_device(id).await.ok();
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::SendFiles(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        let files = portal::pick_files("Select files to send", true, None).await;
                        if !files.is_empty() {
                            backend::send_files(id, files).await.ok();
                        }
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::ShareText(ref device_id) => {
                eprintln!("Share text requested for device: {}", device_id);
            }
            Message::ShareUrl(ref device_id) => {
                eprintln!("Share URL requested for device: {}", device_id);
            }
            Message::ShareClipboard(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        if let Ok(content) = portal::read_clipboard().await {
                            backend::send_clipboard(id, content).await.ok();
                        }
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::RemoteInput(ref device_id) => {
                eprintln!("Remote input requested for device: {}", device_id);
            }
            Message::LockDevice(ref device_id) => {
                eprintln!("Lock device requested for device: {}", device_id);
            }
            Message::PresenterMode(ref device_id) => {
                eprintln!("Presenter mode requested for device: {}", device_id);
            }
            Message::UseAsMonitor(ref device_id) => {
                eprintln!("Use as monitor requested for device: {}", device_id);
            }
            Message::SendSMS(ref device_id) => {
                eprintln!("=== Launching SMS App ===");
                eprintln!("Device: {}", device_id);
                
                // Get device name
                let device_name = self.devices.get(device_id)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                let device_id_clone = device_id.clone();
                
                // Launch SMS app
                return Task::perform(
                    async move {
                        match std::process::Command::new("cosmic-connect-sms")
                            .arg(&device_id_clone)
                            .arg(&device_name)
                            .spawn()
                        {
                            Ok(_) => eprintln!("✓ SMS app launched"),
                            Err(e) => eprintln!("✗ Failed to launch SMS app: {:?}", e),
                        }
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::OpenSettings => {
                eprintln!("Open settings requested");
            }
            Message::AcceptPairing(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        backend::accept_pairing(id).await.ok();
                        // Mark pairing event time
                        let mut time = LAST_PAIR_TIME.lock().await;
                        *time = Some(std::time::Instant::now());
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::RejectPairing(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        backend::reject_pairing(id).await.ok();
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::PairingRequestReceived(device_id, device_name, device_type) => {
                eprintln!("=== Pairing Request Notification ===");
                eprintln!("Device: {} ({})", device_name, device_id);
                eprintln!("Type: {}", device_type);
            }
            Message::MprisReceived(device_id, mpris_data) => {
                eprintln!("=== MPRIS Data Received ===");
                eprintln!("Device: {}", device_id);
                eprintln!("MPRIS Data: {:?}", mpris_data);
                
                // TODO: Store MPRIS data and expose via D-Bus for COSMIC media controls
                // For now, just log the event - full D-Bus MPRIS proxy implementation needed
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("phone-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }
    
    fn view_window(&self, id: SurfaceId) -> Element<'_, Self::Message> {
        let Some(popup_id) = self.popup else {
            return widget::text("").into();
        };
        
        if id != popup_id {
            return widget::text("").into();
        }
        
        let content = ui::popup::create_popup_view(&self.devices, self.expanded_device.as_ref(), None);
        
        self.core
            .applet
            .popup_container(content)
            .into()
    }
    
    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // D-Bus event subscription
        let dbus_events_sub = Subscription::run_with_id(
            "dbus-events",
            cosmic::iced_futures::futures::stream::unfold(None, |stream_opt| async {
                // Initialize stream on first call
                let mut stream = if let Some(s) = stream_opt {
                    s
                } else {
                    backend::event_stream().await
                };

                // Get next event
                if let Some(event) = stream.next().await {
                    use kdeconnect_dbus_client::ServiceEvent;
                    
                    let message = match event {
                        ServiceEvent::DeviceConnected(device_id, device) => {
                            eprintln!("✓ D-Bus: Device connected - {}", device.name);
                            let ui_device = Device {
                                id: device_id.clone(),
                                name: device.name,
                                device_type: "phone".to_string(),
                                is_paired: device.is_paired,
                                is_reachable: device.is_reachable,
                                battery_level: None,
                                is_charging: None,
                                network_type: None,
                                signal_strength: None,
                                pairing_requests: 0,
                                has_battery: false,
                                has_ping: true,
                                has_sms: true,
                                has_contacts: false,
                                has_clipboard: true,
                                has_findmyphone: true,
                                has_share: true,
                                has_sftp: false,
                                has_mpris: false,
                                has_remote_keyboard: false,
                                has_presenter: false,
                                has_lockdevice: false,
                                has_virtualmonitor: false,
                            };
                            backend::update_device(device_id, ui_device).await;
                            Message::RefreshDevices
                        }
                        ServiceEvent::DevicePaired(device_id, device) => {
                            eprintln!("✓ D-Bus: Device paired - {}", device.name);
                            
                            // Show notification
                            let notification_title = "Device Paired";
                            let notification_body = format!("{} is now paired", device.name);
                            
                            if let Err(e) = Self::show_notification(&notification_title, &notification_body) {
                                eprintln!("Failed to show notification: {:?}", e);
                            }
                            
                            let ui_device = Device {
                                id: device_id.clone(),
                                name: device.name,
                                device_type: "phone".to_string(),
                                is_paired: true,
                                is_reachable: device.is_reachable,
                                battery_level: None,
                                is_charging: None,
                                network_type: None,
                                signal_strength: None,
                                pairing_requests: 0,
                                has_battery: false,
                                has_ping: true,
                                has_sms: true,
                                has_contacts: false,
                                has_clipboard: true,
                                has_findmyphone: true,
                                has_share: true,
                                has_sftp: false,
                                has_mpris: false,
                                has_remote_keyboard: false,
                                has_presenter: false,
                                has_lockdevice: false,
                                has_virtualmonitor: false,
                            };
                            backend::update_device(device_id, ui_device).await;
                            
                            // Mark pairing time for delayed refresh
                            let mut time = LAST_PAIR_TIME.lock().await;
                            *time = Some(std::time::Instant::now());
                            
                            Message::RefreshDevices
                        }
                        ServiceEvent::DeviceDisconnected(device_id) => {
                            eprintln!("✗ D-Bus: Device disconnected - {}", device_id);
                            backend::remove_device(&device_id).await;
                            Message::RefreshDevices
                        }
                        ServiceEvent::SmsMessagesReceived(messages_json) => {
                            eprintln!("📨 D-Bus: SMS messages received");
                            // SMS app will handle this
                            Message::RefreshDevices
                        }
                    };
                    
                    return Some((message, Some(stream)));
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Some((Message::RefreshDevices, Some(stream)))
            })
        );
        
        // Delayed refresh subscription
        let delayed_refresh_sub = Subscription::run_with_id(
            "delayed-refresh",
            cosmic::iced_futures::futures::stream::unfold((), |_| async {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                let time_guard = LAST_PAIR_TIME.lock().await;
                if let Some(last_time) = *time_guard {
                    if last_time.elapsed().as_secs() < 10 {
                        drop(time_guard);
                        return Some((Message::DelayedRefresh, ()));
                    }
                }
                // FIXED: Don't return None, keep stream alive with longer sleep
                drop(time_guard);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                Some((Message::RefreshDevices, ()))
            })
        );
        
        Subscription::batch(vec![
            dbus_events_sub,
            delayed_refresh_sub,
            // Less frequent polling as backup
            cosmic::iced::time::every(std::time::Duration::from_secs(30))
                .map(|_| Message::RefreshDevices),
        ])
    }
}

impl KdeConnectApplet {
    fn show_notification(title: &str, body: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;
        
        Command::new("notify-send")
            .arg(title)
            .arg(body)
            .arg("--icon=phone-symbolic")
            .arg("--app-name=KDE Connect")
            .spawn()?;
        
        Ok(())
    }
}

impl Drop for KdeConnectApplet {
    fn drop(&mut self) {
        eprintln!("=== KdeConnectApplet Drop called ===");
    }
}

fn main() -> cosmic::iced::Result {
    eprintln!("=== KDE Connect Applet Starting ===");
    
    ctrlc::set_handler(move || {
        eprintln!("=== Shutdown signal received (SIGTERM/SIGINT) ===");
        eprintln!("Exiting gracefully");
        std::process::exit(0);
    })
    .ok();
    
    cosmic::applet::run::<KdeConnectApplet>(())
}
