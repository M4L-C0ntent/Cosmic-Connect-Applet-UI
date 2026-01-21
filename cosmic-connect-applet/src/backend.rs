// cosmic-connect-applet/src/backend.rs
//! Backend module replacing D-Bus with native KDE Connect adapter.
//!
//! This module provides the same interface as the old dbus.rs but uses
//! the kdeconnect-adapter instead of D-Bus for communication.
// #[allow(dead_code)] = Placeholder for code that will be used once features are fully integrated

use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use anyhow::Result;

use kdeconnect_adapter::{
    KdeConnectAdapter,
    CoreEvent,
    Device,
    DeviceId,
    PairState,
};

use crate::models::Device as UiDevice;

lazy_static::lazy_static! {
    static ref ADAPTER: Arc<Mutex<Option<KdeConnectAdapter>>> = Arc::new(Mutex::new(None));
    static ref DEVICES: Arc<Mutex<HashMap<String, UiDevice>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Initialize the backend
pub async fn initialize() -> Result<()> {
    let mut adapter_guard = ADAPTER.lock().await;
    
    if adapter_guard.is_none() {
        let adapter = KdeConnectAdapter::new().await?;
        *adapter_guard = Some(adapter);
        
        // Start discovery
        if let Some(ref adapter) = *adapter_guard {
            adapter.broadcast().await?;
        }
    }
    
    Ok(())
}

/// Get next event from adapter
pub async fn next_event() -> Option<CoreEvent> {
    let mut adapter_guard = ADAPTER.lock().await;
    if let Some(ref mut adapter) = *adapter_guard {
        adapter.next_event().await
    } else {
        None
    }
}

/// Fetch all devices
pub async fn fetch_devices() -> Vec<UiDevice> {
    let devices = DEVICES.lock().await;
    devices.values().cloned().collect()
}

/// Update device in cache
pub async fn update_device(device_id: String, device: UiDevice) {
    let mut devices = DEVICES.lock().await;
    devices.insert(device_id, device);
}

/// Remove device from cache
pub async fn remove_device(device_id: &str) {
    let mut devices = DEVICES.lock().await;
    devices.remove(device_id);
}

/// Pair with a device
pub async fn pair_device(device_id: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.pair_device(id).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Unpair from a device
pub async fn unpair_device(device_id: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.unpair_device(id).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Send ping to device
pub async fn ping_device(device_id: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.send_ping(id, "Hello from COSMIC!".to_string()).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Send files to device
pub async fn send_files(device_id: String, files: Vec<String>) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.send_files(id, files).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Request battery status
#[allow(dead_code)]
pub async fn request_battery(device_id: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.battery().request_battery_status(id).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

// Media player functions removed - COSMIC desktop handles media controls natively via MPRIS

/// Request notifications
#[allow(dead_code)]
pub async fn request_notifications(device_id: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.notifications().request_notifications(id).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Dismiss notification
#[allow(dead_code)]
pub async fn dismiss_notification(device_id: String, notification_id: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.notifications().dismiss_notification(id, notification_id).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Reply to notification
#[allow(dead_code)]
pub async fn reply_notification(device_id: String, notification_id: String, message: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.notifications().reply_to_notification(id, notification_id, message).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Send clipboard content
pub async fn send_clipboard(device_id: String, content: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.clipboard().send_clipboard(id, content).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Request SMS conversations
#[allow(dead_code)]
pub async fn request_conversations(device_id: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.sms().request_conversations(id).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Request specific conversation
pub async fn request_conversation(device_id: String, thread_id: i64) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.sms().request_conversation(id, thread_id).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Send SMS
#[allow(dead_code)]
pub async fn send_sms(device_id: String, phone_number: String, message: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.sms().send_sms(id, phone_number, message).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Start SFTP browsing
pub async fn start_sftp(device_id: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.sftp().start_browsing(id).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Request remote commands
#[allow(dead_code)]
pub async fn request_commands(device_id: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.commands().request_commands(id).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Execute remote command
#[allow(dead_code)]
pub async fn execute_command(device_id: String, command_key: String) -> Result<()> {
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        adapter.commands().execute_command(id, command_key).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Ring device (find my phone)
pub async fn ring_device(device_id: String) -> Result<()> {
    // Find my phone functionality
    let adapter_guard = ADAPTER.lock().await;
    if let Some(ref adapter) = *adapter_guard {
        let id = DeviceId(device_id);
        // Ring is typically done through ping or a specific plugin
        adapter.send_ping(id, "Ring!".to_string()).await
    } else {
        Err(anyhow::anyhow!("Adapter not initialized"))
    }
}

/// Browse device filesystem via SFTP
pub async fn browse_device_filesystem(device_id: String) -> Result<()> {
    start_sftp(device_id).await
}

/// Share files (alias for send_files)
pub async fn share_files(device_id: String, files: Vec<String>) -> Result<()> {
    send_files(device_id, files).await
}

impl From<Device> for UiDevice {
    fn from(device: Device) -> Self {
        UiDevice {
            id: device.device_id.0.clone(),
            name: device.name,
            device_type: "phone".to_string(), // Default - can be updated from Identity packet
            is_paired: device.pair_state == PairState::Paired,
            is_reachable: true, // Connected if we have the device
            battery_level: None,
            is_charging: Some(false),
            has_battery: false,
            has_ping: true,
            has_share: true,
            has_findmyphone: false,
            has_sms: false,
            has_clipboard: true,
            has_contacts: false,
            has_mpris: false,
            has_remote_keyboard: false,
            has_sftp: false,
            has_presenter: false,
            has_lockdevice: false,
            has_virtualmonitor: false,
            pairing_requests: 0,
            signal_strength: None,
            network_type: None,
        }
    }
}