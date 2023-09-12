use std::process::Command;


/// Request that squid reload its configuration.
/// This requires an entry in /etc/sudoers, otherwise it'll prompt for a
/// password and fail.
pub fn reload_config() {
    let output = Command::new("sudo").args(["pkill", "-HUP", "squid"]).output();
    if let Err(e) = output {
      tracing::error!("Failed to HUP squid: {:?}", e);
    } else {
      tracing::info!("Successfully sent HUP to squid");
    }
}