// cosmic-connect-applet/src/main.rs
mod models;
mod messages;
mod backend;
mod ui;
mod plugins;
mod notifications;

use cosmic::app::Core;
use cosmic::iced::{window, Limits, Subscription};
use cosmic::iced::window::Id as SurfaceId;
use cosmic::iced::Task as Command;
use cosmic::{Element, Action};
use cosmic::widget;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use models::Device;
use messages::Message;

const ICON_PHONE: &str = "phone-symbolic";

lazy_static::lazy_static! {
    static ref PAIRING_RECEIVER: Arc<Mutex<Option<tokio::sync::mpsc::Receiver<notifications::PairingNotification>>>> = 
        Arc::new(Mutex::new(None));
}

pub struct KdeConnectApplet {
    core: Core,
    devices: HashMap<String, Device>,
    popup: Option<window::Id>,
    expanded_device: Option<String>,
}

impl cosmic::Application for KdeConnectApplet {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &str = "io.github.M4LC0ntent.CosmicKdeConnect";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(
        core: Core,
        _flags: Self::Flags,
    ) -> (Self, Command<Action<Self::Message>>) {
        tokio::spawn(async {
            eprintln!("=== Initializing KDE Connect backend ===");
            if let Err(e) = backend::initialize().await {
                eprintln!("Failed to initialize backend: {}", e);
                return;
            }
            eprintln!("✓ Backend initialized");
            
            loop {
                if let Some(event) = backend::next_event().await {
                    match event {
                        kdeconnect_adapter::CoreEvent::Connected((id, device)) => {
                            eprintln!("✓ Device connected: {} ({})", device.name, id.0);
                            let ui_device: Device = device.into();
                            backend::update_device(id.0.clone(), ui_device).await;
                        }
                        kdeconnect_adapter::CoreEvent::DevicePaired((id, device)) => {
                            eprintln!("✓ Device paired: {} ({})", device.name, id.0);
                            let ui_device: Device = device.into();
                            backend::update_device(id.0.clone(), ui_device).await;
                        }
                        kdeconnect_adapter::CoreEvent::Disconnected(id) => {
                            eprintln!("✗ Device disconnected: {}", id.0);
                            backend::remove_device(&id.0).await;
                        }
                        kdeconnect_adapter::CoreEvent::ClipboardReceived(content) => {
                            eprintln!("📋 Clipboard received: {}", content);
                        }
                        _ => {}
                    }
                }
            }
        });
        
        tokio::spawn(async {
            let mut receiver_guard = PAIRING_RECEIVER.lock().await;
            
            if receiver_guard.is_none() {
                eprintln!("=== Initializing notification listener (ONCE) ===");
                
                let (tx, rx) = tokio::sync::mpsc::channel(100);
                *receiver_guard = Some(rx);
                drop(receiver_guard);
                
                notifications::start_notification_listener(tx, false);
            }
        });
        
        let applet = KdeConnectApplet {
            core,
            devices: HashMap::new(),
            popup: None,
            expanded_device: None,
        };

        (applet, Command::perform(backend::fetch_devices(), |devices| {
            Action::App(Message::DevicesUpdated(devices))
        }))
    }

    fn on_close_requested(&self, _id: SurfaceId) -> Option<Message> {
        Some(Message::TogglePopup)
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> Command<Action<Self::Message>> {
        match message {
            Message::TogglePopup => {
                if let Some(popup_id) = self.popup.take() {
                    self.expanded_device = None;
                    return cosmic::iced::platform_specific::shell::commands::popup::destroy_popup(popup_id);
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
                
                return Command::batch(vec![
                    cosmic::iced::platform_specific::shell::commands::popup::get_popup(popup_settings),
                    Command::perform(backend::fetch_devices(), |devices| {
                        Action::App(Message::DevicesUpdated(devices))
                    })
                ]);
            }
            Message::RefreshDevices => {
                return Command::perform(backend::fetch_devices(), |devices| {
                    Action::App(Message::DevicesUpdated(devices))
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
                return Command::perform(
                    async move {
                        backend::ping_device(id).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
                );
            }
            Message::RingDevice(ref device_id) => {
                let id = device_id.clone();
                return Command::perform(
                    async move {
                        backend::ring_device(id).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
                );
            }
            Message::BrowseDevice(ref device_id) => {
                let id = device_id.clone();
                return Command::perform(
                    async move {
                        backend::browse_device_filesystem(id).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
                );
            }
            Message::PairDevice(ref device_id) => {
                let id = device_id.clone();
                return Command::perform(
                    async move {
                        backend::pair_device(id).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
                );
            }
            Message::UnpairDevice(ref device_id) => {
                let id = device_id.clone();
                return Command::perform(
                    async move {
                        backend::unpair_device(id).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
                );
            }
            Message::SendFile(ref device_id) => {
                let id = device_id.clone();
                // TODO: Open file picker and get files
                let files = vec![]; // Placeholder
                return Command::perform(
                    async move {
                        backend::send_files(id, files).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
                );
            }
            Message::ShareUrl(ref device_id, ref url) => {
                let id = device_id.clone();
                let url_clone = url.clone();
                return Command::perform(
                    async move {
                        backend::share_files(id, vec![url_clone]).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
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
                return Command::perform(
                    async move {
                        // TODO: Get clipboard content
                        let content = String::new();
                        backend::send_clipboard(id, content).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
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
                return Command::perform(
                    async move {
                        backend::pair_device(id).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
                );
            }
            Message::RejectPairing(ref device_id) => {
                let id = device_id.clone();
                return Command::perform(
                    async move {
                        backend::unpair_device(id).await.ok();
                    },
                    |_| Action::App(Message::RefreshDevices)
                );
            }
            Message::PairingRequestReceived(device_id, device_name, device_type) => {
                eprintln!("=== Pairing Request in Main App ===");
                eprintln!("Device: {} ({})", device_name, device_id);
                eprintln!("Type: {}", device_type);
                
                tokio::spawn(async move {
                    if let Err(e) = notifications::show_pairing_notification(&device_name, &device_id).await {
                        eprintln!("Failed to show notification: {}", e);
                    }
                });
                
                return Command::perform(backend::fetch_devices(), |devices| {
                    Action::App(Message::DevicesUpdated(devices))
                });
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button(ICON_PHONE)
            .on_press_down(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, id: SurfaceId) -> Element<'_, Self::Message> {
        if !matches!(self.popup, Some(popup_id) if popup_id == id) {
            return widget::text("").into();
        }
        
        ui::popup::create_popup_view(&self.devices, self.expanded_device.as_ref(), None)
    }
    
    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let pairing_sub = Subscription::run_with_id(
            "pairing-notifications",
            futures::stream::unfold((), |_| async {
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
            cosmic::iced::time::every(std::time::Duration::from_secs(5))
                .map(|_| Message::RefreshDevices),
            pairing_sub,
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