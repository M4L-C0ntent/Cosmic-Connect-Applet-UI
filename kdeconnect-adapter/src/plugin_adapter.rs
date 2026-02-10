// kdeconnect-adapter/src/plugin_adapter.rs

use std::sync::Arc;
use tokio::sync::mpsc;
use kdeconnect_core::{
    event::AppEvent,
    device::DeviceId,
    ProtocolPacket, PacketType,
};
use serde_json::json;

/// Battery monitoring adapter
pub struct BatteryAdapter {
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
}

impl BatteryAdapter {
    pub(crate) fn new(event_sender: Arc<mpsc::UnboundedSender<AppEvent>>) -> Self {
        Self { event_sender }
    }
    
    pub async fn request_battery_status(&self, device_id: DeviceId) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::BatteryRequest,
            json!({"request": true})
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to send battery request: {}", e))?;
        Ok(())
    }
}

/// Notification management adapter
pub struct NotificationAdapter {
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
}

impl NotificationAdapter {
    pub(crate) fn new(event_sender: Arc<mpsc::UnboundedSender<AppEvent>>) -> Self {
        Self { event_sender }
    }
    
    pub async fn request_notifications(&self, device_id: DeviceId) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::NotificationRequest,
            json!({
                "request": true
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to request notifications: {}", e))?;
        Ok(())
    }
    
    pub async fn dismiss_notification(&self, device_id: DeviceId, notification_id: String) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::NotificationRequest,
            json!({
                "cancel": notification_id
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to dismiss notification: {}", e))?;
        Ok(())
    }
    
    pub async fn reply_to_notification(&self, device_id: DeviceId, notification_id: String, message: String) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::NotificationReply,
            json!({
                "requestReplyId": notification_id,
                "message": message
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to reply to notification: {}", e))?;
        Ok(())
    }
}

/// Media control adapter
pub struct MediaControlAdapter {
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
}

impl MediaControlAdapter {
    pub(crate) fn new(event_sender: Arc<mpsc::UnboundedSender<AppEvent>>) -> Self {
        Self { event_sender }
    }
    
    pub async fn play_pause(&self, device_id: DeviceId, player: String) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::MprisRequest,
            json!({
                "player": player,
                "action": "PlayPause"
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to send play/pause: {}", e))?;
        Ok(())
    }
    
    pub async fn next(&self, device_id: DeviceId, player: String) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::MprisRequest,
            json!({
                "player": player,
                "action": "Next"
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to send next: {}", e))?;
        Ok(())
    }
    
    pub async fn previous(&self, device_id: DeviceId, player: String) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::MprisRequest,
            json!({
                "player": player,
                "action": "Previous"
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to send previous: {}", e))?;
        Ok(())
    }
}

/// Clipboard synchronization adapter
pub struct ClipboardAdapter {
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
}

impl ClipboardAdapter {
    pub(crate) fn new(event_sender: Arc<mpsc::UnboundedSender<AppEvent>>) -> Self {
        Self { event_sender }
    }
    
    pub async fn send_clipboard(&self, device_id: DeviceId, content: String) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::Clipboard,
            json!({
                "content": content
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to send clipboard: {}", e))?;
        Ok(())
    }
}

/// SMS messaging adapter
pub struct SmsAdapter {
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
}

impl SmsAdapter {
    pub(crate) fn new(event_sender: Arc<mpsc::UnboundedSender<AppEvent>>) -> Self {
        Self { event_sender }
    }
    
    pub async fn send_sms(&self, device_id: DeviceId, phone_number: String, message: String) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::SmsRequest,
            json!({
                "sendSms": true,
                "phoneNumber": phone_number,
                "messageBody": message
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to send SMS: {}", e))?;
        Ok(())
    }
    
    pub async fn request_conversation(&self, device_id: DeviceId, thread_id: i64) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::SmsRequest,
            json!({
                "requestConversation": thread_id
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to request conversation: {}", e))?;
        Ok(())
    }
    
    pub async fn request_conversations(&self, device_id: DeviceId) -> anyhow::Result<()> {
        eprintln!("=== SMS Adapter: Requesting conversations ===");
        eprintln!("Device ID: {:?}", device_id);

        let packet = ProtocolPacket::new(
            PacketType::SmsRequestConversations,
            json!({})
        );

        eprintln!("Packet type: {:?}", packet.packet_type);
        eprintln!("Sending packet...");
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to request conversations: {}", e))?;
        Ok(())
    }
}

/// SFTP browsing adapter
pub struct SftpAdapter {
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
}

impl SftpAdapter {
    pub(crate) fn new(event_sender: Arc<mpsc::UnboundedSender<AppEvent>>) -> Self {
        Self { event_sender }
    }
    
    pub async fn start_browsing(&self, device_id: DeviceId) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::SftpRequest,
            json!({
                "startBrowsing": true
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to start browsing: {}", e))?;
        Ok(())
    }
    
    pub async fn mount(&self, device_id: DeviceId) -> anyhow::Result<()> {
        self.start_browsing(device_id).await
    }
}

/// Remote command execution adapter
pub struct CommandAdapter {
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
}

impl CommandAdapter {
    pub(crate) fn new(event_sender: Arc<mpsc::UnboundedSender<AppEvent>>) -> Self {
        Self { event_sender }
    }
    
    pub async fn execute_command(&self, device_id: DeviceId, command_key: String) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::RunCommandRequest,
            json!({
                "key": command_key
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?;
        Ok(())
    }
    
    pub async fn request_commands(&self, device_id: DeviceId) -> anyhow::Result<()> {
        self.request_command_list(device_id).await
    }
    
    pub async fn request_command_list(&self, device_id: DeviceId) -> anyhow::Result<()> {
        let packet = ProtocolPacket::new(
            PacketType::RunCommandRequest,
            json!({
                "requestCommandList": true
            })
        );
        
        self.event_sender
            .send(AppEvent::SendPacket(device_id, packet))
            .map_err(|e| anyhow::anyhow!("Failed to request command list: {}", e))?;
        Ok(())
    }
}