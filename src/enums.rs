use std::ops;

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

/// 3G/legacy network band bitmask for `NetworkBand` in `api/net/net-mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkBandMask(pub u64);

impl NetworkBandMask {
    pub const BC0A: Self = Self(0x01);
    pub const BC0B: Self = Self(0x02);
    pub const BC1: Self = Self(0x04);
    pub const BC2: Self = Self(0x08);
    pub const BC3: Self = Self(0x10);
    pub const BC4: Self = Self(0x20);
    pub const BC5: Self = Self(0x40);
    pub const GSM1800: Self = Self(0x80);
    pub const GSM900: Self = Self(0x300);
    pub const BC6: Self = Self(0x400);
    pub const BC7: Self = Self(0x800);
    pub const BC8: Self = Self(0x1000);
    pub const BC9: Self = Self(0x2000);
    pub const BC10: Self = Self(0x4000);
    pub const BC11: Self = Self(0x8000);
    pub const GSM850: Self = Self(0x80000);
    pub const GSM1900: Self = Self(0x200000);
    pub const UMTS_B1_2100: Self = Self(0x400000);
    pub const UMTS_B2_1900: Self = Self(0x800000);
    pub const UMTS_B5_850: Self = Self(0x4000000);
    pub const BC12: Self = Self(0x10000000);
    pub const BC13: Self = Self(0x20000000);
    pub const BC14: Self = Self(0x80000000);
    pub const UMTS_B8_900: Self = Self(0x2000000000000);
    pub const ALL: Self = Self(0x3FFFFFFF);

    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub fn as_api_hex(self) -> String {
        format!("{:x}", self.0)
    }
}

impl std::ops::BitOr for NetworkBandMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// LTE band bitmask for `LTEBand` in `api/net/net-mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LteBandMask(pub u64);

impl LteBandMask {
    pub const B1: Self = Self(0x01);
    pub const B3: Self = Self(0x04);
    pub const B7: Self = Self(0x40);
    pub const B8: Self = Self(0x80);
    pub const B20: Self = Self(0x80000);
    pub const B28: Self = Self(0x8000000);
    pub const B38: Self = Self(0x2000000000);
    pub const B40: Self = Self(0x8000000000);
    pub const ALL: Self = Self(0x7FFFFFFFFFFFFFFF);

    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u64 {
        self.0
    }

    pub fn as_api_hex(self) -> String {
        format!("{:x}", self.0)
    }
}

impl ops::BitOr for LteBandMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
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
