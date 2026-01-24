// cosmic-connect-applet/src/notifications.rs
// #[allow(dead_code)] = Placeholder for code that will be used once features are fully integrated

use std::error::Error as StdError;

/// Pairing notification data sent to the UI
#[derive(Debug, Clone)]
pub struct PairingNotification {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
}

/// Start listening for pairing notifications
/// This replaces the old D-Bus signal listener with backend event monitoring
pub fn start_notification_listener(
    tx: tokio::sync::mpsc::Sender<PairingNotification>,
    _use_polling: bool,
) {
    tokio::spawn(async move {
        eprintln!("=== Starting Pairing Notification Listener ===");
        
        loop {
            // Listen for events from the backend
            if let Some(event) = crate::backend::next_event().await {
                use kdeconnect_adapter::CoreEvent;
                
                match event {
                    CoreEvent::PairStateChanged((device_id, pair_state)) => {
                        eprintln!("=== Pairing State Changed ===");
                        eprintln!("Device ID: {}", device_id.0);
                        eprintln!("Pair State: {:?}", pair_state);
                        
                        // Check if this is a pairing request from peer
                        // PairState variants: NotPaired, Requested, Paired
                        // When device requests pairing from us, it shows as Requested
                        use kdeconnect_adapter::PairState;
                        if matches!(pair_state, PairState::Requested) {
                            // Get device info from backend cache
                            let devices = crate::backend::fetch_devices().await;
                            if let Some(device) = devices.iter().find(|d| d.id == device_id.0) {
                                eprintln!("Pairing request from: {} ({})", device.name, device.device_type);
                                
                                let notification = PairingNotification {
                                    device_id: device_id.0.clone(),
                                    device_name: device.name.clone(),
                                    device_type: device.device_type.clone(),
                                };
                                
                                // Send notification event to UI
                                if let Err(e) = tx.send(notification).await {
                                    eprintln!("Failed to send pairing notification: {}", e);
                                }
                            }
                        }
                    }
                    _ => {
                        // Ignore other events
                    }
                }
            }
        }
    });
}

/// Show a desktop notification for pairing request
#[allow(dead_code)]
pub async fn show_pairing_notification(
    device_name: &str,
    device_id: &str,
) -> Result<(), Box<dyn StdError + Send + Sync>> {
    eprintln!("=== Showing Pairing Notification ===");
    eprintln!("Device: {} ({})", device_name, device_id);
    
    let summary = format!("{} wants to pair", device_name);
    let body = "Open COSMIC Connect to accept or reject";
    
    // Use notify-send command for simple notifications
    tokio::spawn(async move {
        let result = tokio::process::Command::new("notify-send")
            .arg("--app-name=COSMIC KDE Connect")
            .arg("--icon=phone")
            .arg("--urgency=critical")
            .arg("--category=device.added")
            .arg(&summary)
            .arg(&body)
            .spawn();
        
        match result {
            Ok(mut child) => {
                match child.wait().await {
                    Ok(status) => {
                        if status.success() {
                            eprintln!("✓ Notification sent successfully");
                        } else {
                            eprintln!("✗ notify-send failed with status: {}", status);
                        }
                    }
                    Err(e) => eprintln!("✗ Failed to wait for notify-send: {}", e),
                }
            }
            Err(e) => {
                eprintln!("✗ Failed to spawn notify-send: {}", e);
                eprintln!("  Is notify-send installed? (package: libnotify)");
            }
        }
    });
    
    Ok(())
}