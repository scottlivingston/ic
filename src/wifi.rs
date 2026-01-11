use bevy::prelude::*;
use serde::Serialize;
use std::sync::Arc;
#[cfg(not(feature = "pi"))]
use std::sync::Mutex;

pub const HOTSPOT_SSID: &str = "IC-Setup";
pub const HOTSPOT_PASSWORD: &str = "icsetup123";
pub const HOTSPOT_IP: &str = "10.42.0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityStatus {
    Full,
    #[cfg_attr(not(feature = "pi"), allow(dead_code))]
    Limited,
    None,
}

#[derive(Debug, Clone, Serialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub security: String,
    pub in_use: bool,
}

#[derive(Debug)]
pub enum WifiError {
    CommandFailed(String),
}

impl std::fmt::Display for WifiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WifiError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
        }
    }
}

impl std::error::Error for WifiError {}

// ============================================================================
// Trait Definition
// ============================================================================

pub trait WifiManager: Send + Sync {
    fn check_connectivity(&self) -> ConnectivityStatus;
    fn scan_networks(&self) -> Result<Vec<WifiNetwork>, WifiError>;
    fn connect(&self, ssid: &str, password: &str) -> Result<(), WifiError>;
    fn forget_network(&self, ssid: &str) -> Result<(), WifiError>;
    fn start_hotspot(&self) -> Result<(), WifiError>;
    fn stop_hotspot(&self) -> Result<(), WifiError>;
    fn get_ip_address(&self) -> Option<String>;
    fn get_current_ssid(&self) -> Option<String>;
}

// ============================================================================
// Bevy Resource
// ============================================================================

#[derive(Resource)]
pub struct WifiService(pub Arc<dyn WifiManager>);

impl WifiService {
    pub fn new(manager: impl WifiManager + 'static) -> Self {
        Self(Arc::new(manager))
    }
}

// ============================================================================
// Real Implementation (NetworkManager/nmcli)
// ============================================================================

#[cfg(feature = "pi")]
pub struct NetworkManagerWifi;

#[cfg(feature = "pi")]
impl WifiManager for NetworkManagerWifi {
    fn check_connectivity(&self) -> ConnectivityStatus {
        use std::process::Command;

        let output = Command::new("nmcli")
            .args(["networking", "connectivity", "check"])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let status = stdout.trim();
                match status {
                    "full" => ConnectivityStatus::Full,
                    "limited" => ConnectivityStatus::Limited,
                    _ => ConnectivityStatus::None,
                }
            }
            Err(_) => ConnectivityStatus::None,
        }
    }

    fn scan_networks(&self) -> Result<Vec<WifiNetwork>, WifiError> {
        use std::process::Command;

        // Rescan first
        let _ = Command::new("nmcli")
            .args(["device", "wifi", "rescan"])
            .output();

        // Small delay for scan to complete
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Get network list with specific fields
        let output = Command::new("nmcli")
            .args([
                "-t",
                "-f",
                "IN-USE,SSID,SIGNAL,SECURITY",
                "device",
                "wifi",
                "list",
            ])
            .output()
            .map_err(|e| WifiError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            return Err(WifiError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut networks = Vec::new();
        let mut seen_ssids = std::collections::HashSet::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                let in_use = parts[0] == "*";
                let ssid = parts[1].to_string();
                let signal: u8 = parts[2].parse().unwrap_or(0);
                let security = parts[3].to_string();

                // Skip empty SSIDs and duplicates
                if ssid.is_empty() || seen_ssids.contains(&ssid) {
                    continue;
                }
                seen_ssids.insert(ssid.clone());

                networks.push(WifiNetwork {
                    ssid,
                    signal,
                    security,
                    in_use,
                });
            }
        }

        // Sort by signal strength (strongest first)
        networks.sort_by(|a, b| b.signal.cmp(&a.signal));

        Ok(networks)
    }

    fn connect(&self, ssid: &str, password: &str) -> Result<(), WifiError> {
        use std::process::Command;

        // First, stop any hotspot that might be running
        let _ = self.stop_hotspot();

        let output = Command::new("nmcli")
            .args(["device", "wifi", "connect", ssid, "password", password])
            .output()
            .map_err(|e| WifiError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WifiError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    fn forget_network(&self, ssid: &str) -> Result<(), WifiError> {
        use std::process::Command;

        let output = Command::new("nmcli")
            .args(["connection", "delete", ssid])
            .output()
            .map_err(|e| WifiError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WifiError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    fn start_hotspot(&self) -> Result<(), WifiError> {
        use std::process::Command;

        let output = Command::new("nmcli")
            .args([
                "device",
                "wifi",
                "hotspot",
                "ssid",
                HOTSPOT_SSID,
                "password",
                HOTSPOT_PASSWORD,
            ])
            .output()
            .map_err(|e| WifiError::CommandFailed(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(WifiError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    fn stop_hotspot(&self) -> Result<(), WifiError> {
        use std::process::Command;

        // Try to bring down the Hotspot connection
        let output = Command::new("nmcli")
            .args(["connection", "down", "Hotspot"])
            .output()
            .map_err(|e| WifiError::CommandFailed(e.to_string()))?;

        // It's okay if it wasn't active
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Only error if it's not a "not active" error
            if !stderr.contains("not active") && !stderr.contains("unknown") {
                return Err(WifiError::CommandFailed(stderr.to_string()));
            }
        }

        Ok(())
    }

    fn get_ip_address(&self) -> Option<String> {
        use std::process::Command;

        // Get IP from the primary connection
        let output = Command::new("hostname").args(["-I"]).output().ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let ip = stdout.split_whitespace().next()?;

        // Validate it looks like an IP
        if ip.contains('.') && !ip.starts_with("10.42.") {
            Some(ip.to_string())
        } else if ip.contains('.') {
            // Might be hotspot IP, try to get the second one
            stdout
                .split_whitespace()
                .find(|ip| !ip.starts_with("10.42."))
                .map(|s| s.to_string())
                .or_else(|| Some(ip.to_string()))
        } else {
            None
        }
    }

    fn get_current_ssid(&self) -> Option<String> {
        use std::process::Command;

        let output = Command::new("nmcli")
            .args(["-t", "-f", "ACTIVE,SSID", "device", "wifi"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.starts_with("yes:") {
                let ssid = line.strip_prefix("yes:")?;
                if !ssid.is_empty() && ssid != HOTSPOT_SSID {
                    return Some(ssid.to_string());
                }
            }
        }
        None
    }
}

// ============================================================================
// Mock Implementation (for debug/testing)
// ============================================================================

#[cfg(not(feature = "pi"))]
#[derive(Default)]
struct MockWifiState {
    connected: bool,
    hotspot_active: bool,
    connected_ssid: Option<String>,
}

#[cfg(not(feature = "pi"))]
pub struct MockWifi {
    state: Mutex<MockWifiState>,
}

#[cfg(not(feature = "pi"))]
impl MockWifi {
    pub fn new() -> Self {
        // Default to connected; set IC_MOCK_DISCONNECTED=1 to start in onboarding mode
        let start_connected = std::env::var("IC_MOCK_DISCONNECTED").is_err();

        Self {
            state: Mutex::new(MockWifiState {
                connected: start_connected,
                hotspot_active: false,
                connected_ssid: if start_connected {
                    Some("HomeWifi".to_string())
                } else {
                    None
                },
            }),
        }
    }
}

#[cfg(not(feature = "pi"))]
impl WifiManager for MockWifi {
    fn check_connectivity(&self) -> ConnectivityStatus {
        let state = self.state.lock().unwrap();
        if state.connected {
            ConnectivityStatus::Full
        } else {
            ConnectivityStatus::None
        }
    }

    fn scan_networks(&self) -> Result<Vec<WifiNetwork>, WifiError> {
        // Simulate scan delay
        std::thread::sleep(std::time::Duration::from_millis(500));

        let state = self.state.lock().unwrap();
        let connected_ssid = state.connected_ssid.as_deref();

        Ok(vec![
            WifiNetwork {
                ssid: "HomeWifi".to_string(),
                signal: 85,
                security: "WPA2".to_string(),
                in_use: connected_ssid == Some("HomeWifi"),
            },
            WifiNetwork {
                ssid: "Neighbor5G".to_string(),
                signal: 62,
                security: "WPA2".to_string(),
                in_use: connected_ssid == Some("Neighbor5G"),
            },
            WifiNetwork {
                ssid: "CoffeeShop".to_string(),
                signal: 45,
                security: "Open".to_string(),
                in_use: connected_ssid == Some("CoffeeShop"),
            },
            WifiNetwork {
                ssid: "SecureNet".to_string(),
                signal: 30,
                security: "WPA3".to_string(),
                in_use: connected_ssid == Some("SecureNet"),
            },
        ])
    }

    fn connect(&self, ssid: &str, _password: &str) -> Result<(), WifiError> {
        // Simulate connection delay
        std::thread::sleep(std::time::Duration::from_secs(1));

        let mut state = self.state.lock().unwrap();
        state.connected = true;
        state.hotspot_active = false;
        state.connected_ssid = Some(ssid.to_string());

        info!("[MockWifi] Connected to {}", ssid);
        Ok(())
    }

    fn forget_network(&self, ssid: &str) -> Result<(), WifiError> {
        let mut state = self.state.lock().unwrap();
        if state.connected_ssid.as_deref() == Some(ssid) {
            state.connected = false;
            state.connected_ssid = None;
        }
        info!("[MockWifi] Forgot network: {}", ssid);
        Ok(())
    }

    fn start_hotspot(&self) -> Result<(), WifiError> {
        let mut state = self.state.lock().unwrap();
        state.hotspot_active = true;
        info!("[MockWifi] Hotspot started");
        Ok(())
    }

    fn stop_hotspot(&self) -> Result<(), WifiError> {
        let mut state = self.state.lock().unwrap();
        state.hotspot_active = false;
        info!("[MockWifi] Hotspot stopped");
        Ok(())
    }

    fn get_ip_address(&self) -> Option<String> {
        let state = self.state.lock().unwrap();
        if state.connected {
            Some("192.168.1.100".to_string())
        } else if state.hotspot_active {
            Some(HOTSPOT_IP.to_string())
        } else {
            None
        }
    }

    fn get_current_ssid(&self) -> Option<String> {
        let state = self.state.lock().unwrap();
        state.connected_ssid.clone()
    }
}
