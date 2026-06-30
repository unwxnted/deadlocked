use std::sync::Arc;

use utils::{channel::Channel, log::LoggerOptions, sync::Mutex};

use crate::{
    data::Data,
    os::{kernel_mem::check_kernel_module, uinput::check_uinput},
    ui::app::App,
};

mod config;
mod constants;
mod cs2;
mod data;
mod game;
mod math;
mod message;
mod obf;
mod os;
mod parser;
mod ui;

#[cfg(not(target_os = "linux"))]
compile_error!("only linux is supported.");

fn set_process_name(name: &str) {
    let cname = std::ffi::CString::new(name).unwrap();
    unsafe {
        libc::prctl(libc::PR_SET_NAME, cname.as_ptr() as *const libc::c_void);
    }
    if let Ok(mut cmdline) = std::fs::OpenOptions::new()
        .write(true)
        .open(crate::obfstr!("/proc/self/comm").decrypt())
    {
        use std::io::Write;
        let _ = writeln!(&mut cmdline, "{}", name);
    }
}

fn main() {
    set_process_name(crate::obfstr!("gdbus").decrypt().as_str());

    unsafe {
        libc::prctl(libc::PR_SET_DUMPABLE, 0);
    }

    let log_path = {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        std::path::PathBuf::from(home).join(".cache.log")
    };
    utils::log::init(
        LoggerOptions::default().file(log_path).truncate(true),
        |w, rec| writeln!(w, "[{}] {}", rec.level, rec.args),
    )
    .expect("failed to initialize logger");

    if !check_kernel_module() {
        return;
    }

    if !check_uinput() {
        return;
    }

    unsafe {
        std::env::remove_var(crate::obfstr!("WAYLAND_DISPLAY").decrypt().as_str());
    }

    let (channel_gui, channel_game) = Channel::new();
    let data = Arc::new(Mutex::new(Data::default()));
    let data_game = data.clone();

    std::thread::spawn(move || {
        game::GameManager::new(channel_game, data_game).run();
    });

    let event_loop = match winit::event_loop::EventLoop::new() {
        Ok(event_loop) => event_loop,
        Err(err) => {
            utils::error!("failed to create event loop: {err}");
            return;
        }
    };
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut app = App::new(channel_gui, data);
    event_loop.run_app(&mut app).unwrap();
}
