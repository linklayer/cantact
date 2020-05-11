use rusb;
use std::time::Duration;

#[repr(u8)]
enum gs_usb_breq {
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
enum gs_can_mode {
	/* reset a channel. turns it off */
	GS_CAN_MODE_RESET = 0,
	/* starts a channel */
	GS_CAN_MODE_START
}

#[repr(u8)]
enum gs_can_state {
	GS_CAN_STATE_ERROR_ACTIVE = 0,
	GS_CAN_STATE_ERROR_WARNING,
	GS_CAN_STATE_ERROR_PASSIVE,
	GS_CAN_STATE_BUS_OFF,
	GS_CAN_STATE_STOPPED,
	GS_CAN_STATE_SLEEPING
}

#[repr(u8)]
enum gs_can_identify_mode {
	GS_CAN_IDENTIFY_OFF = 0,
	GS_CAN_IDENTIFY_ON
}

struct Device {
    hnd: rusb::DeviceHandle<rusb::GlobalContext>,
}

impl Device {
    fn list_devices() {
        for device in rusb::devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();

            println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id());
        }
    }
    
    pub fn new() -> Option<Device> {
        let hnd = rusb::open_device_with_vid_pid(0,0);
        match hnd {
            Some(h) => Some(Device{hnd: h}),
            None => None
        }
    }

    fn control_out(&self, req: gs_usb_breq, channel: u16, data: &[u8]) -> Result<usize, rusb::Error> {
        let rt = rusb::request_type(
            rusb::Direction::Out, 
            rusb::RequestType::Vendor, 
            rusb::Recipient::Interface);
        let timeout = Duration::from_millis(1000);
        self.hnd.write_control(
            rt,         // bmRequestType 
            req as u8,  // bRequest
            channel,    // wValue 
            0,          // wIndex
            data,       // data 
            timeout     // timeout
        )
    }
    fn control_in(&self, req: gs_usb_breq, channel: u16, data: &mut [u8]) -> Result<usize, rusb::Error> {
        let rt = rusb::request_type(
            rusb::Direction::In, 
            rusb::RequestType::Vendor, 
            rusb::Recipient::Interface);
        let timeout = Duration::from_millis(1000);
        self.hnd.write_control(
            rt,         // bmRequestType 
            req as u8,  // bRequest
            channel,    // wValue 
            0,          // wIndex
            data,       // data 
            timeout     // timeout
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn new_device() {
        Device::new().unwrap();
    }
}
