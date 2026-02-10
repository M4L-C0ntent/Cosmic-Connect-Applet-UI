// kdeconnect-service/src/dbus_interface.rs
//! D-Bus interface implementation for KDE Connect service

use anyhow::Result;
use kdeconnect_core::{
    KdeConnectCore,
    event::{AppEvent, ConnectionEvent},
    device::{DeviceId, PairState},
    ProtocolPacket,
    PacketType,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
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
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
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
        self.event_sender.send(AppEvent::Pair(DeviceId(device_id)))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    /// Unpair from a device
    async fn unpair_device(&self, device_id: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: UnpairDevice called for {}", device_id);
        self.event_sender.send(AppEvent::Unpair(DeviceId(device_id)))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    /// Send a ping to a device
    async fn send_ping(&self, device_id: String, message: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: SendPing called for {}", device_id);
        self.event_sender.send(AppEvent::Ping((DeviceId(device_id), message)))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    /// Send files to a device
    async fn send_files(&self, device_id: String, files: Vec<String>) -> zbus::fdo::Result<()> {
        info!("D-Bus: SendFiles called for {} ({} files)", device_id, files.len());
        self.event_sender.send(AppEvent::SendFiles((DeviceId(device_id), files)))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    /// Send clipboard content
    async fn send_clipboard(&self, device_id: String, content: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: SendClipboard called for {}", device_id);
        let packet = ProtocolPacket::new(
            PacketType::Clipboard,
            json!({ "content": content })
        );
        self.event_sender.send(AppEvent::SendPacket(DeviceId(device_id), packet))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    /// Ring a device (findmyphone)
    async fn ring_device(&self, device_id: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: RingDevice called for {}", device_id);
        let packet = ProtocolPacket::new(
            PacketType::FindMyPhoneRequest,
            json!({})
        );
        self.event_sender.send(AppEvent::SendPacket(DeviceId(device_id), packet))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
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
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
}

#[interface(name = "org.cosmic.KdeConnect.Sms")]
impl SmsInterface {
    /// Request all conversations from device
    async fn request_conversations(&self, device_id: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: RequestConversations called for {}", device_id);
        let packet = ProtocolPacket::new(
            PacketType::SmsRequest,
            json!({})
        );
        self.event_sender.send(AppEvent::SendPacket(DeviceId(device_id), packet))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    /// Request messages from a specific conversation
    async fn request_conversation(&self, device_id: String, thread_id: i64) -> zbus::fdo::Result<()> {
        info!("D-Bus: RequestConversation called for {} thread {}", device_id, thread_id);
        let packet = ProtocolPacket::new(
            PacketType::SmsRequest,
            json!({ "threadID": thread_id })
        );
        self.event_sender.send(AppEvent::SendPacket(DeviceId(device_id), packet))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    /// Send an SMS message
    async fn send_sms(&self, device_id: String, phone_number: String, message: String) -> zbus::fdo::Result<()> {
        info!("D-Bus: SendSms called for {}", device_id);
        let packet = ProtocolPacket::new(
            PacketType::SmsRequest,
            json!({
                "sendSms": true,
                "phoneNumber": phone_number,
                "messageBody": message
            })
        );
        self.event_sender.send(AppEvent::SendPacket(DeviceId(device_id), packet))
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }

    /// Signal: SMS messages received
    #[zbus(signal)]
    async fn sms_messages_received(signal_emitter: &SignalEmitter<'_>, messages_json: String) -> zbus::Result<()>;
}

/// Main service coordinator
pub struct KdeConnectService {
    connection: Connection,
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
    devices: Arc<Mutex<HashMap<String, DbusDevice>>>,
}

impl KdeConnectService {
    pub async fn new() -> Result<Self> {
        let connection = Connection::session().await?;

        // Request service name
        connection.request_name(SERVICE_NAME).await?;

        // Initialize kdeconnect-core
        let (mut core, mut event_receiver) = KdeConnectCore::new().await?;
        let event_sender = core.take_events();

        let devices = Arc::new(Mutex::new(HashMap::new()));

        // Register daemon interface
        let daemon_interface = DaemonInterface {
            event_sender: event_sender.clone(),
            devices: devices.clone(),
        };
        connection.object_server().at(DAEMON_PATH, daemon_interface).await?;

        // Register SMS interface
        let sms_interface = SmsInterface {
            event_sender: event_sender.clone(),
        };
        connection.object_server().at(SMS_PATH, sms_interface).await?;

        // Spawn core event loop
        tokio::spawn(async move {
            core.run_event_loop().await;
        });

        // Spawn event processor
        let connection_clone = connection.clone();
        let devices_clone = devices.clone();
        tokio::spawn(async move {
            loop {
                if let Some(event) = event_receiver.recv().await {
                    if let Err(e) = Self::handle_event(event, &connection_clone, &devices_clone).await {
                        eprintln!("Error handling event: {:?}", e);
                    }
                }
            }
        });

        Ok(Self {
            connection,
            event_sender,
            devices,
        })
    }

    pub async fn run(self) -> Result<()> {
        // Keep service alive
        std::future::pending::<()>().await;
        Ok(())
    }

    async fn handle_event(
        event: ConnectionEvent,
        connection: &Connection,
        devices: &Arc<Mutex<HashMap<String, DbusDevice>>>,
    ) -> Result<()> {
        match event {
            ConnectionEvent::Connected((device_id, device)) => {
                info!("Event: Device connected - {}", device.name);
                
                let dbus_device = DbusDevice {
                    id: device_id.0.clone(),
                    name: device.name.clone(),
                    device_type: "phone".to_string(),
                    is_paired: matches!(device.pair_state, PairState::Paired),
                    is_reachable: true,
                };
                
                devices.lock().await.insert(device_id.0.clone(), dbus_device.clone());
                
                let iface_ref = connection.object_server()
                    .interface::<_, DaemonInterface>(DAEMON_PATH).await?;
                
                DaemonInterface::device_connected(iface_ref.signal_emitter(), device_id.0, dbus_device).await?;
            }
            ConnectionEvent::DevicePaired((device_id, device)) => {
                info!("Event: Device paired - {}", device.name);
                
                let dbus_device = DbusDevice {
                    id: device_id.0.clone(),
                    name: device.name.clone(),
                    device_type: "phone".to_string(),
                    is_paired: true,
                    is_reachable: true,
                };
                
                devices.lock().await.insert(device_id.0.clone(), dbus_device.clone());
                
                let iface_ref = connection.object_server()
                    .interface::<_, DaemonInterface>(DAEMON_PATH).await?;
                
                DaemonInterface::device_paired(iface_ref.signal_emitter(), device_id.0, dbus_device).await?;
            }
            ConnectionEvent::Disconnected(device_id) => {
                info!("Event: Device disconnected - {}", device_id.0);
                
                devices.lock().await.remove(&device_id.0);
                
                let iface_ref = connection.object_server()
                    .interface::<_, DaemonInterface>(DAEMON_PATH).await?;
                
                DaemonInterface::device_disconnected(iface_ref.signal_emitter(), device_id.0).await?;
            }
            ConnectionEvent::SmsMessages(sms_data) => {
                info!("Event: SMS messages received");
                
                let messages_json = serde_json::to_string(&sms_data)?;
                
                let iface_ref = connection.object_server()
                    .interface::<_, SmsInterface>(SMS_PATH).await?;
                
                SmsInterface::sms_messages_received(iface_ref.signal_emitter(), messages_json).await?;
            }
            _ => {}
        }
        
        Ok(())
    }
}
