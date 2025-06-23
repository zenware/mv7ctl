use rusb::{Context, DeviceHandle, Direction, Recipient, RequestType, UsbContext};

// Interface 0 is Audio Control
// Interface 1 is Audio Out/Monitor Mix
// Interface 2 is Audio In/Mono Microphone
// Interface 3 is HID ?in?
// Interface 4 is HID ?out?
const MV7_VID: u16 = 0x14ed;
const MV7_PID: u16 = 0x1012;
const MV7_CTL_IFACE: u8 = 0;

pub struct MV7Device {
    handle: DeviceHandle<Context>,
}

impl MV7Device {
    /// Open the first MV7 and claim it's HID interface.
    pub fn open() -> rusb::Result<Self> {
        let ctx = Context::new()?;
        let mut handle = ctx
            .open_device_with_vid_pid(MV7_VID, MV7_PID)
            .ok_or(rusb::Error::NoDevice)?;

        let iface = MV7_CTL_IFACE;
        if handle.kernel_driver_active(iface)? {
            handle.detach_kernel_driver(iface)?;
        }
        handle.claim_interface(iface)?;
        handle.set_alternate_setting(iface, 0)?;

        Ok(MV7Device { handle })
    }

    /// Set the mute toggle
    pub fn set_mute(&mut self, mute: bool) -> rusb::Result<usize> {
        let request_type =
            rusb::request_type(Direction::Out, RequestType::Class, Recipient::Interface);
        let request = 0x01; // SET_CUR
        let value = 0x0100; // MUTE_CONTROL << 8 | channel (0x00)
        // let iface = MV7_CTL_IFACE;
        // TODO: Get this working by actually calculating it, rather than just
        // using the number available in the pcap.
        let index = 0x0600; // 0x0300 | (iface as u16); // Feature Unit ID << 8 | Interface Number
        let mut data = [mute as u8];

        self.handle.write_control(
            request_type,
            request,
            value,
            index,
            &data,
            std::time::Duration::from_millis(100),
        )
    }
}
