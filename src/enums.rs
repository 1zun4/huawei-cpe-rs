/// Network mode for Huawei routers.
///
/// Values correspond to the NetworkMode field in the Huawei XML API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    /// Automatic mode selection.
    Auto,
    /// 2G only.
    Mode2G,
    /// 3G only.
    Mode3G,
    /// 4G only.
    Mode4G,
    /// 3G + 2G auto.
    Mode3G2GAuto,
    /// 4G + 3G auto.
    Mode4G3GAuto,
    /// 4G + 2G auto.
    Mode4G2GAuto,
}

impl NetworkMode {
    /// Returns the Huawei API string representation.
    pub fn as_api_str(&self) -> &'static str {
        match self {
            Self::Auto => "00",
            Self::Mode2G => "01",
            Self::Mode3G => "02",
            Self::Mode4G => "03",
            Self::Mode3G2GAuto => "0201",
            Self::Mode4G3GAuto => "0302",
            Self::Mode4G2GAuto => "0301",
        }
    }
}

/// Device control mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    /// Reboot the device.
    Reboot = 1,
    /// Factory reset.
    Reset = 2,
    /// Power off.
    PowerOff = 4,
}
