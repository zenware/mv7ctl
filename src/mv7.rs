use rusb::{Context, DeviceHandle, Direction, Recipient, RequestType, UsbContext};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::time::Duration;

const MV7_VID: u16 = 0x14ed;
const MV7_PID: u16 = 0x1012;

const USBAUDIO_IFACE: u8 = 0;
const HID_IFACE: u8 = 3;

const HID_EP_OUT: u8 = 0x05;
const HID_EP_IN: u8 = 0x84;

#[repr(u8)]
enum ControlRequest {
    SetCur = 0x01,
    GetCur = 0x81,
    // .. others as needed
}

#[repr(u8)]
enum FeatureUnit {
    Mic = 0x06, // this bUnitID controls the microphone in pcaps
}

#[repr(u8)]
enum FeatureUnitControl {
    Mute = 0x01,
    // ... others as needed
}

/// These are "dspMode=n" values observed in packet captures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicPosition {
    Near = 2,
    Far = 5,
}

impl TryFrom<u8> for MicPosition {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            2 => Ok(MicPosition::Near),
            5 => Ok(MicPosition::Far),
            _ => Err(()),
        }
    }
}

pub struct MV7Device {
    handle: DeviceHandle<Context>,
    was_kernel_attached: HashMap<u8, bool>,
}

impl MV7Device {
    /// Open the first MV7 and claim it's HID interface.
    pub fn open() -> rusb::Result<Self> {
        let ctx = Context::new()?;
        let mut handle = ctx
            .open_device_with_vid_pid(MV7_VID, MV7_PID)
            .ok_or(rusb::Error::NoDevice)?;

        if handle.active_configuration()? != 1 {
            handle.set_active_configuration(1)?;
        }

        // We track whether interfaces had a kernel driver attached so they can
        // be reattached after we're done.
        let mut was_kernel_attached = HashMap::new();
        for &iface in &[USBAUDIO_IFACE, HID_IFACE] {
            let attached = handle.kernel_driver_active(iface)?;
            if attached {
                handle.detach_kernel_driver(iface)?;
            }
            handle.claim_interface(iface)?;
            handle.set_alternate_setting(iface, 0)?;
            was_kernel_attached.insert(iface, attached);
        }

        let mut device = MV7Device {
            handle,
            was_kernel_attached,
        };

        // attempt to clear any startup HID chatter
        std::thread::sleep(Duration::from_millis(100));
        device.drain_hid()?;

        Ok(device)
    }

    pub fn reset_device(mut self) -> rusb::Result<()> {
        println!("[INFO] Resetting MV7...");
        self.handle.reset()?;
        println!("[INFO] Device reset signal sent.");
        Ok(())
    }

    pub fn status(mut self) -> rusb::Result<()> {
        let muted = self.get_mute()?;
        let mic_position = self.get_mic_position()?;
        println!("MV7 Status:");
        println!("-----------");
        println!("Mute: {}", if muted { "On" } else { "Off" });
        println!("Position: {:?}", mic_position);
        Ok(())
    }

    /// Get mute status via USB Audio Class GET_CUR
    pub fn get_mute(&mut self) -> rusb::Result<bool> {
        let request_type =
            rusb::request_type(Direction::In, RequestType::Class, Recipient::Interface);
        let request = ControlRequest::GetCur as u8;
        let value = ((FeatureUnitControl::Mute as u16) << 8) | (USBAUDIO_IFACE as u16);
        let index = ((FeatureUnit::Mic as u16) << 8) | (USBAUDIO_IFACE as u16);
        let mut data = [0u8; 1];

        self.handle.read_control(
            request_type,
            request,
            value,
            index,
            &mut data,
            Duration::from_millis(100),
        )?;

        Ok(data[0] != 0)
    }

    /// Set the mute toggle
    pub fn set_mute(&mut self, mute: bool) -> rusb::Result<usize> {
        let request_type =
            rusb::request_type(Direction::Out, RequestType::Class, Recipient::Interface);
        let request = ControlRequest::SetCur as u8;
        let value = ((FeatureUnitControl::Mute as u16) << 8) | (USBAUDIO_IFACE as u16);
        let index = ((FeatureUnit::Mic as u16) << 8) | (USBAUDIO_IFACE as u16);
        let mut data = [mute as u8];

        self.handle.write_control(
            request_type,
            request,
            value,
            index,
            &data,
            Duration::from_millis(100),
        )
    }

    pub fn get_mic_position(&mut self) -> rusb::Result<MicPosition> {
        self.drain_hid()?;
        let mut cmd = b"dspMode\0".to_vec();
        cmd.resize(64, 0);
        self.handle
            .write_interrupt(HID_EP_OUT, &cmd, Duration::from_millis(100))?;

        let mut buf = [0u8; 64];
        let n = self
            .handle
            .read_interrupt(HID_EP_IN, &mut buf, Duration::from_millis(200))?;
        let response = std::str::from_utf8(&buf[..n]).unwrap_or("").trim();

        if let Some(pos) = response.strip_prefix("dspMode=") {
            let digit: u8 = pos
                .trim()
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .map_err(|_| rusb::Error::Other)?;
            return MicPosition::try_from(digit).map_err(|_| rusb::Error::Other);
        }

        Err(rusb::Error::Other)
    }

    /// Set mic position Near/Far
    pub fn set_mic_position(&mut self, position: MicPosition) -> rusb::Result<()> {
        self.drain_hid()?;
        let mut cmd = format!("dspMode {}\0", position as u8).into_bytes();
        cmd.resize(64, 0);
        self.handle
            .write_interrupt(HID_EP_OUT, &cmd, Duration::from_millis(100))?;
        // swallow ACK/confirmation
        let mut buf = [0u8; 64];

        // First response should contain somethign like "dspMode=<N>"
        // there may be a reason to verify this in the future, but fo
        let _ = self
            .handle
            .read_interrupt(HID_EP_IN, &mut buf, Duration::from_millis(200))?;
        Ok(())
    }

    fn drain_hid(&mut self) -> rusb::Result<()> {
        let mut buf = [0u8; 64];
        for _ in 0..5 {
            match self
                .handle
                .read_interrupt(HID_EP_IN, &mut buf, Duration::from_millis(50))
            {
                Ok(0) => continue,
                Ok(n) => {
                    let s = String::from_utf8_lossy(&buf[..n]);
                    println!("[DEBUG] Drained HID: {:?}", s.trim_end_matches('\0'));
                }
                Err(rusb::Error::Timeout) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl Drop for MV7Device {
    fn drop(&mut self) {
        for (&iface, &was_attached) in &self.was_kernel_attached {
            match self.handle.release_interface(iface) {
                Ok(()) => println!("[DEBUG] Released interface {}", iface),
                Err(e) => eprintln!("[WARN] Failed to release interface {}: {:?}", iface, e),
            }

            if was_attached {
                match self.handle.attach_kernel_driver(iface) {
                    Ok(_) => println!("[DEBUG] Reattached kernel driver to interface {}", iface),
                    Err(e) => eprintln!(
                        "[WARN] Failed to reattach kernel driver to interface {}: {:?}",
                        iface, e
                    ),
                }
            }
        }
    }
}
