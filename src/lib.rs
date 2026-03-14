//! # huawei-cpe-rs
//!
//! A Rust library for controlling Huawei LTE/4G CPE routers via their XML-based
//! HTTP API.
//!
//! ## Supported Devices
//!
//! | Device | Tested |
//! |---|---|
//! | HUAWEI B528s | Yes |
//! | HUAWEI B818 | Yes |
//!
//! Other Huawei HiLink-based routers that expose the same `/api/` XML endpoints
//! are likely to work as well.
//!
//! ## Quick Start
//!
//! ```no_run
//! use anyhow::Result;
//! use huawei_cpe_rs::{RouterClient, client::HuaweiClient};
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut client = HuaweiClient::new("http://192.168.8.1")?;
//!     client.login("admin", "password").await?;
//!
//!     let info = client.get_device_information().await?;
//!     println!("Device: {:?}", info.device_name);
//!
//!     let signal = client.get_device_signal().await?;
//!     println!("RSRP: {:?} dBm", signal.rsrp_dbm());
//!
//!     client.logout().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! | Feature | Description |
//! |---|---|
//! | Login / Logout | SHA-256 and Base64 authentication |
//! | Device reboot | Reboot the router |
//! | Device information | IMEI, versions, WAN IP, etc. |
//! | Signal information | RSSI, RSRP, RSRQ, SINR, band, cell ID |
//! | Monitoring status | Connection status, network type, SIM status |
//! | Network mode | Switch between Auto, 2G, 3G, 4G, or combinations |
//! | PLMN info | Current provider name, MCC/MNC |
//! | Mobile data switch | Toggle mobile data on/off |
//! | WiFi switch | Toggle WiFi on/off |

use anyhow::{Context, Result, bail};
use reqwest::Url;

pub mod client;
pub mod enums;
pub mod session;

#[cfg(test)]
mod tests;

pub(crate) fn normalize_router_url(url: &str) -> Result<String> {
    let mut target = Url::parse(url)
        .with_context(|| format!("Invalid router URL: {url}"))?;

    if target.cannot_be_a_base() {
        bail!("Router URL must be an absolute base URL: {url}");
    }

    if target.query().is_some() || target.fragment().is_some() {
        bail!("Router URL must not include a query string or fragment: {url}");
    }

    if !target.path().ends_with('/') {
        let normalized_path = format!("{}/", target.path().trim_end_matches('/'));
        target.set_path(&normalized_path);
    }

    Ok(target.into())
}

/// Common trait implemented by all Huawei router clients.
#[async_trait::async_trait]
pub trait RouterClient {
    /// Authenticate with the router.
    async fn login(&mut self, username: &str, password: &str) -> Result<()>;

    /// Log out from the router.
    async fn logout(&mut self) -> Result<()>;

    /// Reboot the router.
    async fn reboot(&self) -> Result<()>;

    /// Toggle mobile data on/off.
    async fn set_mobile_dataswitch(&self, enabled: bool) -> Result<()>;

    /// Toggle WiFi on/off.
    async fn set_wifi_enabled(&self, enabled: bool) -> Result<()>;

    /// Set the network mode (e.g. auto, 3G only, 4G only).
    ///
    /// If `network_band`/`lte_band` are `None`, Huawei "ALL" masks are used.
    async fn set_net_mode(
        &self,
        mode: enums::NetworkMode,
        network_band: Option<enums::NetworkBandMask>,
        lte_band: Option<enums::LteBandMask>,
    ) -> Result<()>;

    /// Get device information.
    async fn get_device_information(&self) -> Result<client::DeviceInformation>;

    /// Get signal information.
    async fn get_device_signal(&self) -> Result<client::DeviceSignal>;

    /// Get monitoring status.
    async fn get_monitoring_status(&self) -> Result<client::MonitoringStatus>;
}
