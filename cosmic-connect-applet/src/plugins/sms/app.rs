// cosmic-connect-applet/src/plugins/sms/app.rs
use cosmic::{
    app::Core,
    iced::{Length, Subscription},
    widget, Application, ApplicationExt, Element, Task, Action,
};
use futures::StreamExt;
use std::collections::HashMap;

use super::dbus;
use super::models::{Conversation, Message, ProtocolEvent};
use super::utils;
use super::views;

pub fn run(device_id: String, device_name: String) -> cosmic::iced::Result {
    cosmic::app::run::<SmsWindow>(
        cosmic::app::Settings::default(),
        (device_id, device_name),
    )
}

#[derive(Clone, Debug)]
pub enum SmsMessage {
    LoadConversations,
    ConversationsLoaded(Vec<Conversation>),
    ContactsLoaded(HashMap<String, String>),
    SelectThread(String),
    UpdateInput(String),
    UpdateSearch(String),
    SendMessage,
    RefreshThread,
    CloseWindow,
    ProtocolEventReceived(ProtocolEvent),
    OpenNewChatDialog,
    CloseNewChatDialog,
    UpdateNewChatPhone(String),
    SelectContactForNewChat(String, String),
    CreateNewChat,
}

pub struct SmsWindow {
    core: Core,
    pub device_id: String,
    pub device_name: String,
    pub conversations: Vec<Conversation>,
    pub contacts: HashMap<String, String>,
    pub selected_thread: Option<String>,
    pub messages: Vec<Message>,
    pub message_input: String,
    pub search_query: String,
    pub show_new_chat_dialog: bool,
    pub new_chat_phone_input: String,
}

impl Application for SmsWindow {
    type Executor = cosmic::executor::Default;
    type Flags = (String, String);
    type Message = SmsMessage;
    const APP_ID: &'static str = "com.system76.CosmicConnectSms";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let (device_id, device_name) = flags;
        
        let mut app = Self {
            core,
            device_id: device_id.clone(),
            device_name: device_name.clone(),
            conversations: Vec::new(),
            contacts: HashMap::new(),
            selected_thread: None,
            messages: Vec::new(),
            message_input: String::new(),
            search_query: String::new(),
            show_new_chat_dialog: false,
            new_chat_phone_input: String::new(),
        };

        let title = format!("SMS - {}", device_name);
        let title_task = app.set_window_title(title, app.core.main_window_id().unwrap());

        let device_id_init = device_id.clone();
        
        (
            app,
            Task::batch(vec![
                title_task,
                // Initialize D-Bus client and request conversations
                cosmic::task::future(async move {
                    if let Err(e) = dbus::initialize().await {
                        eprintln!("Failed to initialize SMS D-Bus client: {:?}", e);
                    }
                    
                    // Request conversations
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    let convs = dbus::fetch_conversations(device_id_init.clone()).await;
                    Action::App(SmsMessage::ConversationsLoaded(convs))
                }),
                // Fetch contacts
                cosmic::task::future(async move {
                    dbus::fetch_contacts(device_id).await;
                    Action::App(SmsMessage::ContactsLoaded(HashMap::new()))
                }),
            ]),
        )
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        // Subscribe to SMS D-Bus events
        let device_id = self.device_id.clone();
        
        Subscription::run_with_id(
            "sms-events",
            dbus::listen_for_sms_events_stream(device_id)
                .map(SmsMessage::ProtocolEventReceived)
        )
    }

    fn update(&mut self, message: Self::Message) -> Task<Action<Self::Message>> {
        match message {
            SmsMessage::LoadConversations => {
                let device_id = self.device_id.clone();
                return cosmic::task::future(async move {
                    let convs = dbus::fetch_conversations(device_id).await;
                    Action::App(SmsMessage::ConversationsLoaded(convs))
                });
            }
            SmsMessage::ConversationsLoaded(conversations) => {
                eprintln!("📥 Loaded {} conversations", conversations.len());
                self.conversations = conversations;
                self.update_conversation_names();
            }
            SmsMessage::ContactsLoaded(contacts) => {
                eprintln!("📇 Loaded {} contacts", contacts.len());
                self.contacts = contacts;
                self.update_conversation_names();
            }
            SmsMessage::SelectThread(thread_id) => {
                eprintln!("📱 Selected thread: {}", thread_id);
                self.selected_thread = Some(thread_id.clone());
                self.messages.clear();
                
                let device_id = self.device_id.clone();
                return cosmic::task::future(async move {
                    dbus::request_conversation_messages(device_id, thread_id).await;
                    Action::App(SmsMessage::RefreshThread)
                });
            }
            SmsMessage::UpdateInput(input) => {
                self.message_input = input;
            }
            SmsMessage::UpdateSearch(query) => {
                self.search_query = query;
            }
            SmsMessage::SendMessage => {
                if self.message_input.trim().is_empty() {
                    return Task::none();
                }

                let Some(thread_id) = &self.selected_thread else {
                    return Task::none();
                };

                let Some(conv) = self.conversations.iter().find(|c| c.thread_id == *thread_id) else {
                    return Task::none();
                };

                let device_id = self.device_id.clone();
                let phone = conv.phone_number.clone();
                let message = self.message_input.clone();
                
                // Create optimistic message
                let now = utils::now_millis();
                let optimistic_msg = Message {
                    id: format!("sending_{}", now),
                    thread_id: thread_id.clone(),
                    body: message.clone(),
                    address: phone.clone(),
                    date: now,
                    type_: 2,
                    read: true,
                };
                
                self.messages.push(optimistic_msg);
                self.messages.sort_by_key(|m| m.date);
                self.message_input.clear();
                
                return cosmic::task::future(async move {
                    dbus::send_sms(device_id, phone, message).await;
                    Action::App(SmsMessage::RefreshThread)
                });
            }
            SmsMessage::RefreshThread => {
                // No-op, messages arrive via subscription
            }
            SmsMessage::ProtocolEventReceived(event) => {
                eprintln!("📨 Protocol event received: {:?}", event);
                self.handle_protocol_event(event);
            }
            SmsMessage::OpenNewChatDialog => {
                self.show_new_chat_dialog = true;
            }
            SmsMessage::CloseNewChatDialog => {
                self.show_new_chat_dialog = false;
                self.new_chat_phone_input.clear();
            }
            SmsMessage::UpdateNewChatPhone(phone) => {
                self.new_chat_phone_input = phone;
            }
            SmsMessage::SelectContactForNewChat(phone, _name) => {
                self.new_chat_phone_input = phone;
            }
            SmsMessage::CreateNewChat => {
                // Create new conversation
                let phone = self.new_chat_phone_input.trim().to_string();
                if !phone.is_empty() {
                    let thread_id = format!("new_{}", utils::now_millis());
                    let conv = Conversation {
                        thread_id: thread_id.clone(),
                        phone_number: phone,
                        contact_name: String::new(),
                        last_message: String::new(),
                        timestamp: utils::now_millis(),
                        unread: false,
                    };
                    self.conversations.insert(0, conv);
                    self.show_new_chat_dialog = false;
                    self.new_chat_phone_input.clear();
                    return cosmic::task::message(Action::App(SmsMessage::SelectThread(thread_id)));
                }
            }
            SmsMessage::CloseWindow => {
                std::process::exit(0);
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let content = if self.show_new_chat_dialog {
            views::view_new_chat_dialog(self)
        } else {
            views::view_main(self)
        };

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl SmsWindow {
    fn handle_protocol_event(&mut self, event: ProtocolEvent) {
        match event {
            ProtocolEvent::ConversationsReceived(conversations) => {
                eprintln!("✓ Received {} conversations via D-Bus", conversations.len());
                self.conversations = conversations;
                self.update_conversation_names();
            }
            ProtocolEvent::MessageReceived(message) => {
                eprintln!("✓ Received message for thread {}", message.thread_id);
                
                // Add to messages if this thread is selected
                if let Some(selected) = &self.selected_thread {
                    if *selected == message.thread_id {
                        // Check if message already exists
                        if !self.messages.iter().any(|m| m.id == message.id) {
                            self.messages.push(message.clone());
                            self.messages.sort_by_key(|m| m.date);
                        }
                    }
                }
                
                // Update conversation
                if let Some(conv) = self.conversations.iter_mut()
                    .find(|c| c.thread_id == message.thread_id) 
                {
                    conv.last_message = message.body;
                    conv.timestamp = message.date;
                }
                
                // Sort conversations by timestamp
                self.conversations.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            }
            ProtocolEvent::Error(err) => {
                eprintln!("✗ SMS Protocol Error: {}", err);
            }
        }
    }

    fn update_conversation_names(&mut self) {
        for conv in &mut self.conversations {
            if let Some(name) = self.contacts.get(&conv.phone_number) {
                conv.contact_name = name.clone();
            }
        }
    }
}
