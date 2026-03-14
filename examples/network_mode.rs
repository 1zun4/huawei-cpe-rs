//! Set network mode and optional band masks once.
//!
//! ```sh
//! ROUTER_PASS=yourpassword ROUTER_NET_MODE=4g cargo run --example network_mode
//! ROUTER_PASS=yourpassword ROUTER_NET_MODE=4g ROUTER_LTE_BANDS=1,3,8,20 cargo run --example network_mode
//! ```

use std::env;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use huawei_cpe_rs::client::HuaweiClient;
use huawei_cpe_rs::enums::{LteBandMask, NetworkBandMask, NetworkMode};
use huawei_cpe_rs::RouterClient;
use tokio::time::sleep;

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parse_mode(s: &str) -> Result<NetworkMode> {
    let normalized = s.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "auto" | "00" => Ok(NetworkMode::Auto),
        "2g" | "2g-only" | "01" => Ok(NetworkMode::Mode2G),
        "3g" | "3g-only" | "02" => Ok(NetworkMode::Mode3G),
        "4g" | "4g-only" | "03" => Ok(NetworkMode::Mode4G),
        "3g2g" | "3g-2g" | "0201" => Ok(NetworkMode::Mode3G2GAuto),
        "4g3g" | "4g-3g" | "0302" => Ok(NetworkMode::Mode4G3GAuto),
        "4g2g" | "4g-2g" | "0301" => Ok(NetworkMode::Mode4G2GAuto),
        _ => bail!("Unsupported ROUTER_NET_MODE value: {s}"),
    }
}

fn parse_hex_mask(s: &str) -> Result<u64> {
    let trimmed = s.trim();
    let hex = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .unwrap_or(trimmed);
    u64::from_str_radix(hex, 16)
        .with_context(|| format!("Invalid hex mask: {s}"))
}

fn parse_lte_bands_csv(s: &str) -> Result<LteBandMask> {
    let mut mask = 0_u64;
    for raw in s.split(',') {
        let band = raw.trim();
        if band.is_empty() {
            continue;
        }
        let value = match band {
            "all" | "ALL" => return Ok(LteBandMask::ALL),
            "1" => LteBandMask::B1.bits(),
            "3" => LteBandMask::B3.bits(),
            "7" => LteBandMask::B7.bits(),
            "8" => LteBandMask::B8.bits(),
            "20" => LteBandMask::B20.bits(),
            "28" => LteBandMask::B28.bits(),
            "38" => LteBandMask::B38.bits(),
            "40" => LteBandMask::B40.bits(),
            _ => bail!("Unsupported LTE band in ROUTER_LTE_BANDS: {band}"),
        };
        mask |= value;
    }
    if mask == 0 {
        bail!("ROUTER_LTE_BANDS did not contain any usable bands");
    }
    Ok(LteBandMask::from_bits(mask))
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let url = env_or("ROUTER_URL", "http://192.168.8.1");
    let user = env_or("ROUTER_USER", "admin");
    let mode = parse_mode(&env_or("ROUTER_NET_MODE", "auto"))?;
    let network_band = match env::var("ROUTER_NETWORK_BAND_HEX") {
        Ok(v) => Some(NetworkBandMask::from_bits(parse_hex_mask(&v)?)),
        Err(_) => None,
    };

    let lte_band = match (env::var("ROUTER_LTE_BAND_HEX"), env::var("ROUTER_LTE_BANDS")) {
        (Ok(hex), _) => Some(LteBandMask::from_bits(parse_hex_mask(&hex)?)),
        (Err(_), Ok(csv)) => Some(parse_lte_bands_csv(&csv)?),
        (Err(_), Err(_)) => None,
    };

    let pass = env::var("ROUTER_PASS")
        .expect("Set ROUTER_PASS environment variable");

    let mut client = HuaweiClient::new(&url)?;
    client.login(&user, &pass).await?;

    let before = client.get_net_mode().await?;
    println!(
        "Current mode: NetworkMode={:?}, Band={:?}, LTEBand={:?}",
        before.network_mode, before.network_band, before.lte_band
    );

    println!(
        "\nApplying requested mode: {:?}, NetworkBand={:?}, LTEBand={:?}",
        mode,
        network_band.map(|b| b.as_api_hex()),
        lte_band.map(|b| b.as_api_hex())
    );
    client
        .set_net_mode(mode, network_band, lte_band)
        .await?;
    println!("Waiting 10s for mode change...");
    sleep(Duration::from_secs(10)).await;

    let after = client.get_net_mode().await?;
    println!(
        "Configured mode after apply: NetworkMode={:?}, Band={:?}, LTEBand={:?}",
        after.network_mode, after.network_band, after.lte_band
    );

    let status = client.get_monitoring_status().await?;
    println!("Current network type: {:?} ({})",
        status.current_network_type,
        status.network_type_name().unwrap_or("unknown"));

    client.logout().await?;
    println!("\nDone.");
    Ok(())
}
