// cosmic-connect-applet/src/main.rs
use cosmic::app::{Core, Task};
use cosmic::iced::{Limits, Subscription, window};
use cosmic::iced::platform_specific::shell::commands::popup::{destroy_popup, get_popup};
use cosmic::iced_runtime::core::window::Id as SurfaceId;
use cosmic::Element;
use cosmic::widget;
use std::collections::HashMap;
use lazy_static::lazy_static;
use tokio::sync::Mutex;

mod backend;
mod messages;
mod models;
mod notifications;
mod portal;
mod plugins;
mod ui;

use messages::Message;
use models::Device;

const ID: &str = "dev.mal.CosmicConnect.Applet";

lazy_static! {
    static ref PAIRING_RECEIVER: Mutex<Option<tokio::sync::mpsc::Receiver<notifications::PairingNotification>>> = 
        Mutex::new(None);
    static ref BACKEND_INITIALIZED: Mutex<bool> = Mutex::new(false);
}

#[derive(Clone, Default)]
struct KdeConnectApplet {
    core: Core,
    devices: HashMap<String, Device>,
    popup: Option<window::Id>,
    expanded_device: Option<String>,
}

impl cosmic::Application for KdeConnectApplet {
    type Message = Message;
    type Executor = cosmic::executor::multi::Executor;
    type Flags = ();
    const APP_ID: &'static str = ID;

    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        // Initialize pairing notifications
        tokio::task::spawn(async {
            let mut receiver_guard = PAIRING_RECEIVER.lock().await;
            if receiver_guard.is_none() {
                eprintln!("=== Initializing notification listener (ONCE) ===");
                
                let (tx, rx) = tokio::sync::mpsc::channel(100);
                *receiver_guard = Some(rx);
                drop(receiver_guard);
                
                notifications::start_notification_listener(tx, false);
            }
        });
        
        // Initialize backend
        tokio::task::spawn(async {
            let mut init_guard = BACKEND_INITIALIZED.lock().await;
            if !*init_guard {
                eprintln!("=== Initializing KDE Connect backend ===");
                match backend::initialize().await {
                    Ok(_) => {
                        eprintln!("✓ Backend initialized successfully");
                        *init_guard = true;
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to initialize backend: {}", e);
                    }
                }
            }
        });
        
        let applet = KdeConnectApplet {
            core,
            devices: HashMap::new(),
            popup: None,
            expanded_device: None,
        };

        (applet, Task::none())
    }

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn on_close_requested(&self, _id: SurfaceId) -> Option<Message> {
        Some(Message::TogglePopup)
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                if let Some(popup_id) = self.popup.take() {
                    self.expanded_device = None;
                    return destroy_popup(popup_id);
                }
                
                let new_id = window::Id::unique();
                self.popup = Some(new_id);
                
                let mut popup_settings = self.core.applet.get_popup_settings(
                    self.core.main_window_id().unwrap(),
                    new_id,
                    None,
                    None,
                    None,
                );
                
                popup_settings.positioner.size_limits = Limits::NONE
                    .min_width(400.0)
                    .max_width(400.0)
                    .min_height(200.0)
                    .max_height(700.0);
                
                return Task::batch(vec![
                    get_popup(popup_settings),
                    Task::perform(backend::fetch_devices(), |devices| {
                        cosmic::Action::App(Message::DevicesUpdated(devices))
                    })
                ]);
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
            Message::SendFile(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        let files = crate::portal::pick_files(
                            "Select files to send",
                            true,
                            None,
                        ).await;
                        
                        if !files.is_empty() {
                            backend::send_files(id, files).await.ok();
                        }
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::ShareUrl(ref device_id, ref url) => {
                let id = device_id.clone();
                let url_clone = url.clone();
                return Task::perform(
                    async move {
                        backend::share_files(id, vec![url_clone]).await.ok();
                    },
                    |_| cosmic::Action::App(Message::RefreshDevices)
                );
            }
            Message::SendSMS(ref device_id) => {
                let id = device_id.clone();
                let device_name = self.devices.get(device_id)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string());
                
                std::process::Command::new("cosmic-connect-sms")
                    .arg(&id)
                    .arg(&device_name)
                    .spawn()
                    .ok();
            }
            Message::ShareClipboard(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        match std::process::Command::new("wl-paste")
                            .output()
                        {
                            Ok(output) if output.status.success() => {
                                if let Ok(content) = String::from_utf8(output.stdout) {
                                    backend::send_clipboard(id, content).await.ok();
                                }
                            }
                            _ => eprintln!("Failed to get clipboard content"),
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
            Message::AcceptPairing(ref device_id) => {
                let id = device_id.clone();
                return Task::perform(
                    async move {
                        backend::accept_pairing(id).await.ok();
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
                // Process next event from adapter
                if let Some(event) = backend::next_event().await {
                    use kdeconnect_adapter::CoreEvent;
                    
                    match event {
                        CoreEvent::Connected((id, device)) => {
                            eprintln!("✓ Device connected: {} ({})", device.name, id.0);
                            let ui_device: Device = device.into();
                            backend::update_device(id.0.clone(), ui_device).await;
                            return Some((Message::RefreshDevices, ()));
                        }
                        CoreEvent::DevicePaired((id, device)) => {
                            eprintln!("✓ Device paired: {} ({})", device.name, id.0);
                            let ui_device: Device = device.into();
                            backend::update_device(id.0.clone(), ui_device).await;
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
        
        Subscription::batch(vec![
            backend_sub,
            pairing_sub,
            cosmic::iced::time::every(std::time::Duration::from_secs(30))
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