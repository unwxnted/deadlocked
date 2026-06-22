use std::{fs::File, io, os::unix::io::AsRawFd, path::Path};

use nix::libc;

#[repr(C)]
struct DeadlockedRW {
    target_pid: i32,
    addr: u64,
    size: u64,
    buf: *mut libc::c_void,
}

const IOC_READWRITE: u32 = 3;
const IOC_DIRSHIFT: u32 = 30;
const IOC_TYPESHIFT: u32 = 8;
const IOC_NRSHIFT: u32 = 0;
const IOC_SIZESHIFT: u32 = 16;

const fn ioc_rw(magic: u8, nr: u8, size: usize) -> u32 {
    (IOC_READWRITE << IOC_DIRSHIFT)
        | ((magic as u32) << IOC_TYPESHIFT)
        | ((nr as u32) << IOC_NRSHIFT)
        | ((size as u32) << IOC_SIZESHIFT)
}

// Both READ and WRITE use _IOWR since the struct is bidirectional
const IOCTL_DEADLOCKED_READ: u32 = ioc_rw(b'D', 1, std::mem::size_of::<DeadlockedRW>());
const IOCTL_DEADLOCKED_WRITE: u32 = ioc_rw(b'D', 2, std::mem::size_of::<DeadlockedRW>());

const MAX_TRANSFER: usize = 1_048_576;

#[derive(Debug)]
pub struct KernelMem {
    file: File,
}

impl KernelMem {
    pub fn open() -> io::Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .open("/dev/deadlocked")?;
        Ok(Self { file })
    }

    pub fn is_available() -> bool {
        Path::new("/dev/deadlocked").exists()
    }

    pub fn read(&self, pid: i32, addr: u64, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len();
        if len == 0 {
            return Ok(0);
        }

        if len > MAX_TRANSFER {
            return self.read_chunked(pid, addr, buf);
        }

        self.read_single(pid, addr, buf)
    }

    fn read_single(&self, pid: i32, addr: u64, buf: &mut [u8]) -> io::Result<usize> {
        let mut rw = DeadlockedRW {
            target_pid: pid,
            addr,
            size: buf.len() as u64,
            buf: buf.as_mut_ptr() as *mut libc::c_void,
        };

        let ret = unsafe {
            libc::ioctl(
                self.file.as_raw_fd(),
                IOCTL_DEADLOCKED_READ as libc::c_ulong,
                &mut rw as *mut DeadlockedRW,
            )
        };

        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    fn read_chunked(&self, pid: i32, mut addr: u64, buf: &mut [u8]) -> io::Result<usize> {
        let mut total = 0;
        let mut offset = 0;

        while offset < buf.len() {
            let chunk_size = (buf.len() - offset).min(MAX_TRANSFER);
            let chunk = &mut buf[offset..offset + chunk_size];
            let n = self.read_single(pid, addr, chunk)?;
            total += n;
            if n < chunk_size {
                break;
            }
            offset += n;
            addr += n as u64;
        }

        Ok(total)
    }

    pub fn write(&self, pid: i32, addr: u64, buf: &[u8]) -> io::Result<usize> {
        let len = buf.len();
        if len == 0 {
            return Ok(0);
        }

        if len > MAX_TRANSFER {
            return self.write_chunked(pid, addr, buf);
        }

        self.write_single(pid, addr, buf)
    }

    fn write_single(&self, pid: i32, addr: u64, buf: &[u8]) -> io::Result<usize> {
        let mut rw = DeadlockedRW {
            target_pid: pid,
            addr,
            size: buf.len() as u64,
            buf: buf.as_ptr() as *mut libc::c_void,
        };

        let ret = unsafe {
            libc::ioctl(
                self.file.as_raw_fd(),
                IOCTL_DEADLOCKED_WRITE as libc::c_ulong,
                &mut rw as *mut DeadlockedRW,
            )
        };

        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
    }

    fn write_chunked(&self, pid: i32, mut addr: u64, buf: &[u8]) -> io::Result<usize> {
        let mut total = 0;
        let mut offset = 0;

        while offset < buf.len() {
            let chunk_size = (buf.len() - offset).min(MAX_TRANSFER);
            let chunk = &buf[offset..offset + chunk_size];
            let n = self.write_single(pid, addr, chunk)?;
            total += n;
            if n < chunk_size {
                break;
            }
            offset += n;
            addr += n as u64;
        }

        Ok(total)
    }
}

pub fn check_deadlocked() -> bool {
    if !KernelMem::is_available() {
        utils::error!("the deadlocked kernel module is not loaded.");
        utils::error!("please run the setup script to build and load it.");
        return false;
    }
    if KernelMem::open().is_err() {
        utils::error!("user has no permissions for /dev/deadlocked.");
        utils::error!("did you run the setup script?");
        return false;
    }
    true
}
