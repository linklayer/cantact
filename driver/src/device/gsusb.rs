//! Structure declarations for the GSUSB protocol
#![allow(dead_code)]

// can id is OR'd with flag when frame is extended
pub(crate) const GSUSB_EXT_FLAG: u32 = 0x8000_0000;
// can id is OR'd with flag when frame is RTR
pub(crate) const GSUSB_RTR_FLAG: u32 = 0x4000_0000;
// echo id for non-loopback frames
pub(crate) const GSUSB_RX_ECHO_ID: u32 = 0xFFFF_FFFF;

// device features bit map
pub(crate) const GSUSB_FEATURE_LISTEN_ONLY: u32 = 1;
pub(crate) const GSUSB_FEATURE_LOOP_BACK: u32 = 1 << 1;

#[repr(u8)]
#[derive(Debug)]
pub(crate) enum UsbBreq {
    HostFormat = 0,
    BitTiming,
    Mode,
    Berr,
    BitTimingConsts,
    DeviceConfig,
    Timestamp,
    Identify,
}
#[repr(u8)]
pub(crate) enum CanMode {
    Reset = 0,
    Start,
}

#[repr(u8)]
pub(crate) enum CanState {
    ErrorActive = 0,
    ErrorWarning,
    ErrorPassive,
    BusOff,
    Stopped,
    Sleeping,
}

fn u32_from_le_bytes(bs: &[u8]) -> u32 {
    let arr: [u8; 4] = [bs[0], bs[1], bs[2], bs[3]];
    u32::from_le_bytes(arr)
}

#[repr(C)]
pub(crate) struct Mode {
    pub mode: u32,
    pub flags: u32,
}
impl Mode {
    pub(crate) fn to_le_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];
        data.extend_from_slice(&self.mode.to_le_bytes());
        data.extend_from_slice(&self.flags.to_le_bytes());
        data
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct BitTiming {
    pub prop_seg: u32,
    pub phase_seg1: u32,
    pub phase_seg2: u32,
    pub sjw: u32,
    pub brp: u32,
}
impl BitTiming {
    pub(crate) fn to_le_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];
        data.extend_from_slice(&self.prop_seg.to_le_bytes());
        data.extend_from_slice(&self.phase_seg1.to_le_bytes());
        data.extend_from_slice(&self.phase_seg2.to_le_bytes());
        data.extend_from_slice(&self.sjw.to_le_bytes());
        data.extend_from_slice(&self.brp.to_le_bytes());
        data
    }
}

#[derive(Debug)]
#[repr(C)]
pub(crate) struct BitTimingConsts {
    feature: u32,
    pub(crate) fclk_can: u32,
    tseg1_min: u32,
    tseg1_max: u32,
    tseg2_min: u32,
    tseg2_max: u32,
    sjw_max: u32,
    brp_min: u32,
    brp_max: u32,
    brp_inc: u32,
}
impl BitTimingConsts {
    pub(crate) fn from_le_bytes(bs: &[u8]) -> BitTimingConsts {
        BitTimingConsts {
            feature: u32_from_le_bytes(&bs[0..4]),
            fclk_can: u32_from_le_bytes(&bs[4..8]),
            tseg1_min: u32_from_le_bytes(&bs[8..12]),
            tseg1_max: u32_from_le_bytes(&bs[12..16]),
            tseg2_min: u32_from_le_bytes(&bs[16..20]),
            tseg2_max: u32_from_le_bytes(&bs[20..24]),
            sjw_max: u32_from_le_bytes(&bs[24..28]),
            brp_min: u32_from_le_bytes(&bs[28..32]),
            brp_max: u32_from_le_bytes(&bs[32..36]),
            brp_inc: u32_from_le_bytes(&bs[36..40]),
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub(crate) struct DeviceConfig {
    reserved1: u8,
    reserved2: u8,
    reserved3: u8,
    pub(crate) icount: u8,
    pub(crate) sw_version: u32,
    pub(crate) hw_version: u32,
}
impl DeviceConfig {
    pub(crate) fn from_le_bytes(bs: &[u8]) -> DeviceConfig {
        DeviceConfig {
            reserved1: bs[0],
            reserved2: bs[1],
            reserved3: bs[2],
            icount: bs[3],
            sw_version: u32_from_le_bytes(&bs[4..8]),
            hw_version: u32_from_le_bytes(&bs[8..12]),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub(crate) struct HostFrame {
    pub echo_id: u32,
    pub can_id: u32,

    pub can_dlc: u8,
    pub channel: u8,
    pub flags: u8,
    pub reserved: u8,

    pub data: [u8; 8],
}
impl HostFrame {
    pub(crate) fn from_le_bytes(bs: &[u8]) -> HostFrame {
        HostFrame {
            echo_id: u32_from_le_bytes(&bs[0..4]),
            can_id: u32_from_le_bytes(&bs[4..8]),
            can_dlc: bs[8],
            channel: bs[9],
            flags: bs[10],
            reserved: bs[11],
            data: [
                bs[12], bs[13], bs[14], bs[15], bs[16], bs[17], bs[18], bs[19],
            ],
        }
    }
    pub(crate) fn to_le_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];
        data.extend_from_slice(&self.echo_id.to_le_bytes());
        data.extend_from_slice(&self.can_id.to_le_bytes());
        data.push(self.can_dlc);
        data.push(self.channel);
        data.push(self.flags);
        data.push(self.reserved);
        data.extend_from_slice(&self.data);
        data
    }
}
