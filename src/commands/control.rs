//! Probe control commands.
//  COMMAND_ID = 0x0d

use std::fmt;

use crate::{RiscvChip, WchLinkVariant};

use super::*;

/// GetDeviceVersion (0x0d, 0x01)
#[derive(Debug)]
pub struct GetProbeInfo;
impl Command for GetProbeInfo {
    type Response = ProbeInfo;
    const COMMAND_ID: u8 = 0x0d;
    fn payload(&self) -> Vec<u8> {
        vec![0x01]
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeInfo {
    pub major_version: u8,
    pub minor_version: u8,
    pub variant: WchLinkVariant,
}
impl ProbeInfo {
    pub fn version(&self) -> (u8, u8) {
        (self.major_version, self.minor_version)
    }
}
impl Response for ProbeInfo {
    fn from_payload(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 3 {
            return Err(crate::error::Error::InvalidPayloadLength);
        }
        Ok(Self {
            major_version: bytes[0],
            minor_version: bytes[1],
            // Only avaliable in newer version of firmware
            variant: if bytes.len() == 4 {
                WchLinkVariant::try_from_u8(bytes[2])?
            } else {
                WchLinkVariant::Ch549
            },
        })
    }
}
impl fmt::Display for ProbeInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WCH-Link v{}.{} ({})",
            self.major_version, self.minor_version, self.variant
        )
    }
}

/// ?SetChipType (0x0d, 0x02)
#[derive(Debug)]
pub struct AttachChip;
impl Command for AttachChip {
    type Response = AttachChipResponse;
    const COMMAND_ID: u8 = 0x0d;
    fn payload(&self) -> Vec<u8> {
        vec![0x02]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttachChipResponse {
    pub chip_family: RiscvChip,
    pub riscvchip: u8,
    pub chip_id: u32,
}
impl AttachChipResponse {}
impl Response for AttachChipResponse {
    fn from_payload(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 5 {
            return Err(crate::error::Error::InvalidPayloadLength);
        }
        Ok(Self {
            chip_family: RiscvChip::try_from_u8(bytes[0])?,
            riscvchip: bytes[0],
            chip_id: u32::from_be_bytes(bytes[1..5].try_into().unwrap()),
        })
    }
}
// For logging
impl fmt::Display for AttachChipResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.chip_id == 0 {
            write!(f, "{:?}", self.chip_family)
        } else if let Some(chip_name) = crate::chips::chip_id_to_chip_name(self.chip_id) {
            write!(
                f,
                "{:?} [{}] (ChipID: 0x{:08x})",
                self.chip_family, chip_name, self.chip_id
            )
        } else {
            write!(f, "{:?} (ChipID: 0x{:08x})", self.chip_family, self.chip_id)
        }
    }
}

/// Erase code flash, only supported by WCH-LinkE.
#[derive(Debug)]
pub enum EraseCodeFlash {
    ByPinRST(RiscvChip),
    ByPowerOff(RiscvChip),
}
impl Command for EraseCodeFlash {
    type Response = ();
    const COMMAND_ID: u8 = 0x0d;

    fn payload(&self) -> Vec<u8> {
        match self {
            // This is more complex, require RST pin to be connected.
            EraseCodeFlash::ByPinRST(c) => vec![0x08, *c as u8],
            // NOTE: From the protocol, this command's bytes is wrongly seted
            // 81 0d 01 0f 09, note here, the "length" bytes is wrong.
            // I guess it is not verified. So here we use `02`.
            EraseCodeFlash::ByPowerOff(c) => vec![0x0f, *c as u8],
        }
    }
}

/// GetROMRAM, Only avaliable for CH32V2, CH32V3, CH56X
/// 0, 1, 2, 3
#[derive(Debug)]
pub struct GetChipRomRamSplit;
impl Command for GetChipRomRamSplit {
    type Response = u8;
    const COMMAND_ID: u8 = 0x0d;
    fn payload(&self) -> Vec<u8> {
        vec![0x04]
    }
}

/// 0, 1, 2, 3
#[derive(Debug)]
pub struct SetChipRomRamSplit(u8);
impl Command for SetChipRomRamSplit {
    type Response = ();
    const COMMAND_ID: u8 = 0x0d;
    fn payload(&self) -> Vec<u8> {
        vec![0x05, self.0]
    }
}

// ?? close out
/// Detach Chip, (0x0d, 0xff)
#[derive(Debug)]
pub struct OptEnd;
impl Command for OptEnd {
    type Response = ();
    const COMMAND_ID: u8 = 0x0d;
    fn payload(&self) -> Vec<u8> {
        vec![0xff]
    }
}

/// Set Power, from pow3v3, pow5v fn
#[derive(Debug)]
pub enum SetPower {
    Enable3V3,
    Disable3V3,
    Enable5V,
    Disable5V,
}
impl Command for SetPower {
    type Response = ();
    const COMMAND_ID: u8 = 0x0d;
    fn payload(&self) -> Vec<u8> {
        match self {
            SetPower::Enable3V3 => vec![0x09],
            SetPower::Disable3V3 => vec![0x0A],
            SetPower::Enable5V => vec![0x0B],
            SetPower::Disable5V => vec![0x0C],
        }
    }
}
