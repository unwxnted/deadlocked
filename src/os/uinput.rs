use std::{
    fs::File,
    io::Write,
    os::fd::AsRawFd,
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use bytemuck::{Pod, Zeroable, cast_slice};
use glam::{IVec2, Vec2};
use nix::{ioctl_none, ioctl_write_int, ioctl_write_ptr, libc::c_ulong};
use rand::seq::IndexedRandom;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Timeval {
    seconds: u64,
    microseconds: u64,
}

unsafe impl Zeroable for Timeval {}
unsafe impl Pod for Timeval {}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct InputEvent {
    time: Timeval,
    event_type: u16,
    code: u16,
    value: i32,
}

unsafe impl Zeroable for InputEvent {}
unsafe impl Pod for InputEvent {}

#[repr(C)]
struct DeviceSetup {
    id: InputId,
    name: [u8; 80],
    ff_effects_max: u32,
}

#[repr(C)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

struct DeviceProfile {
    vendor: u16,
    product: u16,
    name: &'static str,
}

const MOUSE_PROFILES: &[DeviceProfile] = &[
    DeviceProfile {
        vendor: 0x046d,
        product: 0xc077,
        name: "Logitech G203",
    },
    DeviceProfile {
        vendor: 0x046d,
        product: 0xc092,
        name: "Logitech G Pro",
    },
    DeviceProfile {
        vendor: 0x046d,
        product: 0xc084,
        name: "Logitech G403",
    },
    DeviceProfile {
        vendor: 0x04d9,
        product: 0xfc2b,
        name: "USB Optical Mouse",
    },
    DeviceProfile {
        vendor: 0x1532,
        product: 0x006e,
        name: "Razer DeathAdder V2",
    },
    DeviceProfile {
        vendor: 0x1532,
        product: 0x0090,
        name: "Razer Viper",
    },
    DeviceProfile {
        vendor: 0x04b4,
        product: 0x1001,
        name: "HID-compliant mouse",
    },
    DeviceProfile {
        vendor: 0x093a,
        product: 0x2510,
        name: "USB Optical Mouse",
    },
    DeviceProfile {
        vendor: 0x1bcf,
        product: 0x0005,
        name: "USB Optical Mouse",
    },
    DeviceProfile {
        vendor: 0x413c,
        product: 0x2107,
        name: "Dell USB Mouse",
    },
    DeviceProfile {
        vendor: 0x046d,
        product: 0xc07d,
        name: "Logitech G502",
    },
    DeviceProfile {
        vendor: 0x046d,
        product: 0xc31c,
        name: "Logitech G903",
    },
    DeviceProfile {
        vendor: 0x03f0,
        product: 0x034a,
        name: "HP USB Mouse",
    },
    DeviceProfile {
        vendor: 0x04f2,
        product: 0x1125,
        name: "Chicony USB Mouse",
    },
];

const KEYBOARD_PROFILES: &[DeviceProfile] = &[
    DeviceProfile {
        vendor: 0x04d9,
        product: 0xfc2b,
        name: "USB Keyboard",
    },
    DeviceProfile {
        vendor: 0x258a,
        product: 0x0049,
        name: "Dell USB Keyboard",
    },
    DeviceProfile {
        vendor: 0x1c4f,
        product: 0x0002,
        name: "SIGMACHIP USB Keyboard",
    },
    DeviceProfile {
        vendor: 0x413c,
        product: 0x2107,
        name: "Dell USB Entry Keyboard",
    },
    DeviceProfile {
        vendor: 0x03f0,
        product: 0x034a,
        name: "HP USB Keyboard",
    },
    DeviceProfile {
        vendor: 0x04f2,
        product: 0x1125,
        name: "Chicony USB Keyboard",
    },
    DeviceProfile {
        vendor: 0x046d,
        product: 0xc31c,
        name: "Logitech G903 Keyboard",
    },
    DeviceProfile {
        vendor: 0x04b4,
        product: 0x1001,
        name: "HID Keyboard",
    },
    DeviceProfile {
        vendor: 0x0c45,
        product: 0x7603,
        name: "USB Keyboard",
    },
    DeviceProfile {
        vendor: 0x04d9,
        product: 0x1603,
        name: "USB Keyboard",
    },
    DeviceProfile {
        vendor: 0x1a2c,
        product: 0x0e24,
        name: "USB Keyboard",
    },
];

const UINPUT_IOCTL_BASE: c_ulong = b'U' as c_ulong;
ioctl_none!(ui_dev_create, UINPUT_IOCTL_BASE, 1);
ioctl_none!(ui_dev_destroy, UINPUT_IOCTL_BASE, 2);
ioctl_write_int!(ui_set_evbit, UINPUT_IOCTL_BASE, 100);
ioctl_write_int!(ui_set_keybit, UINPUT_IOCTL_BASE, 101);
ioctl_write_int!(ui_set_relbit, UINPUT_IOCTL_BASE, 102);
ioctl_write_ptr!(ui_dev_setup, UINPUT_IOCTL_BASE, 3, DeviceSetup);

const EV_SYN: u16 = 0x00;
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;
const SYN_REPORT: u16 = 0x00;
const AXIS_X: u16 = 0x00;
const AXIS_Y: u16 = 0x01;
const BTN_LEFT: u16 = 0x110;
const KEY_SPACE: u16 = 57;
const KEY_ESC: u16 = 1;
const KEY_ENTER: u16 = 28;
const KEY_A: u16 = 30;
const KEY_D: u16 = 32;
const KEY_Z: u16 = 44;

struct VirtualDevice {
    file: File,
}

pub struct InputDevice {
    mouse: VirtualDevice,
    keyboard: VirtualDevice,
}

static CREATED: AtomicBool = AtomicBool::new(false);

fn select_mouse_profile() -> &'static DeviceProfile {
    let mut rng = rand::rng();
    MOUSE_PROFILES.choose(&mut rng).unwrap()
}

fn select_keyboard_profile() -> &'static DeviceProfile {
    let mut rng = rand::rng();
    KEYBOARD_PROFILES.choose(&mut rng).unwrap()
}

fn build_device_setup(profile: &DeviceProfile) -> DeviceSetup {
    let mut name = [0u8; 80];
    let bytes = profile.name.as_bytes();
    let len = bytes.len().min(79);
    name[..len].copy_from_slice(&bytes[..len]);
    DeviceSetup {
        id: InputId {
            bustype: 0x03,
            vendor: profile.vendor,
            product: profile.product,
            version: 2,
        },
        name,
        ff_effects_max: 0,
    }
}

impl InputDevice {
    pub fn open() -> Result<Self, String> {
        if CREATED.swap(true, Ordering::Relaxed) {
            return Err(crate::obfstr!("input device already initialized").decrypt());
        }

        let mouse = VirtualDevice::open_mouse().inspect_err(|_| {
            CREATED.store(false, Ordering::Relaxed);
        })?;
        let keyboard = VirtualDevice::open_keyboard().inspect_err(|_| {
            CREATED.store(false, Ordering::Relaxed);
        })?;

        Ok(Self { mouse, keyboard })
    }

    pub fn move_rel(&mut self, coords: Vec2) {
        let coords = IVec2::new(coords.x as i32, coords.y as i32);
        if coords == IVec2::ZERO {
            return;
        }

        let time = Self::time_now();
        let events = [
            InputEvent {
                time,
                event_type: EV_REL,
                code: AXIS_X,
                value: coords.x,
            },
            InputEvent {
                time,
                event_type: EV_REL,
                code: AXIS_Y,
                value: coords.y,
            },
            Self::syn(time),
        ];

        self.mouse.write_events(&events);
    }

    pub fn left_press(&mut self) {
        self.mouse.key(BTN_LEFT, 1);
    }

    pub fn left_release(&mut self) {
        self.mouse.key(BTN_LEFT, 0);
    }

    pub fn space_press(&mut self) {
        self.keyboard.key(KEY_SPACE, 1);
    }

    pub fn space_release(&mut self) {
        self.keyboard.key(KEY_SPACE, 0);
    }

    pub fn a_press(&mut self) {
        self.keyboard.key(KEY_A, 1);
    }

    pub fn a_release(&mut self) {
        self.keyboard.key(KEY_A, 0);
    }

    pub fn d_press(&mut self) {
        self.keyboard.key(KEY_D, 1);
    }

    pub fn d_release(&mut self) {
        self.keyboard.key(KEY_D, 0);
    }

    fn syn(time: Timeval) -> InputEvent {
        InputEvent {
            time,
            event_type: EV_SYN,
            code: SYN_REPORT,
            value: 0,
        }
    }

    fn time_now() -> Timeval {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Timeval {
            seconds: now.as_secs(),
            microseconds: now.subsec_micros() as u64,
        }
    }
}

impl VirtualDevice {
    fn open_mouse() -> Result<Self, String> {
        let uinput_path = crate::obfstr!("/dev/uinput").decrypt();
        let file = File::options()
            .write(true)
            .open(&uinput_path)
            .map_err(|e| e.to_string())?;
        let fd = file.as_raw_fd();

        let profile = select_mouse_profile();
        let setup = build_device_setup(profile);

        unsafe {
            ui_set_evbit(fd, EV_SYN as u64).map_err(|e| e.to_string())?;
            ui_set_evbit(fd, EV_KEY as u64).map_err(|e| e.to_string())?;
            ui_set_evbit(fd, EV_REL as u64).map_err(|e| e.to_string())?;

            ui_set_relbit(fd, AXIS_X as u64).map_err(|e| e.to_string())?;
            ui_set_relbit(fd, AXIS_Y as u64).map_err(|e| e.to_string())?;
            ui_set_keybit(fd, BTN_LEFT as u64).map_err(|e| e.to_string())?;

            ui_dev_setup(fd, &setup).map_err(|e| e.to_string())?;
            ui_dev_create(fd).map_err(|e| e.to_string())?;
        }

        Ok(Self { file })
    }

    fn open_keyboard() -> Result<Self, String> {
        let uinput_path = crate::obfstr!("/dev/uinput").decrypt();
        let file = File::options()
            .write(true)
            .open(&uinput_path)
            .map_err(|e| e.to_string())?;
        let fd = file.as_raw_fd();

        let profile = select_keyboard_profile();
        let setup = build_device_setup(profile);

        unsafe {
            ui_set_evbit(fd, EV_SYN as u64).map_err(|e| e.to_string())?;
            ui_set_evbit(fd, EV_KEY as u64).map_err(|e| e.to_string())?;

            for code in [KEY_ESC, KEY_ENTER, KEY_SPACE, KEY_A, KEY_D, KEY_Z] {
                ui_set_keybit(fd, code as u64).map_err(|e| e.to_string())?;
            }

            ui_dev_setup(fd, &setup).map_err(|e| e.to_string())?;
            ui_dev_create(fd).map_err(|e| e.to_string())?;
        }

        Ok(Self { file })
    }

    fn key(&mut self, code: u16, pressed: i32) {
        let time = InputDevice::time_now();
        let events = [
            InputEvent {
                time,
                event_type: EV_KEY,
                code,
                value: pressed,
            },
            InputDevice::syn(time),
        ];

        self.write_events(&events);
    }

    fn write_events(&mut self, events: &[InputEvent]) {
        self.file.write_all(cast_slice(events)).unwrap();
    }
}

impl Drop for VirtualDevice {
    fn drop(&mut self) {
        let _ = unsafe { ui_dev_destroy(self.file.as_raw_fd()) };
    }
}

impl Drop for InputDevice {
    fn drop(&mut self) {
        CREATED.store(false, Ordering::Relaxed);
    }
}

pub fn check_uinput() -> bool {
    let uinput_path = crate::obfstr!("/dev/uinput").decrypt();
    let path = std::path::Path::new(&uinput_path);
    if !path.exists() {
        utils::error!("the uinput kernel module is not loaded.");
        utils::error!("this module needs to be loaded for input emulation to work.");
        utils::error!("please carefully read the readme before using.");
        return false;
    }
    if File::options().write(true).open(path).is_err() {
        utils::error!("user has no write permissions for /dev/uinput.");
        utils::error!("did you run the setup script?");
        utils::error!("please carefully read the readme before using.");
        return false;
    }
    true
}
