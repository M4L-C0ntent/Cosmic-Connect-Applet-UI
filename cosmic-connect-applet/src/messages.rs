// cosmic-connect-applet/src/messages.rs
// #[allow(dead_code)] = Placeholder for code that will be used once features are fully integrated

use crate::models::Device;

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    RefreshDevices,
    DevicesUpdated(Vec<Device>),
    ToggleDeviceMenu(String),
    
    // Device actions
    PingDevice(String),
    #[allow(dead_code)]
    PairDevice(String),
    #[allow(dead_code)]
    UnpairDevice(String),
    RingDevice(String),
    BrowseDevice(String),
    SendFile(String),
    SendSMS(String),
    ShareClipboard(String),
    #[allow(dead_code)]
    ShareUrl(String, String),
    
    // Advanced features (not yet implemented)
    #[allow(dead_code)]
    RemoteInput(String),
    LockDevice(String),
    OpenSettings,
    UseAsMonitor(String),
    #[allow(dead_code)]
    PresenterMode(String),
    
    // Pairing
    AcceptPairing(String),
    RejectPairing(String),
    PairingRequestReceived(String, String, String), // device_id, device_name, device_type
}