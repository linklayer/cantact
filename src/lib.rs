use rusb;
use std::mem::size_of;
use std::time::Duration;

fn u32_from_le_bytes(bs: &[u8]) -> u32 {
    let arr: [u8; 4] = [bs[0], bs[1], bs[2], bs[3]];
    u32::from_le_bytes(arr)
}

#[repr(u8)]
pub enum gs_usb_breq {
    GS_USB_BREQ_HOST_FORMAT = 0,
    GS_USB_BREQ_BITTIMING,
    GS_USB_BREQ_MODE,
    GS_USB_BREQ_BERR,
    GS_USB_BREQ_BT_CONST,
    GS_USB_BREQ_DEVICE_CONFIG,
    GS_USB_BREQ_TIMESTAMP,
    GS_USB_BREQ_IDENTIFY,
}
#[repr(u8)]
pub enum gs_can_mode {
    /* reset a channel. turns it off */
    GS_CAN_MODE_RESET = 0,
    /* starts a channel */
    GS_CAN_MODE_START,
}

#[repr(u8)]
pub enum gs_can_state {
    GS_CAN_STATE_ERROR_ACTIVE = 0,
    GS_CAN_STATE_ERROR_WARNING,
    GS_CAN_STATE_ERROR_PASSIVE,
    GS_CAN_STATE_BUS_OFF,
    GS_CAN_STATE_STOPPED,
    GS_CAN_STATE_SLEEPING,
}

#[repr(u8)]
pub enum gs_can_identify_mode {
    GS_CAN_IDENTIFY_OFF = 0,
    GS_CAN_IDENTIFY_ON,
}

#[repr(C)]
pub struct gs_device_mode {
    pub mode: u32,
    pub flags: u32,
}
impl gs_device_mode {
    fn to_le_bytes(&self) -> Vec<u8> {
        let mut data: Vec<u8> = vec![];
        data.extend_from_slice(&self.mode.to_le_bytes());
        data.extend_from_slice(&self.flags.to_le_bytes());
        data
    }
}

#[repr(C)]
pub struct gs_device_bittiming {
    pub prop_seg: u32,
    pub phase_seg1: u32,
    pub phase_seg2: u32,
    pub sjw: u32,
    pub brp: u32,
}
impl gs_device_bittiming {
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
pub struct gs_device_bt_const {
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
impl gs_device_bt_const {
    fn from_le_bytes(bs: &[u8; 40]) -> gs_device_bt_const {
        gs_device_bt_const {
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

#[repr(C)]
#[derive(Debug)]
pub struct gs_host_frame {
    pub echo_id: u32,
    pub can_id: u32,

    pub can_dlc: u8,
    pub channel: u8,
    pub flags: u8,
    pub reserved: u8,

    pub data: [u8; 8],
}
impl gs_host_frame {
    fn from_le_bytes(bs: &[u8]) -> gs_host_frame {
        gs_host_frame {
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

pub struct Device {
    hnd: rusb::DeviceHandle<rusb::GlobalContext>,
}

impl Device {
    fn list_devices() {
        for device in rusb::devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();

            println!(
                "Bus {:03} Device {:03} ID {:04x}:{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id()
            );
        }
    }

    pub fn new() -> Option<Device> {
        let hnd = rusb::open_device_with_vid_pid(0x1d50, 0x606f);
        match hnd {
            Some(mut h) => {
                h.claim_interface(0).unwrap();
                Some(Device { hnd: h })
            }
            None => None,
        }
    }

    fn control_out(
        &self,
        req: gs_usb_breq,
        channel: u16,
        data: &[u8],
    ) -> Result<usize, rusb::Error> {
        let rt = rusb::request_type(
            rusb::Direction::Out,
            rusb::RequestType::Vendor,
            rusb::Recipient::Interface,
        );
        let timeout = Duration::from_millis(1000);
        self.hnd.write_control(
            rt,        // bmRequestType
            req as u8, // bRequest
            channel,   // wValue
            0,         // wIndex
            data,      // data
            timeout,   // timeout
        )
    }
    fn control_in(
        &self,
        req: gs_usb_breq,
        channel: u16,
        data: &mut [u8],
    ) -> Result<usize, rusb::Error> {
        let rt = rusb::request_type(
            rusb::Direction::In,
            rusb::RequestType::Vendor,
            rusb::Recipient::Interface,
        );
        let timeout = Duration::from_millis(1000);
        self.hnd.read_control(
            rt,        // bmRequestType
            req as u8, // bRequest
            channel,   // wValue
            0,         // wIndex
            data,      // data
            timeout,   // timeout
        )
    }
    pub fn set_mode(
        &self,
        channel: u16,
        device_mode: gs_device_mode,
    ) -> Result<usize, rusb::Error> {
        self.control_out(
            gs_usb_breq::GS_USB_BREQ_MODE,
            channel,
            &device_mode.to_le_bytes(),
        )
    }
    pub fn set_bit_timing(
        &self,
        channel: u16,
        timing: gs_device_bittiming,
    ) -> Result<usize, rusb::Error> {
        self.control_out(
            gs_usb_breq::GS_USB_BREQ_BITTIMING,
            channel,
            &timing.to_le_bytes(),
        )
    }
    pub fn get_bit_timing_consts(&self, channel: u16) -> Result<gs_device_bt_const, rusb::Error> {
        let mut buf: [u8; size_of::<gs_device_bt_const>()] = [0u8; size_of::<gs_device_bt_const>()];
        self.control_in(gs_usb_breq::GS_USB_BREQ_BT_CONST, channel, &mut buf)?;
        Ok(gs_device_bt_const::from_le_bytes(&buf))
    }

    pub fn get_frame(&self) -> Result<gs_host_frame, rusb::Error> {
        let mut buf: [u8; size_of::<gs_host_frame>()] = [0u8; size_of::<gs_host_frame>()];
        let timeout = Duration::from_millis(1000);
        let res = self.hnd.read_bulk(0x81, &mut buf, timeout);
        match res {
            Ok(_) => Ok(gs_host_frame::from_le_bytes(&buf)),
            Err(e) => return Err(e),
        }
    }
    pub fn send_frame(&self, frame: gs_host_frame) -> Result<(), rusb::Error> {
        let timeout = Duration::from_millis(1000);
        let res = self.hnd.write_bulk(0x2, &frame.to_le_bytes(), timeout)?;
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
    fn list_devices() {
        Device::list_devices();
    }
    #[test]
    fn set_mode() {
        let d = Device::new().unwrap();
        d.set_mode(
            0,
            gs_device_mode {
                mode: gs_can_mode::GS_CAN_MODE_START as u32,
                flags: 0,
            },
        )
        .unwrap();
    }
    #[test]
    fn set_timing() {
        let d = Device::new().unwrap();
        let bt = gs_device_bittiming {
            prop_seg: 0,
            phase_seg1: 13,
            phase_seg2: 2,
            sjw: 1,
            brp: 6,
        };
        d.set_bit_timing(0, bt).unwrap();
    }
    #[test]
    fn get_consts() {
        let d = Device::new().unwrap();
        let consts = d.get_bit_timing_consts(0).unwrap();
        println!("consts: {:?}", consts)
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
        let f = gs_host_frame {
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
