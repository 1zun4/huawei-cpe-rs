//! Get device information, signal quality, monitoring status, and PLMN info.
//!
//! ```sh
//! ROUTER_PASS=yourpassword cargo run --example device_info
//! ```

use std::env;

use anyhow::Result;
use huawei_cpe_rs::client::{HuaweiClient, non_empty};
use huawei_cpe_rs::RouterClient;

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let url = env_or("ROUTER_URL", "http://192.168.8.1");
    let user = env_or("ROUTER_USER", "admin");
    let pass = env::var("ROUTER_PASS")
        .expect("Set ROUTER_PASS environment variable");

    let mut client = HuaweiClient::new(&url)?;
    client.login(&user, &pass).await?;

    // Device information
    println!("=== Device Information ===");
    let info = client.get_device_information().await?;
    println!("  DeviceName:       {:?}", info.device_name);
    println!("  SerialNumber:     {:?}", info.serial_number);
    println!("  IMEI:             {:?}", info.imei);
    println!("  IMSI:             {:?}", info.imsi);
    println!("  ICCID:            {:?}", info.iccid);
    println!("  HardwareVersion:  {:?}", info.hardware_version);
    println!("  SoftwareVersion:  {:?}", info.software_version);
    println!("  WanIPAddress:     {:?}", info.wan_ip_address);
    println!("  ProductFamily:    {:?}", info.product_family);

    // Signal quality
    println!("\n=== Signal ===");
    let signal = client.get_device_signal().await?;
    println!("  RSSI:    {:?} (parsed: {:?} dBm)", signal.rssi, signal.rssi_dbm());
    println!("  RSRP:    {:?} (parsed: {:?} dBm)", signal.rsrp, signal.rsrp_dbm());
    println!("  RSRQ:    {:?} (parsed: {:?} dB)", signal.rsrq, signal.rsrq_db());
    println!("  SINR:    {:?} (parsed: {:?} dB)", signal.sinr, signal.sinr_db());
    println!("  Band:    {:?}", signal.band);
    println!("  Cell ID: {:?}", signal.cell_id);
    println!("  Mode:    {:?}", signal.mode);

    // Monitoring status
    println!("\n=== Monitoring Status ===");
    let status = client.get_monitoring_status().await?;
    println!("  ConnectionStatus:     {:?}", status.connection_status);
    println!("  CurrentNetworkType:   {:?}", status.current_network_type);
    println!("  CurrentNetworkTypeEx: {:?}", status.current_network_type_ex);
    println!("  Network type name:    {:?}", status.network_type_name());
    println!("  SignalStrength:       {:?}", status.signal_strength);
    println!("  SimStatus:            {:?}", status.sim_status);
    println!("  WifiStatus:           {:?}", status.wifi_status);

    // PLMN (provider)
    println!("\n=== Current PLMN ===");
    let plmn = client.get_current_plmn().await?;
    println!("  FullName:  {:?}", non_empty(&plmn.full_name));
    println!("  ShortName: {:?}", non_empty(&plmn.short_name));
    println!("  Numeric:   {:?}", non_empty(&plmn.numeric));

    client.logout().await?;
    println!("\nDone.");
    Ok(())
}
