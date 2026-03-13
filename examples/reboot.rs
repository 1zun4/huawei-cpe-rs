//! Reboot the router.
//!
//! ```sh
//! ROUTER_PASS=yourpassword cargo run --example reboot
//! ```

use std::env;

use anyhow::Result;
use huawei_cpe_rs::client::HuaweiClient;
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

    println!("Rebooting router at {}...", url);
    client.reboot().await?;
    println!("Reboot command sent. The router will restart shortly.");

    Ok(())
}
