#![allow(dead_code)]

use rusb;
/// Implementation of CANtact USB device support using libusb via rusb.
/// This crate is not intended to be used by end users.
use std::mem::size_of;
use std::time::Duration;

const USB_VID: u16 = 0x1d50; //0x606f
const USB_PID: u16 = 0x606f; //0x606f

#[repr(u8)]
enum UsbBreq {
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
enum CanState {
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
    fn to_le_bytes(&self) -> Vec<u8> {
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
    fn to_le_bytes(&self) -> Vec<u8> {
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
    fclk_can: u32,
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
    fn from_le_bytes(bs: &[u8; 40]) -> BitTimingConsts {
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
    icount: u8,
    sw_version: u32,
    hw_version: u32,
}
impl DeviceConfig {
    fn from_le_bytes(bs: &[u8; 12]) -> DeviceConfig {
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
    fn from_le_bytes(bs: &[u8]) -> HostFrame {
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
    fn to_le_bytes(&self) -> Vec<u8> {
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

pub(crate) struct Device {
    hnd: rusb::DeviceHandle<rusb::GlobalContext>,
    timeout: Duration,
}

impl Device {
    pub(crate) fn new() -> Option<Device> {
        let hnd = rusb::open_device_with_vid_pid(USB_VID, USB_PID);
        match hnd {
            Some(mut h) => {
                h.claim_interface(0).unwrap();
                Some(Device {
                    hnd: h,
                    timeout: Duration::from_millis(1000),
                })
            }
            None => None,
        }
    }

    pub(crate) fn set_timeout(&mut self, d: Duration) {
        self.timeout = d;
    }

    fn control_out(&self, req: UsbBreq, channel: u16, data: &[u8]) -> Result<usize, rusb::Error> {
        let rt = rusb::request_type(
            rusb::Direction::Out,
            rusb::RequestType::Vendor,
            rusb::Recipient::Interface,
        );
        self.hnd.write_control(
            rt,           // bmRequestType
            req as u8,    // bRequest
            channel,      // wValue
            0,            // wIndex
            data,         // data
            self.timeout, // timeout
        )
    }
    fn control_in(
        &self,
        req: UsbBreq,
        channel: u16,
        data: &mut [u8],
    ) -> Result<usize, rusb::Error> {
        let rt = rusb::request_type(
            rusb::Direction::In,
            rusb::RequestType::Vendor,
            rusb::Recipient::Interface,
        );
        self.hnd.read_control(
            rt,           // bmRequestType
            req as u8,    // bRequest
            channel,      // wValue
            0,            // wIndex
            data,         // data
            self.timeout, // timeout
        )
    }

    pub(crate) fn set_host_format(&self, val: u32) -> Result<usize, rusb::Error> {
        let channel = 0;
        self.control_out(UsbBreq::HostFormat, channel, &val.to_le_bytes())
    }

    pub(crate) fn set_bit_timing(
        &self,
        channel: u16,
        timing: BitTiming,
    ) -> Result<usize, rusb::Error> {
        self.control_out(UsbBreq::BitTiming, channel, &timing.to_le_bytes())
    }

    pub(crate) fn set_mode(&self, channel: u16, device_mode: Mode) -> Result<usize, rusb::Error> {
        self.control_out(UsbBreq::Mode, channel, &device_mode.to_le_bytes())
    }

    pub(crate) fn set_identify(&self, val: u32) -> Result<usize, rusb::Error> {
        let channel = 0;
        self.control_out(UsbBreq::Identify, channel, &val.to_le_bytes())
    }

    pub(crate) fn set_berr(&self, val: u32) -> Result<usize, rusb::Error> {
        // TODO
        let channel = 0;
        self.control_out(UsbBreq::Berr, channel, &val.to_le_bytes())
    }

    pub(crate) fn get_device_config(&self) -> Result<DeviceConfig, rusb::Error> {
        let channel = 0;
        let mut buf: [u8; size_of::<DeviceConfig>()] = [0u8; size_of::<DeviceConfig>()];
        self.control_in(UsbBreq::DeviceConfig, channel, &mut buf)?;
        Ok(DeviceConfig::from_le_bytes(&buf))
    }

    pub(crate) fn get_bit_timing_consts(&self) -> Result<BitTimingConsts, rusb::Error> {
        let channel = 0;
        let mut buf: [u8; size_of::<BitTimingConsts>()] = [0u8; size_of::<BitTimingConsts>()];
        self.control_in(UsbBreq::BitTimingConsts, channel, &mut buf)?;
        Ok(BitTimingConsts::from_le_bytes(&buf))
    }

    pub(crate) fn get_timestamp(&self) -> Result<u32, rusb::Error> {
        let channel = 0;
        let mut buf: [u8; size_of::<u32>()] = [0u8; size_of::<u32>()];
        self.control_in(UsbBreq::Timestamp, channel, &mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub(crate) fn get_frame(&self) -> Result<HostFrame, rusb::Error> {
        let mut buf: [u8; size_of::<HostFrame>()] = [0u8; size_of::<HostFrame>()];
        let res = self.hnd.read_bulk(0x81, &mut buf, Duration::from_micros(1));
        match res {
            Ok(_) => Ok(HostFrame::from_le_bytes(&buf)),
            Err(e) => return Err(e),
        }
    }
    pub(crate) fn send_frame(&self, frame: HostFrame) -> Result<(), rusb::Error> {
        self.hnd
            .write_bulk(0x2, &frame.to_le_bytes(), self.timeout)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_device() {
        let d = Device::new().unwrap();
        println!("{:?}", d.hnd.device().device_descriptor());
    }
    #[test]
    fn set_host_format() {
        let d = Device::new().unwrap();
        d.set_host_format(0x0).unwrap();
    }
    #[test]
    fn set_bit_timing() {
        let d = Device::new().unwrap();
        let bt = BitTiming {
            prop_seg: 0,
            phase_seg1: 13,
            phase_seg2: 2,
            sjw: 1,
            brp: 6,
        };
        d.set_bit_timing(0, bt).unwrap();
    }
    #[test]
    fn set_mode() {
        let d = Device::new().unwrap();
        d.set_mode(
            0,
            Mode {
                mode: CanMode::Start as u32,
                flags: 0,
            },
        )
        .unwrap();
    }
    #[test]
    fn set_identify() {
        let d = Device::new().unwrap();
        d.set_identify(1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1000));
        d.set_identify(0).unwrap();
    }
    #[test]
    fn set_berr() {
        let d = Device::new().unwrap();
        d.set_berr(0x0).unwrap();
    }
    #[test]
    fn get_device_config() {
        let d = Device::new().unwrap();
        let consts = d.get_device_config().unwrap();
        println!("device config: {:?}", consts)
    }
    #[test]
    fn get_consts() {
        let d = Device::new().unwrap();
        let consts = d.get_bit_timing_consts().unwrap();
        println!("consts: {:?}", consts)
    }
    #[test]
    fn get_timestamp() {
        let d = Device::new().unwrap();
        let consts = d.get_timestamp().unwrap();
        println!("timestamp: {:?}", consts)
    }
    #[test]
    fn get_frame() {
        let d = Device::new().unwrap();
        let f = d.get_frame().unwrap();
        println!("frame: {:?}", f);
    }
    #[test]
    fn send_frame() {
        let d = Device::new().unwrap();
        let f = HostFrame {
            echo_id: 1,
            can_id: 0x456,
            can_dlc: 2,
            channel: 0,
            flags: 0,
            reserved: 0,
            data: [0xCA, 0xFE, 0, 0, 0, 0, 0, 0],
        };
        d.send_frame(f).unwrap();
    }
}
