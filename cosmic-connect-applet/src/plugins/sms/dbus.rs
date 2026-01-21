// cosmic-connect-applet/src/plugins/sms/dbus.rs
//! SMS operations using the native KDE Connect protocol adapter.
//!
//! This module replaces the old D-Bus implementation with direct protocol communication
//! through the kdeconnect-adapter backend.
 
// #[allow(dead_code)] = Placeholder for code that will be used once features are fully integrated

use std::collections::HashMap;

use crate::backend;
use super::models::{ContactsMap, Conversation, Message};
use super::messages::SmsMessage;
use super::utils::now_millis;

/// Fetches all conversations from the device.
/// 
/// This triggers a request to the device via the adapter.
/// Conversations will arrive via ConnectionEvent packets.
#[allow(dead_code)]
pub async fn fetch_conversations(device_id: String) -> Vec<Conversation> {
    eprintln!("=== Requesting Conversations via Adapter ===");
    eprintln!("Device: {}", device_id);
    
    // Request conversations through the adapter
    match backend::request_conversations(device_id.clone()).await {
        Ok(_) => {
            eprintln!("✓ Conversation request sent");
            eprintln!("  Conversations will arrive via protocol events");
        }
        Err(e) => {
            eprintln!("✗ Failed to request conversations: {:?}", e);
        }
    }
    
    // Return placeholder - actual conversations will come through events
    vec![Conversation {
        thread_id: "info_1".to_string(),
        contact_name: "📱 KDE Connect SMS".to_string(),
        phone_number: "System".to_string(),
        last_message: "Waiting for conversations from device...".to_string(),
        timestamp: now_millis(),
        unread: false,
    }]
}

/// Requests messages for a specific conversation thread.
#[allow(dead_code)]
pub async fn request_conversation_messages(device_id: String, thread_id: String) {
    eprintln!("=== Requesting Messages via Adapter ===");
    eprintln!("Device: {}, Thread: {}", device_id, thread_id);
    
    let thread_id_i64 = match thread_id.parse::<i64>() {
        Ok(id) => id,
        Err(_) => {
            eprintln!("✗ Invalid thread ID");
            return;
        }
    };
    
    match backend::request_conversation(device_id, thread_id_i64).await {
        Ok(_) => {
            eprintln!("✓ Requested messages for thread {}", thread_id_i64);
            eprintln!("  Messages will arrive via protocol events");
        }
        Err(e) => {
            eprintln!("✗ Failed to request conversation: {:?}", e);
        }
    }
}

/// Sends an SMS message to the specified phone number.
#[allow(dead_code)]
pub async fn send_sms(device_id: String, phone_number: String, message: String) {
    let log = format!(
        "=== Sending SMS via Adapter ===\nDevice: {}\nTo: {}\nMessage: {}\n",
        device_id, phone_number, message
    );
    
    eprintln!("{}", log);
    
    match backend::send_sms(device_id, phone_number.clone(), message.clone()).await {
        Ok(_) => {
            eprintln!("✓ SMS sent successfully");
        }
        Err(e) => {
            eprintln!("✗ Failed to send SMS: {:?}", e);
        }
    }
}

/// Fetches contacts from the device.
/// 
/// Note: This is a placeholder. The actual implementation depends on
/// the contacts plugin being implemented in the protocol adapter.
#[allow(dead_code)]
pub async fn fetch_contacts(device_id: String) -> ContactsMap {
    eprintln!("=== Requesting Contacts ===");
    eprintln!("Device: {}", device_id);
    eprintln!("⚠️  Contact sync not yet implemented in adapter");
    
    HashMap::new()
}

/// Creates a stream that listens for SMS-related protocol events.
/// 
/// This replaces the old D-Bus signal listener with a protocol event listener.
#[allow(dead_code)]
pub fn listen_for_sms_events_stream(
    device_id: String,
) -> impl futures::Stream<Item = SmsMessage> {
    let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
    
    tokio::spawn(async move {
        eprintln!("📊 Starting protocol event listener for device: {}", device_id);
        
        loop {
            // Listen for events from the backend
            if let Some(event) = backend::next_event().await {
                use kdeconnect_adapter::CoreEvent;
                
                match event {
                    CoreEvent::ClipboardReceived(_) => {
                        // Not relevant for SMS
                        continue;
                    }
                    CoreEvent::Connected(_) => {
                        eprintln!("📱 Device connected");
                        continue;
                    }
                    CoreEvent::Disconnected(_) => {
                        eprintln!("📱 Device disconnected");
                        break;
                    }
                    CoreEvent::DevicePaired(_) => {
                        eprintln!("📱 Device paired");
                        continue;
                    }
                    CoreEvent::StateUpdated(state) => {
                        // Handle state updates (battery, connectivity, etc.)
                        eprintln!("📱 Device state updated: {:?}", state);
                        continue;
                    }
                    CoreEvent::PairStateChanged(_) => {
                        eprintln!("📱 Pair state changed");
                        continue;
                    }
                    CoreEvent::Mpris(_) => {
                        // Not relevant for SMS
                        continue;
                    }
                }
            } else {
                // Channel closed
                eprintln!("⚠️  Backend event channel closed");
                break;
            }
        }
        
        eprintln!("Protocol event listener task ended");
    });
    
    tokio_stream::wrappers::UnboundedReceiverStream::new(rx)
}

/// Parse SMS messages from protocol packet body.
/// 
/// This will be called when PacketReceived events arrive with SMS packet types.
#[allow(dead_code)]
pub fn parse_sms_packet(packet_body: serde_json::Value) -> Option<Message> {
    // Parse the JSON packet body based on KDE Connect SMS protocol
    // Format: { "event": "sms", "messageBody": "...", "phoneNumber": "...", ... }
    
    let event = packet_body.get("event")?.as_str()?;
    if event != "sms" {
        return None;
    }
    
    let message_body = packet_body.get("messageBody")?.as_str()?.to_string();
    let phone_number = packet_body.get("phoneNumber")?.as_str()?.to_string();
    let timestamp = packet_body.get("date")?.as_i64().unwrap_or_else(now_millis);
    let thread_id = packet_body.get("threadId")
        .and_then(|v| v.as_i64())
        .map(|id| id.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let message_type = packet_body.get("type")
        .and_then(|v| v.as_i64())
        .unwrap_or(1) as i32;
    
    let message_id = format!("{}_{}", thread_id, timestamp);
    
    Some(Message {
        id: message_id,
        thread_id,
        body: message_body,
        address: phone_number,
        date: timestamp,
        type_: message_type,
        read: true,
    })
}

/// Parse conversation list from protocol packet body.
#[allow(dead_code)]
pub fn parse_conversations_packet(packet_body: serde_json::Value) -> Vec<Conversation> {
    let mut conversations = Vec::new();
    
    if let Some(convs_array) = packet_body.get("conversations").and_then(|v| v.as_array()) {
        for conv_value in convs_array {
            if let Some(conv) = parse_conversation_from_json(conv_value) {
                conversations.push(conv);
            }
        }
    }
    
    conversations
}

#[allow(dead_code)]
fn parse_conversation_from_json(value: &serde_json::Value) -> Option<Conversation> {
    let thread_id = value.get("threadId")?.as_i64()?.to_string();
    let phone_number = value.get("address")?.as_str()?.to_string();
    let last_message = value.get("snippet")?.as_str()?.to_string();
    let timestamp = value.get("date")?.as_i64()?;
    
    Some(Conversation {
        thread_id,
        contact_name: phone_number.clone(),
        phone_number,
        last_message,
        timestamp,
        unread: false,
    })
}