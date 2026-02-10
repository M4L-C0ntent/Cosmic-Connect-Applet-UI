// cosmic-connect-applet/src/plugins/sms/dbus.rs
//! D-Bus interface for SMS plugin using kdeconnect-dbus-client

use anyhow::Result;
use kdeconnect_dbus_client::{KdeConnectClient, ServiceEvent};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::models::{Conversation, Message, ProtocolEvent};

lazy_static::lazy_static! {
    static ref SMS_CLIENT: Arc<Mutex<Option<Arc<KdeConnectClient>>>> = Arc::new(Mutex::new(None));
}

/// Initialize SMS D-Bus client
pub async fn initialize() -> Result<()> {
    eprintln!("=== Initializing SMS D-Bus Client ===");
    
    let client = KdeConnectClient::new().await?;
    
    let mut client_guard = SMS_CLIENT.lock().await;
    *client_guard = Some(Arc::new(client));
    
    eprintln!("✓ SMS D-Bus client connected");
    Ok(())
}

/// Fetch conversations from device
pub async fn fetch_conversations(device_id: String) -> Vec<Conversation> {
    eprintln!("=== Fetching Conversations via D-Bus ===");
    eprintln!("Device: {}", device_id);
    
    let client_guard = SMS_CLIENT.lock().await;
    
    let Some(client) = client_guard.as_ref() else {
        eprintln!("⚠️  SMS D-Bus client not initialized");
        return vec![];
    };
    
    // Request conversations from device
    if let Err(e) = client.request_conversations(&device_id).await {
        eprintln!("✗ Failed to request conversations: {:?}", e);
        return vec![];
    }
    
    eprintln!("✓ Conversation request sent, waiting for response...");
    
    // Note: Actual conversations will come via D-Bus signal
    // Return empty for now, they'll be populated when signal arrives
    vec![]
}

/// Fetch contacts from device
pub async fn fetch_contacts(device_id: String) -> std::collections::HashMap<String, String> {
    eprintln!("=== Fetching Contacts via D-Bus ===");
    eprintln!("Device: {}", device_id);
    eprintln!("⚠️  Contact sync not yet implemented");
    
    // TODO: Implement contacts via D-Bus
    std::collections::HashMap::new()
}

/// Request specific conversation messages
pub async fn request_conversation_messages(device_id: String, thread_id: String) {
    eprintln!("=== Requesting Conversation Messages ===");
    eprintln!("Device: {}, Thread: {}", device_id, thread_id);
    
    let client_guard = SMS_CLIENT.lock().await;
    
    let Some(client) = client_guard.as_ref() else {
        eprintln!("⚠️  SMS D-Bus client not initialized");
        return;
    };
    
    // Convert String thread_id to i64
    let thread_id_i64 = thread_id.parse::<i64>().unwrap_or(0);
    
    if let Err(e) = client.request_conversation(&device_id, thread_id_i64).await {
        eprintln!("✗ Failed to request conversation: {:?}", e);
    } else {
        eprintln!("✓ Conversation messages request sent");
    }
}

/// Send SMS message
pub async fn send_sms(device_id: String, phone_number: String, message: String) {
    eprintln!("=== Sending SMS via D-Bus ===");
    eprintln!("Device: {}", device_id);
    eprintln!("To: {}", phone_number);
    
    let client_guard = SMS_CLIENT.lock().await;
    
    let Some(client) = client_guard.as_ref() else {
        eprintln!("⚠️  SMS D-Bus client not initialized");
        return;
    };
    
    if let Err(e) = client.send_sms(&device_id, &phone_number, &message).await {
        eprintln!("✗ Failed to send SMS: {:?}", e);
    } else {
        eprintln!("✓ SMS sent successfully");
    }
}

/// Listen for SMS events from D-Bus
pub fn listen_for_sms_events_stream(device_id: String) -> impl futures::Stream<Item = ProtocolEvent> {
    use tokio::sync::mpsc;
    
    let (tx, rx) = mpsc::channel::<ProtocolEvent>(100);
    
    tokio::spawn(async move {
        eprintln!("📊 Starting SMS D-Bus event listener for device: {}", device_id);
        
        let client = {
            let client_guard = SMS_CLIENT.lock().await;
            client_guard.clone()
        };
        
        let Some(client) = client else {
            eprintln!("⚠️  SMS D-Bus client not initialized");
            return;
        };
        
        let mut stream = client.listen_for_events().await;
        
        while let Some(event) = stream.next().await {
            match event {
                ServiceEvent::SmsMessagesReceived(messages_json) => {
                    eprintln!("📨 Received SMS messages via D-Bus");
                    
                    // Parse JSON into SMS data
                    if let Ok(sms_data) = serde_json::from_str::<kdeconnect_core::plugins::sms::SmsMessages>(&messages_json) {
                        eprintln!("✓ Parsed {} messages", sms_data.messages.len());
                        
                        // Convert to UI messages
                        let messages: Vec<Message> = sms_data.messages.iter().map(|msg| {
                            // Get first address string from SmsAddress objects
                            let address = msg.addresses.first()
                                .map(|addr| addr.address.clone())
                                .unwrap_or_default();
                            
                            Message {
                                id: msg.id.to_string(),
                                thread_id: msg.thread_id.to_string(),
                                address,
                                body: msg.body.clone(),
                                date: msg.date,
                                type_: msg.message_type,
                                read: msg.read == 1,
                            }
                        }).collect();
                        
                        // Send individual message events
                        for message in messages.iter() {
                            let event = ProtocolEvent::MessageReceived(message.clone());
                            if tx.send(event).await.is_err() {
                                eprintln!("Event receiver dropped");
                                return;
                            }
                        }
                        
                        // Group into conversations
                        let conversations = messages_to_conversations(&messages);
                        let event = ProtocolEvent::ConversationsReceived(conversations);
                        if tx.send(event).await.is_err() {
                            eprintln!("Event receiver dropped");
                            return;
                        }
                    } else {
                        eprintln!("✗ Failed to parse SMS messages JSON");
                    }
                }
                _ => {
                    // Ignore other events
                }
            }
        }
        
        eprintln!("SMS D-Bus event listener ended");
    });
    
    futures::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|event| (event, rx))
    })
}

/// Convert messages to conversations
fn messages_to_conversations(messages: &[Message]) -> Vec<Conversation> {
    use std::collections::HashMap;
    
    let mut conversations: HashMap<String, Vec<Message>> = HashMap::new();
    
    // Group messages by thread_id
    for message in messages {
        conversations
            .entry(message.thread_id.clone())
            .or_insert_with(Vec::new)
            .push(message.clone());
    }
    
    // Convert to Conversation structs
    conversations
        .into_iter()
        .map(|(thread_id, mut msgs)| {
            // Sort by date (newest first)
            msgs.sort_by(|a, b| b.date.cmp(&a.date));
            
            let last_message = msgs.first()
                .map(|m| m.body.clone())
                .unwrap_or_default();
            
            let timestamp = msgs.first()
                .map(|m| m.date)
                .unwrap_or(0);
            
            let phone_number = msgs.first()
                .map(|m| m.address.clone())
                .unwrap_or_default();
            
            let unread = msgs.iter().any(|m| !m.read);
            
            Conversation {
                thread_id,
                phone_number,
                last_message,
                timestamp,
                unread,
                contact_name: String::new(),
            }
        })
        .collect()
}
