// cosmic-connect-applet/src/main.rs

use cosmic::app::Core;
use cosmic::iced::window::Id as SurfaceId;
use cosmic::iced::Subscription;
use cosmic::iced_runtime::core::window::Id as WindowId;
use cosmic::{widget, Element, Task};
use std::collections::HashMap;
use tokio::sync::Mutex as TokioMutex;

// Import from the library instead of declaring modules
use cosmic_connect_applet::backend;
use cosmic_connect_applet::messages::Message;
use cosmic_connect_applet::models::Device;
use cosmic_connect_applet::notifications::{self, PairingNotification};
use cosmic_connect_applet::portal;
use cosmic_connect_applet::ui;

lazy_static::lazy_static! {
    static ref PAIRING_RECEIVER: TokioMutex<Option<tokio::sync::mpsc::Receiver<PairingNotification>>> 
        = TokioMutex::new(None);
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
        let (tx, rx) = tokio::sync::mpsc::channel::<PairingNotification>(10);
        tokio::spawn(async move {
            let mut receiver_guard = PAIRING_RECEIVER.lock().await;
            *receiver_guard = Some(rx);
        });

        notifications::start_notification_listener(tx, false);

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
                        WindowId::from(new_id),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = cosmic::iced::Limits::NONE
                        .max_width(400.0)
                        .min_width(300.0)
                        .min_height(200.0)
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
                        } else {
                            eprintln!("Failed to get clipboard content");
                        }
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::OpenSettings => {
                std::process::Command::new("cosmic-connect-settings")
                    .spawn()
                    .ok();
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
                eprintln!("SMS requested for device: {}", device_id);
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
        
        ui::popup::create_popup_view(&self.devices, self.expanded_device.as_ref(), None)
    }
    
    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Backend event processing subscription
        let backend_sub = Subscription::run_with_id(
            "backend-events",
            cosmic::iced_futures::futures::stream::unfold((), |_| async {
                if let Some(event) = backend::next_event().await {
                    use kdeconnect_adapter::CoreEvent;
                    
                    match event {
                        CoreEvent::Connected((id, device)) => {
                            eprintln!("✓ Device connected: {} ({})", device.name, id.0);
                            let ui_device: Device = device.into();
                            backend::update_device(id.0.clone(), ui_device).await;
                            
                            // Mark pairing time and trigger refresh
                            let mut time = LAST_PAIR_TIME.lock().await;
                            *time = Some(std::time::Instant::now());
                            
                            return Some((Message::RefreshDevices, ()));
                        }
                        CoreEvent::DevicePaired((id, device)) => {
                            eprintln!("✓ Device paired: {} ({})", device.name, id.0);
                            let ui_device: Device = device.into();
                            backend::update_device(id.0.clone(), ui_device).await;
                            
                            // Mark pairing time and trigger refresh
                            let mut time = LAST_PAIR_TIME.lock().await;
                            *time = Some(std::time::Instant::now());
                            
                            return Some((Message::RefreshDevices, ()));
                        }
                        CoreEvent::Disconnected(id) => {
                            eprintln!("✗ Device disconnected: {}", id.0);
                            backend::remove_device(&id.0).await;
                            return Some((Message::RefreshDevices, ()));
                        }
                        _ => {}
                    }
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                None
            })
        );
        
        // Pairing notifications subscription
        let pairing_sub = Subscription::run_with_id(
            "pairing-notifications",
            cosmic::iced_futures::futures::stream::unfold((), |_| async {
                let mut receiver_guard = PAIRING_RECEIVER.lock().await;
                
                if let Some(rx) = receiver_guard.as_mut() {
                    if let Some(notification) = rx.recv().await {
                        return Some((notification, ()));
                    }
                }
                
                drop(receiver_guard);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                None
            })
        ).map(|notification| Message::PairingRequestReceived(
            notification.device_id,
            notification.device_name,
            notification.device_type,
        ));
        
        // Delayed refresh subscription - triggers extra refreshes after pairing
        let delayed_refresh_sub = Subscription::run_with_id(
            "delayed-refresh",
            cosmic::iced_futures::futures::stream::unfold((), |_| async {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                let time_guard = LAST_PAIR_TIME.lock().await;
                if let Some(last_time) = *time_guard {
                    // Trigger extra refreshes for 10 seconds after pairing event
                    if last_time.elapsed().as_secs() < 10 {
                        drop(time_guard);
                        eprintln!("=== Post-Pairing Refresh ({}s elapsed) ===", last_time.elapsed().as_secs());
                        return Some((Message::DelayedRefresh, ()));
                    }
                }
                None
            })
        );
        
        Subscription::batch(vec![
            backend_sub,
            pairing_sub,
            delayed_refresh_sub,
            // Reduced from 30s to 10s for more responsive updates
            cosmic::iced::time::every(std::time::Duration::from_secs(10))
                .map(|_| Message::RefreshDevices),
        ])
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