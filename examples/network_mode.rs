//! Switch network mode to force new IP assignment, then switch back.
//!
//! This is commonly used to rotate the public IP address by switching
//! from LTE to 3G and back to Auto.
//!
//! ```sh
//! ROUTER_PASS=yourpassword cargo run --example network_mode
//! ```

use std::env;
use std::time::Duration;

use anyhow::Result;
use huawei_cpe_rs::client::HuaweiClient;
use huawei_cpe_rs::enums::NetworkMode;
use huawei_cpe_rs::RouterClient;
use tokio::time::sleep;

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

    // Show current mode
    let net_mode = client.get_net_mode().await?;
    println!("Current mode: NetworkMode={:?}, Band={:?}, LTEBand={:?}",
        net_mode.network_mode, net_mode.network_band, net_mode.lte_band);

    // Switch to 3G
    println!("\nSwitching to 3G...");
    client.set_net_mode(NetworkMode::Mode3G).await?;
    println!("Waiting 5s for mode change...");
    sleep(Duration::from_secs(5)).await;

    let status = client.get_monitoring_status().await?;
    println!("Network type after 3G switch: {:?} ({})",
        status.current_network_type,
        status.network_type_name().unwrap_or("unknown"));

    // Switch back to Auto
    println!("\nSwitching back to Auto...");
    client.set_net_mode(NetworkMode::Auto).await?;
    println!("Waiting 5s for mode change...");
    sleep(Duration::from_secs(5)).await;

    let status = client.get_monitoring_status().await?;
    println!("Network type after Auto switch: {:?} ({})",
        status.current_network_type,
        status.network_type_name().unwrap_or("unknown"));

    client.logout().await?;
    println!("\nDone.");
    Ok(())
}
