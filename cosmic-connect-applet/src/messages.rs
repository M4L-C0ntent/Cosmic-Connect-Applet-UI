// cosmic-connect-applet/src/messages.rs
#[derive(Debug, Clone)]
pub enum Message {
    // Popup control
    TogglePopup,
    
    // Device management
    RefreshDevices,
    DevicesUpdated(Vec<crate::models::Device>),
    PairDevice(String),
    UnpairDevice(String),
    AcceptPairing(String),
    RejectPairing(String),
    
    // Device menu toggle
    ToggleDeviceMenu(String),
    
    // Communication
    PingDevice(String),
    SendSMS(String),
    ShareClipboard(String),
    
    // File operations
    SendFile(String),
    ShareUrl(String, String),
    BrowseDevice(String),
    
    // Remote control
    RemoteInput(String),
    RingDevice(String),
    LockDevice(String),
    
    // Advanced features
    PresenterMode(String),
    UseAsMonitor(String),
    
    // Settings
    OpenSettings,
    
    // Pairing notifications
    PairingRequestReceived(String, String, String), // device_id, device_name, device_type
}