// kdeconnect-service/src/dbus_interface.rs
//! D-Bus interface implementation for KDE Connect service

use anyhow::Result;
use kdeconnect_adapter::{KdeConnectAdapter, CoreEvent, DeviceId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use zbus::{Connection, interface};
use zbus::object_server::SignalEmitter;

const SERVICE_NAME: &str = "org.cosmic.KdeConnect";
const DAEMON_PATH: &str = "/org/cosmic/KdeConnect/Daemon";
const SMS_PATH: &str = "/org/cosmic/KdeConnect/Sms";

/// Simplified device info for D-Bus
#[derive(Debug, Clone, Serialize, Deserialize, zbus::zvariant::Type, zbus::zvariant::Value, zbus::zvariant::OwnedValue)]
pub struct DbusDevice {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub is_paired: bool,
    pub is_reachable: bool,
}

/// Main daemon D-Bus interface
pub struct DaemonInterface {
    adapter: Arc<Mutex<KdeConnectAdapter>>,
    devices: Arc<Mutex<HashMap<String, DbusDevice>>>,
}

#[interface(name = "org.cosmic.KdeConnect.Daemon")]
impl DaemonInterface {
    /// List all known devices
    async fn list_devices(&self) -> Vec<DbusDevice> {
        info!("D-Bus: ListDevices called");
        let devices = self.devices.lock().await;
        let device_list: Vec<DbusDevice> = devices.values().cloned().collect();
        info!("D-Bus: Returning {} devices", device_list.len());
        device_list
    }

    /// Pair with a device
    async fn pair_device(&self, device_id: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: PairDevice called for {}", device_id);
        let adapter = self.adapter.lock().await;
        adapter.pair_device(DeviceId(device_id))
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Unpair from a device
    async fn unpair_device(&self, device_id: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: UnpairDevice called for {}", device_id);
        let adapter = self.adapter.lock().await;
        adapter.unpair_device(DeviceId(device_id))
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Send a ping to a device
    async fn send_ping(&self, device_id: String, message: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: SendPing called for {}", device_id);
        let adapter = self.adapter.lock().await;
        adapter.send_ping(DeviceId(device_id), message)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Send files to a device
    async fn send_files(&self, device_id: String, files: Vec<String>) -> zbus::fdo::Result<()> {
        info!("D-Bus: SendFiles called for {} ({} files)", device_id, files.len());
        let adapter = self.adapter.lock().await;
        adapter.send_files(DeviceId(device_id), files)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Send clipboard content
    async fn send_clipboard(&self, device_id: String, content: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: SendClipboard called for {}", device_id);
        let adapter = self.adapter.lock().await;
        adapter.clipboard().send_clipboard(DeviceId(device_id), content)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Signal: Device connected
    #[zbus(signal)]
    async fn device_connected(signal_emitter: &SignalEmitter<'_>, device_id: String, device: DbusDevice) -> zbus::Result<()>;

    /// Signal: Device paired
    #[zbus(signal)]
    async fn device_paired(signal_emitter: &SignalEmitter<'_>, device_id: String, device: DbusDevice) -> zbus::Result<()>;

    /// Signal: Device disconnected
    #[zbus(signal)]
    async fn device_disconnected(signal_emitter: &SignalEmitter<'_>, device_id: String) -> zbus::Result<()>;
}

/// SMS-specific D-Bus interface
pub struct SmsInterface {
    adapter: Arc<Mutex<KdeConnectAdapter>>,
}

#[interface(name = "org.cosmic.KdeConnect.Sms")]
impl SmsInterface {
    /// Request all conversations from device
    async fn request_conversations(&self, device_id: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: RequestConversations called for {}", device_id);
        let adapter = self.adapter.lock().await;
        adapter.sms().request_conversations(DeviceId(device_id))
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Request messages from a specific conversation
    async fn request_conversation(&self, device_id: String, thread_id: i64) -> zbus::fdo::Result<()> {
        info!("D-Bus: RequestConversation called for {} thread {}", device_id, thread_id);
        let adapter = self.adapter.lock().await;
        adapter.sms().request_conversation(DeviceId(device_id), thread_id)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Send an SMS message
    async fn send_sms(&self, device_id: String, phone_number: String, message: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: SendSms called for {}", device_id);
        let adapter = self.adapter.lock().await;
        adapter.sms().send_sms(DeviceId(device_id), phone_number, message)
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))
    }

    /// Signal: SMS messages received
    #[zbus(signal)]
    async fn sms_messages_received(signal_emitter: &SignalEmitter<'_>, messages_json: String) -> zbus::Result<()>;
}

/// Main service coordinator
pub struct KdeConnectService {
    connection: Connection,
    adapter: Arc<Mutex<KdeConnectAdapter>>,
    devices: Arc<Mutex<HashMap<String, DbusDevice>>>,
}

impl KdeConnectService {
    pub async fn new(adapter: KdeConnectAdapter) -> Result<Self> {
        let connection = Connection::session().await?;

        // Request service name
        connection
            .request_name(SERVICE_NAME)
            .await?;

        let adapter = Arc::new(Mutex::new(adapter));
        let devices = Arc::new(Mutex::new(HashMap::new()));

        // Register daemon interface
        let daemon_interface = DaemonInterface {
            adapter: adapter.clone(),
            devices: devices.clone(),
        };
        connection
            .object_server()
            .at(DAEMON_PATH, daemon_interface)
            .await?;

        // Register SMS interface
        let sms_interface = SmsInterface {
            adapter: adapter.clone(),
        };
        connection
            .object_server()
            .at(SMS_PATH, sms_interface)
            .await?;

        Ok(Self {
            connection,
            adapter,
            devices,
        })
    }

    pub async fn run(self) -> Result<()> {
        // Event processing loop
        loop {
            let mut adapter_guard = self.adapter.lock().await;
            if let Some(event) = adapter_guard.next_event().await {
                drop(adapter_guard); // Release lock before processing
                
                self.handle_event(event).await?;
            }
        }
    }

    async fn handle_event(&self, event: CoreEvent) -> Result<()> {
        match event {
            CoreEvent::Connected((device_id, device)) => {
                info!("Event: Device connected - {}", device.name);
                
                let dbus_device = DbusDevice {
                    id: device_id.0.clone(),
                    name: device.name.clone(),
                    device_type: "phone".to_string(),
                    is_paired: matches!(device.pair_state, kdeconnect_adapter::PairState::Paired),
                    is_reachable: true,
                };

                // Store device in cache
                {
                    let mut devices = self.devices.lock().await;
                    devices.insert(device_id.0.clone(), dbus_device.clone());
                    info!("Device cache now has {} devices", devices.len());
                }

                let iface_ref = self.connection
                    .object_server()
                    .interface::<_, DaemonInterface>(DAEMON_PATH)
                    .await?;

                DaemonInterface::device_connected(
                    iface_ref.signal_emitter(),
                    device_id.0,
                    dbus_device,
                ).await?;
            }
            CoreEvent::DevicePaired((device_id, device)) => {
                info!("Event: Device paired - {}", device.name);
                
                let dbus_device = DbusDevice {
                    id: device_id.0.clone(),
                    name: device.name.clone(),
                    device_type: "phone".to_string(),
                    is_paired: true,
                    is_reachable: true,
                };

                // Update device in cache
                {
                    let mut devices = self.devices.lock().await;
                    devices.insert(device_id.0.clone(), dbus_device.clone());
                }

                let iface_ref = self.connection
                    .object_server()
                    .interface::<_, DaemonInterface>(DAEMON_PATH)
                    .await?;

                DaemonInterface::device_paired(
                    iface_ref.signal_emitter(),
                    device_id.0,
                    dbus_device,
                ).await?;
            }
            CoreEvent::Disconnected(device_id) => {
                info!("Event: Device disconnected - {}", device_id.0);

                // Remove device from cache
                {
                    let mut devices = self.devices.lock().await;
                    devices.remove(&device_id.0);
                    info!("Device cache now has {} devices", devices.len());
                }

                let iface_ref = self.connection
                    .object_server()
                    .interface::<_, DaemonInterface>(DAEMON_PATH)
                    .await?;

                DaemonInterface::device_disconnected(
                    iface_ref.signal_emitter(),
                    device_id.0,
                ).await?;
            }
            CoreEvent::SmsMessages(sms_data) => {
                info!("Event: SMS messages received - {} messages", sms_data.messages.len());

                // Serialize to JSON for D-Bus signal
                let messages_json = serde_json::to_string(&sms_data)?;

                let iface_ref = self.connection
                    .object_server()
                    .interface::<_, SmsInterface>(SMS_PATH)
                    .await?;

                SmsInterface::sms_messages_received(
                    iface_ref.signal_emitter(),
                    messages_json,
                ).await?;
            }
            _ => {
                // Handle other events as needed
            }
        }

        Ok(())
    }
}
