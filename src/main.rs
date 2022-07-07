use std::convert::{TryFrom, TryInto};
use std::io::Write;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
enum Error {
    Io(std::io::Error),
    Usb(rusb::Error),
}

impl std::convert::From<std::io::Error> for Error {
    fn from(ioerr: std::io::Error) -> Self {
        Error::Io(ioerr)
    }
}

impl std::convert::From<rusb::Error> for Error {
    fn from(err: rusb::Error) -> Self {
        Error::Usb(err)
    }
}

pub struct CommandFactory {
    seq: Arc<Mutex<u16>>,
}

impl CommandFactory {
    const OPERATION_GET: u32 = 1;
    const OPERATION_SET: u32 = 2;

    pub fn new() -> CommandFactory {
        CommandFactory {
            seq: Arc::new(Mutex::new(0u16)),
        }
    }

    pub fn make_command(&mut self, opcode: u16, operation: u32, data: &[u8]) -> Vec<u8> {
        let len = 0x10 + u16::try_from(data.len()).unwrap();
        let seq = {
            let mut guard = self.seq.lock().unwrap();
            let previous = *guard;
            *guard = previous.overflowing_add(1).0;
            previous
        };
        let mut cmd = vec![0u8; len.try_into().unwrap()];
        cmd[0x00..=0x01].copy_from_slice(&len.to_le_bytes());
        cmd[0x04..=0x05].copy_from_slice(&opcode.to_le_bytes());
        cmd[0x06] = 0x10;
        cmd[0x07] = 0x99;
        cmd[0x08..=0x0b].copy_from_slice(&operation.to_le_bytes());
        cmd[0x0c..=0x0d].copy_from_slice(&seq.to_le_bytes());
        cmd[0x0e] = 0x10;
        cmd[0x0f] = 0x99;
        cmd[0x10..].copy_from_slice(data);
        cmd
    }

    pub fn make_reboot(&mut self) -> Vec<u8> {
        self.make_command(0x0001, Self::OPERATION_SET, &[])
    }

    pub fn make_set_state(&mut self, word1: u32) -> Vec<u8> {
        let mut data = [0u8; 4];
        data[0..=3].copy_from_slice(&word1.to_le_bytes());
        self.make_command(0x0002, Self::OPERATION_SET, &data)
    }

    pub fn make_get_source(&mut self) -> Vec<u8> {
        const GET_SOURCE_DATA: [u8; 8] = [0u8; 8];
        self.make_command(0x0003, Self::OPERATION_GET, &GET_SOURCE_DATA)
    }

    pub fn make_set_source(&mut self, audio_src: u32, video_src: u32) -> Vec<u8> {
        let mut data = [0u8; 8];
        data[0..=3].copy_from_slice(&audio_src.to_le_bytes());
        data[4..=7].copy_from_slice(&video_src.to_le_bytes());
        self.make_command(0x0003, Self::OPERATION_SET, &data)
    }

    pub fn make_set_brightness(&mut self, brightness: u32) -> Vec<u8> {
        let mut data = [0u8; 8];
        data[0..=3].copy_from_slice(&0u32.to_le_bytes());
        data[4..=7].copy_from_slice(&brightness.to_le_bytes());
        self.make_command(0x0101, Self::OPERATION_SET, &data)
    }

    pub fn make_set_contrast(&mut self, contrast: u32) -> Vec<u8> {
        let mut data = [0u8; 8];
        data[0..=3].copy_from_slice(&0u32.to_le_bytes());
        data[4..=7].copy_from_slice(&contrast.to_le_bytes());
        self.make_command(0x0102, Self::OPERATION_SET, &data)
    }

    pub fn make_set_hue(&mut self, hue: u32) -> Vec<u8> {
        let mut data = [0u8; 8];
        data[0..=3].copy_from_slice(&0u32.to_le_bytes());
        data[4..=7].copy_from_slice(&hue.to_le_bytes());
        self.make_command(0x0103, Self::OPERATION_SET, &data)
    }

    pub fn make_set_saturation(&mut self, saturation: u32) -> Vec<u8> {
        let mut data = [0u8; 8];
        data[0..=3].copy_from_slice(&0u32.to_le_bytes());
        data[4..=7].copy_from_slice(&saturation.to_le_bytes());
        self.make_command(0x0104, Self::OPERATION_SET, &data)
    }

    pub fn make_set_video_compression_keyframe_rate(&mut self, stream_idx: u32, rate: u32) -> Vec<u8> {
        let mut data = [0u8; 8];
        data[0..=3].copy_from_slice(&stream_idx.to_le_bytes());
        data[4..=7].copy_from_slice(&rate.to_le_bytes());
        self.make_command(0x0202, Self::OPERATION_SET, &data)
    }

    pub fn make_set_video_compression_quality(&mut self, stream_idx: u32, quality: u32) -> Vec<u8> {
        let mut data = [0u8; 8];
        data[0..=3].copy_from_slice(&stream_idx.to_le_bytes());
        data[4..=7].copy_from_slice(&quality.to_le_bytes());
        self.make_command(0x0203, Self::OPERATION_SET, &data)
    }

    pub fn make_get_firmware_status(&mut self) -> Vec<u8> {
        self.make_command(0x0008, Self::OPERATION_GET, &[])
    }

    pub fn make_get_profile(&mut self) -> Vec<u8> {
        self.make_command(0x000a, Self::OPERATION_GET, &[])
    }

    pub fn make_get_pc_grabber_small(&mut self) -> Vec<u8> {
        let dummy = [
            0x01u8, 0x40, 0x38, 0x38, 0x3c, 0xc6, 0xb0, 0x93, 0xba, 0xc1, 0xb0, 0x93,
        ];
        self.make_command(0xe001, Self::OPERATION_GET, &dummy)
    }

    pub fn make_set_pc_grabber_small(&mut self, enable: bool) -> Vec<u8> {
        let data: [u8; 0x0c] = [
            0x01, 0x40, 0x38, 0x38, 0x51, 0xd3, 0xcf, 0x77, if enable { 0x01 } else { 0x00 }, 0x00, 0x00, 0x00,
        ];
        self.make_command(0xe001, Self::OPERATION_SET, &data)
    }

    pub fn make_set_pc_grabber(&mut self, index: u32) -> Vec<u8> {
        let mut data: [u8; 0x3c] = [
            0x08, 0x20, 0x38, 0x38, 0x00, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x80, 0x07, 0x00, 0x00, 0x38, 0x04, 0x00, 0x00,
            0x10, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x1e, 0x00,
            0x00, 0x00, 0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];
        data[0xc..=0xf].copy_from_slice(&index.to_le_bytes());
        self.make_command(0xe001, Self::OPERATION_SET, &data)
    }

    pub fn make_set_pc_grabber_large(&mut self) -> Vec<u8> {
        let mut data: [u8; 0x200] = [
            0x00, 0x02, 0x00, 0x00, 0x01, 0xe0, 0x10, 0x99, 0x01, 0x00, 0x00, 0x00, 0x36, 0x00,
            0x10, 0x99, 0x02, 0x00, 0x38, 0x38, 0x3c, 0xc6, 0xb0, 0x93, 0xba, 0xc1, 0xb0, 0x93,
            0x00, 0x00, 0x00, 0x00, 0x28, 0x8b, 0x5d, 0x8a, 0x5d, 0x6b, 0xb0, 0x93, 0x74, 0xd0,
            0xcc, 0x84, 0xb8, 0x63, 0xdf, 0x84, 0xb8, 0x65, 0xdf, 0x84, 0x48, 0xce, 0xd8, 0x84,
            0x07, 0x00, 0x00, 0x00, 0x3c, 0xc6, 0xb0, 0x93, 0xae, 0xba, 0xb0, 0x93, 0x24, 0x8b,
            0x5d, 0x8a, 0x98, 0xc6, 0xb0, 0x93, 0xc0, 0xa8, 0x98, 0x84, 0x01, 0x00, 0x00, 0xc0,
            0x78, 0x8b, 0x5d, 0x8a, 0x21, 0x61, 0x22, 0x8d, 0x74, 0xd0, 0xcc, 0x84, 0xb8, 0x65,
            0xdf, 0x84, 0xb8, 0x63, 0xdf, 0x84, 0xac, 0xaa, 0x7f, 0x07, 0xd0, 0x12, 0x22, 0x8d,
            0x28, 0x00, 0x00, 0x00, 0x05, 0xce, 0xd8, 0x84, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00,
            0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00,
            0x3c, 0x8b, 0x5d, 0x8a, 0x00, 0x00, 0x00, 0x00, 0xc0, 0x8c, 0x5d, 0x8a, 0xea, 0x0a,
            0x22, 0x8d, 0xd4, 0x3b, 0x00, 0x00, 0xfe, 0xff, 0xff, 0xff, 0xac, 0x8b, 0x5d, 0x8a,
            0x85, 0x5a, 0x22, 0x8d, 0x48, 0xce, 0xd8, 0x84, 0x05, 0x00, 0x00, 0x00, 0xb0, 0x38,
            0xcb, 0x95, 0xb8, 0x63, 0xdf, 0x84, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0xe8, 0xe2, 0xd8, 0x84, 0x48, 0xce, 0xd8, 0x84, 0x48, 0xce,
            0xd8, 0x84, 0x25, 0x02, 0x00, 0xc0, 0xd4, 0x8b, 0x5d, 0x8a, 0x43, 0x6c, 0x22, 0x8d,
            0x48, 0xce, 0xd8, 0x84, 0x60, 0x38, 0xcb, 0x95, 0x30, 0x52, 0xd8, 0x84, 0x38, 0x52,
            0xd8, 0x84, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe8, 0xe2, 0xd8, 0x84,
            0x08, 0xd0, 0xcc, 0x84, 0xe4, 0x8b, 0x5d, 0x8a, 0x8f, 0x54, 0x22, 0x8d, 0x70, 0x5c,
            0x3e, 0x84, 0x48, 0xce, 0xd8, 0x84, 0xfc, 0x8b, 0x5d, 0x8a, 0xba, 0x50, 0x21, 0x8d,
            0x70, 0x5c, 0x3e, 0x84, 0x48, 0xce, 0xd8, 0x84, 0x70, 0x5c, 0x3e, 0x84, 0x00, 0x00,
            0x00, 0x00, 0x14, 0x8c, 0x5d, 0x8a, 0x47, 0x20, 0x83, 0x82, 0x70, 0x5c, 0x3e, 0x84,
            0x48, 0xce, 0xd8, 0x84, 0x48, 0xce, 0xd8, 0x84, 0x70, 0x5c, 0x3e, 0x84, 0x34, 0x8c,
            0x5d, 0x8a, 0xd5, 0x89, 0xa0, 0x82, 0xe8, 0xe2, 0xd8, 0x84, 0x48, 0xce, 0xd8, 0x84,
            0x48, 0xcf, 0xd8, 0x84, 0xb4, 0x01, 0x00, 0x00, 0x8c, 0x8c, 0x5d, 0x04, 0x44, 0x8c,
            0x5d, 0x8a, 0xd0, 0x8c, 0x5d, 0x8a, 0xc8, 0xad, 0xa0, 0x82, 0x70, 0x5c, 0x3e, 0x84,
            0xe8, 0xe2, 0xd8, 0x84, 0x00, 0x00, 0x00, 0x00, 0x01, 0xf1, 0xa4, 0x82, 0x00, 0x7a,
            0x6b, 0x20, 0x02, 0x00, 0x00, 0x00, 0xf4, 0x7d, 0x6b, 0x20, 0x44, 0x04, 0x00, 0x00,
            0xc8, 0xfb, 0x25, 0x09, 0x73, 0x1d, 0xa1, 0x82, 0x00, 0x00, 0x00, 0x00, 0x9f, 0x01,
            0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0xe8, 0xe2, 0xd8, 0x84,
            0x00, 0x00, 0x00, 0x00, 0xd5, 0x74, 0xa5, 0x08, 0xc0, 0x0a, 0xd8, 0x84, 0x84, 0x75,
            0xa5, 0x82, 0x01, 0x8e, 0x8b, 0x82, 0xc8, 0xf5, 0x42, 0x84, 0x10, 0x00, 0x00, 0x00,
            0xa4, 0x8c, 0x5d, 0x8a, 0x30, 0xfc, 0x25, 0x09, 0x00, 0x7a, 0x6b, 0x20, 0x03, 0x00,
            0x00, 0x00, 0x01, 0xf1, 0xa4, 0x82, 0xc8, 0xf5, 0x42, 0x84, 0xe8, 0xe2, 0xd8, 0x84,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x54, 0x8c, 0x5d, 0x8a, 0x18, 0x8d,
            0x5d, 0x8a, 0xff, 0xff, 0xff, 0xff, 0x0b, 0x8e, 0x8b, 0x82, 0x7c, 0xf2, 0xb3, 0x28,
            0xfe, 0xff, 0xff, 0xff, 0x04, 0x8d, 0x5d, 0x8a,
        ];
        self.make_command(0xe001, Self::OPERATION_SET, &data)
    }

    pub fn make_time_query(&mut self, ts: u32) -> Vec<u8> {
        let mut data = [0u8; 4];
        data[0..=3].copy_from_slice(&ts.to_le_bytes());
        self.make_command(0xf001, Self::OPERATION_GET, &data)
    }

    pub fn make_get_hw_grabber(&mut self) -> Vec<u8> {
        self.make_command(0xf002, Self::OPERATION_GET, &[])
    }
}

impl Clone for CommandFactory {
    fn clone(&self) -> Self {
        Self {
            seq: self.seq.clone(),
        }
    }
}

fn print_resp_data(datatype: &str, data: &[u8]) {
    if data.len() < 0x10 {
        eprintln!("{}: Short response!", datatype);
        return;
    }
    if data.len() == 0x10 {
        eprintln!("{}: No data", datatype);
        return;
    }
    eprintln!("{}: {:02x?}", datatype, &data[0x10..data.len()]);
}

fn wait_pc_grabber_ready(
    devhnd: Arc<Mutex<rusb::DeviceHandle<rusb::GlobalContext>>>,
    factory: &mut CommandFactory,
) -> Result<(), Error> {
    const USB_TIMEOUT: Duration = Duration::from_secs(2);
    loop {
        let mut respbuf = [0u8; 0x200];
        let recvd = {
            let devhnd = devhnd.lock().unwrap();
            devhnd.write_bulk(2, &factory.make_get_pc_grabber_small(), USB_TIMEOUT)?;
            devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?
        };
        print_resp_data("PC grabber state", &respbuf[0..recvd]);
        if recvd == 0x1c && respbuf[0x18] == 0x01 {
            break;
        };
        thread::sleep(Duration::from_secs(1));
    }
    Ok(())
}

fn timer_thread(
    devhnd: Arc<Mutex<rusb::DeviceHandle<rusb::GlobalContext>>>,
    mut factory: CommandFactory,
) {
    use std::time::Instant;

    const USB_TIMEOUT: Duration = Duration::from_secs(5);
    let mut respbuf = [0u8; 0x200];
    let mut ts = 0u32;
    let mut now = Instant::now();
    let mut last = now;

    loop {
        ts = ts
            .overflowing_add(u32::try_from(now.duration_since(last).as_millis()).unwrap())
            .0;
        last = now;
        let recvd = {
            let devhnd = devhnd.lock().unwrap();
            let res = devhnd.write_bulk(2, &factory.make_time_query(ts), USB_TIMEOUT);
            if res.is_err() {
                eprintln!("Failed to write timestamp request: {}", &res.unwrap_err());
                continue;
            }
            let res = devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT);
            if res.is_err() {
                eprintln!(
                    "Failed to read timestamp request response: {}",
                    &res.unwrap_err()
                );
                continue;
            }
            res.unwrap()
        };
        print_resp_data("Remote timestamp", &respbuf[0..recvd]);
        thread::sleep(Duration::from_secs(10));
        now = Instant::now();
    }
}

fn main() -> Result<(), Error> {
    const USB_TIMEOUT: Duration = Duration::from_secs(2);
    let devhnd = if let Some(hnd) = rusb::open_device_with_vid_pid(0x048d, 0x9910) {
        Arc::new(Mutex::new(hnd))
    } else {
        println!("No device found.");
        exit(1);
    };

    {
        let mut devhnd = devhnd.lock().unwrap();
        devhnd.reset()?;
        devhnd.claim_interface(0)?;
        devhnd.set_alternate_setting(0, 0)?;
        devhnd.clear_halt(0x81)?;
        devhnd.clear_halt(0x83)?;
    }

    let mut respbuf = [0u8; 0x200];
    let mut factory = CommandFactory::new();
    let recvd = {
        let devhnd = devhnd.lock().unwrap();
        devhnd.write_bulk(2, &factory.make_get_profile(), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?
    };
    print_resp_data("Profile", &respbuf[0..recvd]);
    //let timer_hnd = {
    //    let factory = factory.clone();
    //    let devhnd = devhnd.clone();
    //    thread::spawn(move || {
    //        timer_thread(devhnd, factory);
    //    })
    //};
    let recvd = {
        let devhnd = devhnd.lock().unwrap();
        devhnd.write_bulk(2, &factory.make_get_source(), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?
    };
    print_resp_data("Source", &respbuf[0..recvd]);
    //    let recvd = {
    //        let devhnd = devhnd.lock().unwrap();
    //        devhnd.write_bulk(2, &factory.make_get_firmware_status(), USB_TIMEOUT)?;
    //        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?
    //    };
    //    print_resp_data("Firmware status", &respbuf[0..recvd]);
    //    eprintln!("Setting initial PC grabber...");

    let recvd = {
        let devhnd = devhnd.lock().unwrap();
        devhnd.write_bulk(2, &factory.make_set_pc_grabber_small(false), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?
    };
    print_resp_data("Returned PC grabber state", &respbuf[0..recvd]);

    // Alter some settings _before_ starting capture
    /*{
        let devhnd = devhnd.lock().unwrap();
        devhnd.write_bulk(2, &factory.make_set_brightness(0), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?;
        devhnd.write_bulk(2, &factory.make_set_contrast(100), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?;
        devhnd.write_bulk(2, &factory.make_set_hue(0), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?;
        devhnd.write_bulk(2, &factory.make_set_saturation(100), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?;
    }*/

    let recvd = {
        let devhnd = devhnd.lock().unwrap();
        devhnd.write_bulk(2, &factory.make_set_pc_grabber_small(true), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?
    };
    print_resp_data("Returned PC grabber state", &respbuf[0..recvd]);
    eprintln!("Waiting for PC grabber...");
    wait_pc_grabber_ready(devhnd.clone(), &mut factory)?;
    eprintln!("Setting PC grabber state...");
    for i in 0u32..=21u32 {
        let recvd = {
            let devhnd = devhnd.lock().unwrap();
            devhnd.write_bulk(2, &factory.make_set_pc_grabber(i), USB_TIMEOUT)?;
            devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?
        };
    }
    eprintln!("Starting capture...");
    let recvd = {
        let devhnd = devhnd.lock().unwrap();
        devhnd.write_bulk(2, &factory.make_set_state(0x2), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?
    };
    print_resp_data("State", &respbuf[0..recvd]);
    let recvd = {
        let devhnd = devhnd.lock().unwrap();
        devhnd.write_bulk(2, &factory.make_set_pc_grabber_large(), USB_TIMEOUT)?;
        devhnd.read_bulk(0x81, &mut respbuf, USB_TIMEOUT)?
    };

    loop {
        const TS_TIMEOUT: Duration = Duration::from_secs(1);
        let mut tsbuf = vec![0u8; 0x4000];
        let recvd = {
            let mut devhnd = devhnd.lock().unwrap();
            let res = devhnd.read_bulk(0x83, &mut tsbuf, TS_TIMEOUT);
            match res {
                Err(rusb::Error::Timeout) => {
                    eprintln!("Timeout");
                    continue;
                },
                Err(e) => {
                    eprintln!("Failed to read TS stream: {}", &e);
                    break;
                }
                Ok(len) => len
            }
        };
        {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(&tsbuf[..recvd])?;
        }
    }
    Ok(())
}
