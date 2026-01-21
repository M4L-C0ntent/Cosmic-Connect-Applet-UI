// kdeconnect-adapter/src/lib.rs

//! Adapter layer between UI and kdeconnect-core
//! 
//! This module provides a clean interface for UI applications
//! to interact with the KDE Connect protocol implementation.

use std::sync::Arc;
use tokio::sync::mpsc;
use kdeconnect_core::{KdeConnectCore, event::{AppEvent, ConnectionEvent}};

pub use kdeconnect_core::{
    device::{Device, DeviceId, DeviceState, PairState},
    event::{AppEvent as UiEvent, ConnectionEvent as CoreEvent},
    ProtocolPacket, PacketType,
};

// Re-export serde_json for packet body creation
pub use serde_json::json;

// Plugin adapters module
pub mod plugin_adapter;

pub use plugin_adapter::{
    BatteryAdapter,
    NotificationAdapter,
    MediaControlAdapter,
    ClipboardAdapter,
    SmsAdapter,
    SftpAdapter,
    CommandAdapter,
};

/// Main adapter interface for UI applications
pub struct KdeConnectAdapter {
    #[allow(dead_code)]
    core_handle: tokio::task::JoinHandle<()>,
    event_sender: Arc<mpsc::UnboundedSender<AppEvent>>,
    event_receiver: mpsc::UnboundedReceiver<ConnectionEvent>,
}

impl KdeConnectAdapter {
    /// Initialize the adapter and start the core event loop
    pub async fn new() -> anyhow::Result<Self> {
        let (mut core, event_receiver) = KdeConnectCore::new().await?;
        let event_sender = core.take_events();
        
        // Spawn the core event loop
        let core_handle = tokio::spawn(async move {
            core.run_event_loop().await;
        });
        
        Ok(Self {
            core_handle,
            event_sender,
            event_receiver,
        })
    }
    
    /// Get a sender for sending commands to the core
    pub fn get_event_sender(&self) -> Arc<mpsc::UnboundedSender<AppEvent>> {
        self.event_sender.clone()
    }
    
    /// Get the next event from the core (blocking)
    pub async fn next_event(&mut self) -> Option<ConnectionEvent> {
        self.event_receiver.recv().await
    }
    
    /// Broadcast identity to discover devices on the network
    pub async fn broadcast(&self) -> anyhow::Result<()> {
        self.event_sender
            .send(AppEvent::Broadcasting)
            .map_err(|e| anyhow::anyhow!("Failed to send broadcast event: {}", e))?;
        Ok(())
    }
    
    /// Request to pair with a device
    pub async fn pair_device(&self, device_id: DeviceId) -> anyhow::Result<()> {
        self.event_sender
            .send(AppEvent::Pair(device_id))
            .map_err(|e| anyhow::anyhow!("Failed to send pair event: {}", e))?;
        Ok(())
    }
    
    /// Unpair from a device
    pub async fn unpair_device(&self, device_id: DeviceId) -> anyhow::Result<()> {
        self.event_sender
            .send(AppEvent::Unpair(device_id))
            .map_err(|e| anyhow::anyhow!("Failed to send unpair event: {}", e))?;
        Ok(())
    }
    
    /// Send a ping to a device
    pub async fn send_ping(&self, device_id: DeviceId, message: String) -> anyhow::Result<()> {
        self.event_sender
            .send(AppEvent::Ping((device_id, message)))
            .map_err(|e| anyhow::anyhow!("Failed to send ping event: {}", e))?;
        Ok(())
    }
    
    /// Send files to a device
    pub async fn send_files(&self, device_id: DeviceId, files: Vec<String>) -> anyhow::Result<()> {
        self.event_sender
            .send(AppEvent::SendFiles((device_id, files)))
            .map_err(|e| anyhow::anyhow!("Failed to send files event: {}", e))?;
        Ok(())
    }
    
    // Plugin adapter creation methods
    
    /// Create a battery monitoring adapter
    pub fn battery(&self) -> BatteryAdapter {
        BatteryAdapter::new(self.event_sender.clone())
    }
    
    /// Create a notification adapter
    pub fn notifications(&self) -> NotificationAdapter {
        NotificationAdapter::new(self.event_sender.clone())
    }
    
    /// Create a media control adapter
    pub fn media_control(&self) -> MediaControlAdapter {
        MediaControlAdapter::new(self.event_sender.clone())
    }
    
    /// Create a clipboard adapter
    pub fn clipboard(&self) -> ClipboardAdapter {
        ClipboardAdapter::new(self.event_sender.clone())
    }
    
    /// Create an SMS adapter
    pub fn sms(&self) -> SmsAdapter {
        SmsAdapter::new(self.event_sender.clone())
    }
    
    /// Create an SFTP adapter
    pub fn sftp(&self) -> SftpAdapter {
        SftpAdapter::new(self.event_sender.clone())
    }
    
    /// Create a command adapter
    pub fn commands(&self) -> CommandAdapter {
        CommandAdapter::new(self.event_sender.clone())
    }
}