// kdeconnect-service/src/main.rs
//! KDE Connect D-Bus Service Daemon
//!
//! This service runs kdeconnect-core and exposes its functionality via D-Bus.
//! Multiple applications (applet, SMS app, etc.) can connect to this service.

use anyhow::Result;
use tracing::info;

mod dbus_interface;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    info!("=== KDE Connect Service Starting ===");

    // Initialize the adapter
    info!("Initializing KDE Connect adapter...");
    let adapter = kdeconnect_adapter::KdeConnectAdapter::new().await?;
    info!("✓ Adapter initialized");

    // Start D-Bus service
    info!("Starting D-Bus service...");
    let dbus_service = dbus_interface::KdeConnectService::new(adapter).await?;
    info!("✓ D-Bus service started on org.cosmic.KdeConnect");

    // Run the service
    info!("Service ready - listening for requests");
    dbus_service.run().await?;

    Ok(())
}
