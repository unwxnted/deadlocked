use std::{fs::File, io, os::unix::io::AsRawFd, path::Path};

use nix::libc;

#[repr(C)]
struct DeadlockedRW {
    target_pid: i32,
    addr: u64,
    size: u64,
    buf: *mut libc::c_void,
}

const CMD_PING: u32 = 0xDEAD0000;
const CMD_READ: u32 = 0xDEAD0001;
const CMD_WRITE: u32 = 0xDEAD0002;

const MAX_TRANSFER: usize = 1_048_576;

#[derive(Debug)]
pub struct KernelMem {
    file: File,
}

impl KernelMem {
    pub fn open() -> io::Result<Self> {
        let file = File::open(crate::obfstr!("/dev/null").decrypt())?;
        Ok(Self { file })
    }

    pub fn is_available() -> bool {
        Path::new(&crate::obfstr!("/dev/null").decrypt()).exists()
    }

    fn do_ioctl(&self, cmd: u32, op: &mut DeadlockedRW) -> io::Result<usize> {
        let ret = unsafe {
            libc::ioctl(
                self.file.as_raw_fd(),
                cmd as libc::c_ulong,
                op as *mut DeadlockedRW,
            )
        };

        if ret < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ret as usize)
        }
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

        self.do_ioctl(CMD_READ, &mut rw)
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

        self.do_ioctl(CMD_WRITE, &mut rw)
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

pub fn check_kernel_module() -> bool {
    if !KernelMem::is_available() {
        utils::error!("/dev/null is not available");
        return false;
    }

    let file = match File::open(crate::obfstr!("/dev/null").decrypt()) {
        Ok(f) => f,
        Err(e) => {
            utils::error!("failed to open /dev/null: {e}");
            return false;
        }
    };

    let mut rw = DeadlockedRW {
        target_pid: 0,
        addr: 0,
        size: 0,
        buf: std::ptr::null_mut(),
    };

    let ret = unsafe {
        libc::ioctl(
            file.as_raw_fd(),
            CMD_PING as libc::c_ulong,
            &mut rw as *mut DeadlockedRW,
        )
    };

    if ret == 0 {
        true
    } else {
        utils::error!("kernel module is not loaded or hook not active");
        false
    }
}
