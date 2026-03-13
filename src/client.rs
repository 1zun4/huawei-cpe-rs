use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::enums::{ControlMode, NetworkMode};
use crate::session::HuaweiSession;
use crate::RouterClient;

/// Huawei B528s (and compatible) client.
///
/// Uses the Huawei XML-based HTTP API.
pub struct HuaweiClient {
    session: HuaweiSession,
}

// --- Response structs ---
// All response structs include a `data` field with the full raw JSON for
// fields that could not be deserialized. This ensures forward-compatibility
// with varying firmware versions.

/// Device information from `api/device/information`.
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceInformation {
    #[serde(rename = "DeviceName", default)]
    pub device_name: Option<String>,
    #[serde(rename = "SerialNumber", default)]
    pub serial_number: Option<String>,
    #[serde(rename = "Imei", default)]
    pub imei: Option<String>,
    #[serde(rename = "Imsi", default)]
    pub imsi: Option<String>,
    #[serde(rename = "Iccid", default)]
    pub iccid: Option<String>,
    #[serde(rename = "Msisdn", default)]
    pub msisdn: Option<String>,
    #[serde(rename = "HardwareVersion", default)]
    pub hardware_version: Option<String>,
    #[serde(rename = "SoftwareVersion", default)]
    pub software_version: Option<String>,
    #[serde(rename = "WebUIVersion", default)]
    pub webui_version: Option<String>,
    #[serde(rename = "MacAddress1", default)]
    pub mac_address_1: Option<String>,
    #[serde(rename = "MacAddress2", default)]
    pub mac_address_2: Option<String>,
    #[serde(rename = "ProductFamily", default)]
    pub product_family: Option<String>,
    #[serde(rename = "Classify", default)]
    pub classify: Option<String>,
    #[serde(rename = "WanIPAddress", default)]
    pub wan_ip_address: Option<String>,

    /// Raw data containing all fields as JSON. Use this to access fields that
    /// are not explicitly deserialized.
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Device signal from `api/device/signal`.
#[derive(Debug, Clone, Deserialize)]
pub struct DeviceSignal {
    /// Current cell ID.
    #[serde(rename = "cell_id", default)]
    pub cell_id: Option<String>,
    /// RSSI in dBm (e.g. "-75dBm").
    #[serde(rename = "rssi", default)]
    pub rssi: Option<String>,
    /// RSRP in dBm.
    #[serde(rename = "rsrp", default)]
    pub rsrp: Option<String>,
    /// RSRQ in dB.
    #[serde(rename = "rsrq", default)]
    pub rsrq: Option<String>,
    /// SINR in dB.
    #[serde(rename = "sinr", default)]
    pub sinr: Option<String>,
    /// RSCP in dBm (3G).
    #[serde(rename = "rscp", default)]
    pub rscp: Option<String>,
    /// Ec/Io in dB (3G).
    #[serde(rename = "ecio", default)]
    pub ecio: Option<String>,
    /// Current band info.
    #[serde(rename = "band", default)]
    pub band: Option<String>,
    /// DL bandwidth.
    #[serde(rename = "dlbandwidth", default)]
    pub dl_bandwidth: Option<String>,
    /// UL bandwidth.
    #[serde(rename = "ulbandwidth", default)]
    pub ul_bandwidth: Option<String>,
    /// Mode (e.g. 7 for LTE).
    #[serde(rename = "mode", default)]
    pub mode: Option<String>,

    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl DeviceSignal {
    /// Parse RSSI as integer, stripping "dBm" suffix.
    pub fn rssi_dbm(&self) -> Option<i32> {
        parse_signal_value(self.rssi.as_deref())
    }

    /// Parse RSRP as integer, stripping "dBm" suffix.
    pub fn rsrp_dbm(&self) -> Option<i32> {
        parse_signal_value(self.rsrp.as_deref())
    }

    /// Parse RSRQ as integer, stripping "dB" suffix.
    pub fn rsrq_db(&self) -> Option<i32> {
        parse_signal_value(self.rsrq.as_deref())
    }

    /// Parse SINR as integer, stripping "dB" suffix.
    pub fn sinr_db(&self) -> Option<i32> {
        parse_signal_value(self.sinr.as_deref())
    }
}

/// Monitoring status from `api/monitoring/status`.
#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringStatus {
    #[serde(rename = "ConnectionStatus", default)]
    pub connection_status: Option<String>,
    #[serde(rename = "WifiConnectionStatus", default)]
    pub wifi_connection_status: Option<String>,
    #[serde(rename = "SignalStrength", default)]
    pub signal_strength: Option<String>,
    #[serde(rename = "SignalIcon", default)]
    pub signal_icon: Option<String>,
    #[serde(rename = "CurrentNetworkType", default)]
    pub current_network_type: Option<String>,
    #[serde(rename = "CurrentServiceDomain", default)]
    pub current_service_domain: Option<String>,
    #[serde(rename = "RoamingStatus", default)]
    pub roaming_status: Option<String>,
    #[serde(rename = "BatteryStatus", default)]
    pub battery_status: Option<String>,
    #[serde(rename = "BatteryLevel", default)]
    pub battery_level: Option<String>,
    #[serde(rename = "BatteryPercent", default)]
    pub battery_percent: Option<String>,
    #[serde(rename = "PrimaryDns", default)]
    pub primary_dns: Option<String>,
    #[serde(rename = "SecondaryDns", default)]
    pub secondary_dns: Option<String>,
    #[serde(rename = "PrimaryIPv6Dns", default)]
    pub primary_ipv6_dns: Option<String>,
    #[serde(rename = "SecondaryIPv6Dns", default)]
    pub secondary_ipv6_dns: Option<String>,
    #[serde(rename = "CurrentWifiUser", default)]
    pub current_wifi_user: Option<String>,
    #[serde(rename = "TotalWifiUser", default)]
    pub total_wifi_user: Option<String>,
    #[serde(rename = "ServiceStatus", default)]
    pub service_status: Option<String>,
    #[serde(rename = "SimStatus", default)]
    pub sim_status: Option<String>,
    #[serde(rename = "WifiStatus", default)]
    pub wifi_status: Option<String>,
    #[serde(rename = "CurrentNetworkTypeEx", default)]
    pub current_network_type_ex: Option<String>,
    #[serde(rename = "maxsignal", default)]
    pub max_signal: Option<String>,
    #[serde(rename = "wifiindooronly", default)]
    pub wifi_indoor_only: Option<String>,
    #[serde(rename = "WifiFrequence", default)]
    pub wifi_frequence: Option<String>,
    #[serde(rename = "classify", default)]
    pub classify: Option<String>,
    #[serde(rename = "flymode", default)]
    pub flymode: Option<String>,
    #[serde(rename = "cellroam", default)]
    pub cellroam: Option<String>,

    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl MonitoringStatus {
    /// Parse CurrentNetworkType/CurrentNetworkTypeEx to a human-readable string.
    ///
    /// Prefers CurrentNetworkTypeEx when available (more specific), falls back
    /// to CurrentNetworkType.
    pub fn network_type_name(&self) -> Option<&'static str> {
        // Try the extended type first (more specific on newer firmwares)
        if let Some(name) = self.current_network_type_ex.as_deref().and_then(map_network_type) {
            return Some(name);
        }
        self.current_network_type.as_deref().and_then(map_network_type)
    }
}

fn map_network_type(s: &str) -> Option<&'static str> {
    match s {
        "0" => Some("No Service"),
        "1" => Some("GSM"),
        "2" => Some("GPRS"),
        "3" => Some("EDGE"),
        "4" => Some("WCDMA"),
        "5" => Some("HSDPA"),
        "6" => Some("HSUPA"),
        "7" => Some("HSPA"),
        "8" => Some("TD-SCDMA"),
        "9" => Some("HSPA+"),
        "10" => Some("EV-DO rev. 0"),
        "11" => Some("EV-DO rev. A"),
        "12" => Some("EV-DO rev. B"),
        "13" => Some("1xRTT"),
        "14" => Some("UMB"),
        "15" => Some("1xEVDV"),
        "16" => Some("3xRTT"),
        "17" => Some("HSPA+ 64QAM"),
        "18" => Some("HSPA+ MIMO"),
        "19" => Some("LTE"),
        "41" => Some("UMTS"),
        "44" => Some("HSPA"),
        "45" => Some("HSPA+"),
        "46" => Some("DC-HSPA+"),
        "64" => Some("HSPA (64QAM)"),
        "65" => Some("HSPA+"),
        "101" => Some("LTE"),
        "1011" => Some("LTE+"),
        "111" => Some("LTE+"),
        _ => None,
    }
}

/// WiFi network switch state from `api/wlan/wifi-network-switch`.
#[derive(Debug, Clone, Deserialize)]
pub struct WifiNetworkSwitch {
    #[serde(rename = "WifiEnable", default)]
    pub wifi_enable: Option<String>,

    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Network mode from `api/net/net-mode`.
#[derive(Debug, Clone, Deserialize)]
pub struct NetMode {
    #[serde(rename = "NetworkMode", default)]
    pub network_mode: Option<String>,
    #[serde(rename = "NetworkBand", default)]
    pub network_band: Option<String>,
    #[serde(rename = "LTEBand", default)]
    pub lte_band: Option<String>,

    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Current PLMN info from `api/net/current-plmn`.
#[derive(Debug, Clone, Deserialize)]
pub struct CurrentPlmn {
    #[serde(rename = "State", default)]
    pub state: Option<String>,
    #[serde(rename = "FullName", default)]
    pub full_name: Option<String>,
    #[serde(rename = "ShortName", default)]
    pub short_name: Option<String>,
    #[serde(rename = "Numeric", default)]
    pub numeric: Option<String>,
    #[serde(rename = "Rat", default)]
    pub rat: Option<String>,

    #[serde(flatten)]
    pub data: serde_json::Value,
}

// --- Client implementation ---

impl HuaweiClient {
    /// Create a new client for the given router URL.
    pub fn new(url: &str) -> Result<Self> {
        let session = HuaweiSession::new(url)?;
        Ok(Self { session })
    }

    /// Access the underlying session for advanced API calls.
    pub fn session(&self) -> &HuaweiSession {
        &self.session
    }

    /// Initialize session (fetches CSRF tokens). Must be called before login.
    pub async fn initialize(&self) -> Result<()> {
        self.session.initialize().await
    }

    /// Get current PLMN (network provider) info.
    pub async fn get_current_plmn(&self) -> Result<CurrentPlmn> {
        self.session.api_get("net/current-plmn").await
    }

    /// Get current network mode settings.
    pub async fn get_net_mode(&self) -> Result<NetMode> {
        self.session.api_get("net/net-mode").await
    }

    /// Get WiFi network switch status.
    pub async fn get_wifi_network_switch(&self) -> Result<WifiNetworkSwitch> {
        self.session.api_get("wlan/wifi-network-switch").await
    }
}

#[async_trait::async_trait]
impl RouterClient for HuaweiClient {
    async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        self.session.initialize().await?;
        self.session.login(username, password).await
    }

    async fn logout(&mut self) -> Result<()> {
        self.session.logout().await
    }

    async fn reboot(&self) -> Result<()> {
        #[derive(Serialize)]
        struct ControlRequest {
            #[serde(rename = "Control")]
            control: i32,
        }

        self.session
            .api_post_set(
                "device/control",
                &ControlRequest {
                    control: ControlMode::Reboot as i32,
                },
            )
            .await
            .context("Reboot failed")?;
        Ok(())
    }

    async fn set_mobile_dataswitch(&self, enabled: bool) -> Result<()> {
        #[derive(Serialize)]
        struct DataSwitchRequest {
            dataswitch: i32,
        }

        self.session
            .api_post_set(
                "dialup/mobile-dataswitch",
                &DataSwitchRequest {
                    dataswitch: if enabled { 1 } else { 0 },
                },
            )
            .await
            .context("Set mobile dataswitch failed")?;
        Ok(())
    }

    async fn set_wifi_enabled(&self, enabled: bool) -> Result<()> {
        #[derive(Serialize)]
        struct WifiSwitchRequest {
            #[serde(rename = "WifiEnable")]
            wifi_enable: i32,
        }

        self.session
            .api_post_set(
                "wlan/wifi-network-switch",
                &WifiSwitchRequest {
                    wifi_enable: if enabled { 1 } else { 0 },
                },
            )
            .await
            .context("Set WiFi switch failed")?;
        Ok(())
    }

    async fn set_net_mode(&self, mode: NetworkMode) -> Result<()> {
        #[derive(Serialize)]
        struct NetModeRequest {
            #[serde(rename = "NetworkMode")]
            network_mode: String,
            #[serde(rename = "NetworkBand")]
            network_band: String,
            #[serde(rename = "LTEBand")]
            lte_band: String,
        }

        self.session
            .api_post_set(
                "net/net-mode",
                &NetModeRequest {
                    network_mode: mode.as_api_str().to_string(),
                    network_band: "3FFFFFFF".to_string(),
                    lte_band: "7FFFFFFFFFFFFFFF".to_string(),
                },
            )
            .await
            .context("Set net mode failed")?;
        Ok(())
    }

    async fn get_device_information(&self) -> Result<DeviceInformation> {
        self.session
            .api_get("device/information")
            .await
            .context("Get device information failed")
    }

    async fn get_device_signal(&self) -> Result<DeviceSignal> {
        self.session
            .api_get("device/signal")
            .await
            .context("Get device signal failed")
    }

    async fn get_monitoring_status(&self) -> Result<MonitoringStatus> {
        self.session
            .api_get("monitoring/status")
            .await
            .context("Get monitoring status failed")
    }
}

/// Filter empty strings returned by the Huawei XML API as None.
pub fn non_empty(s: &Option<String>) -> Option<&str> {
    s.as_deref().filter(|v| !v.is_empty())
}

/// Parse a signal value string like "-75dBm" or "-12dB" to an i32.
pub fn parse_signal_value(s: Option<&str>) -> Option<i32> {
    let s = s?.trim();
    if s.is_empty() {
        return None;
    }
    let cleaned: String = s
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '.')
        .collect();
    cleaned.parse::<f64>().ok().map(|v| v as i32)
}
