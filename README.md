# huawei-cpe-rs

A Rust library for controlling Huawei CPE routers.

## Supported Devices

- HUAWEI B528s
- HUAWEI B818

## Features

| Feature | Description |
| --- | --- |
| Device reboot | Reboot the router |
| Device information | IMEI, versions, WAN IP, etc. |
| Signal information | RSSI, RSRP, RSRQ, SINR, band, cell ID |
| Monitoring status | Connection status, network type, SIM status |
| Network mode | Switch between Auto, 2G, 3G, 4G, or combinations |
| PLMN info | Current provider name, MCC/MNC |
| Mobile data switch | Toggle mobile data on/off |
| WiFi switch | Toggle WiFi on/off |

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
huawei-cpe-rs = "0.1"
```

## Usage

```rust
use anyhow::Result;
use huawei_cpe_rs::{RouterClient, client::HuaweiClient};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = HuaweiClient::new("http://192.168.8.1")?;
    client.login("admin", "password").await?;

    let info = client.get_device_information().await?;
    println!("Device: {:?}", info.device_name);

    let signal = client.get_device_signal().await?;
    println!("RSRP: {:?} dBm", signal.rsrp_dbm());

    client.logout().await?;
    Ok(())
}
```

See the [examples/](examples/) directory for more usage.

## Examples

All examples read credentials from environment variables:

- `ROUTER_URL` — Router address (default: `http://192.168.8.1`)
- `ROUTER_USER` — Login username (default: `admin`)
- `ROUTER_PASS` — Login password (**required**)

```sh
export ROUTER_URL=http://192.168.8.1
export ROUTER_USER=admin
export ROUTER_PASS=yourpassword

# Get device info and signal status
cargo run --example device_info

# Switch network mode (3G → Auto)
cargo run --example network_mode

# Reboot the router
cargo run --example reboot
```

## Acknowledgements

This project was inspired by and uses knowledge from:

- [huawei-lte-api](https://github.com/Salamek/huawei-lte-api)
- [huawei-modem-python-api-client](https://github.com/pablo/huawei-modem-python-api-client)

## License

This project is licensed under the GNU GENERAL PUBLIC LICENSE.
