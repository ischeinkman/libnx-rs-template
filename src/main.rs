#![allow(dead_code)]


extern crate libnx_rs;
use libnx_rs::libnx::*;
use libnx_rs::{service, LibnxError};
use libnx_rs::ipc::{IpcCommandHeader, RawIpcArgs};
use libnx_rs::console::ConsoleHandle;
extern crate libc;


use std::result::Result;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::io;
use std::os::unix::io::AsRawFd;
use std::panic;
use std::ptr;


#[cfg(feature="sysmodule-test")]
mod sysmodule_example;

#[cfg(feature="sysmodule-test")]
pub fn main() {
    sysmodule_example::example();
}


#[cfg(feature="sysmodule-verify")]
pub fn main() -> Result<(), LibnxError> {
    let mut sm_ctx = service::SmContext::initialize()?;
        
    let mut logfile = OpenOptions::new()
        .append(true).create(true).create_new(false)
        .open("libnx_rs_sysmodule_test_example.txt")
        .map_err(|e| LibnxError::from_msg(format!("Error opening sysmodule log: {:?}", e)))?;
    
    let mut console = ConsoleHandle::init_default();
    writeln!(logfile, "Verifying sysmodule installation.");
    println!(         "Verifying sysmodule installation.");
    
    let mut added_service = match sm_ctx.get_service("lnxrs") {
        Ok(a) => a, 
        Err(e) => {
            writeln!(logfile, "Could not find handle: {:?}", e);
            println!(         "Could not find handle: {:?}", e);
            return Err(e);
        }
    };
    let args = IpcCommandHeader::with_args(RawIpcArgs::new(vec![0xFFFFFFFF, 0xafafbba, 1, u32::max_value()]));
    unsafe {
        match added_service.handle().dispatch_command(args) {
            Ok(_) => {}, 
            Err(e) => {
                writeln!(logfile, "Could not dispatch command: {:?}", e);
                println!(         "Could not dispatch command: {:?}", e);
                return Err(e);
            }

        }
    }

    Ok(())
}

#[cfg(not(any(
    feature="sysmodule-test", 
    feature="sysmodule-verify",
    feature="conrod-test"
)))]
pub fn main() {
    example();
}

pub fn redirect_stdout (filename : &str) -> Result<File, io::Error> {
    let mut outfile = OpenOptions::new()
        .write(true)
        .create(true)
        .open(filename)?;
    outfile.write_fmt(format_args!("Redirecting standard output to {}.", filename))?;
    let raw_fd = outfile.as_raw_fd();
    let new_fd = unsafe {
        libc::fflush(0 as *mut libc::FILE);
        libc::dup2(raw_fd, libc::STDOUT_FILENO)
    };
    if new_fd != libc::STDOUT_FILENO {
        Err(io::Error::new(io::ErrorKind::Other, format!("Could not call dup2. Ended up redirecting fd {} to {} instead of {}.", raw_fd, new_fd, libc::STDOUT_FILENO)))
    }
    else { 
        Ok(outfile) 
    }
}

pub fn redirect_stderr (filename : &str) -> Result<File, io::Error> {
    let mut outfile = OpenOptions::new()
        .write(true)
        .create(true)
        .open(filename)?;
    outfile.write_fmt(format_args!("Redirecting standard error to {}.\n", filename))?;
    let raw_fd = outfile.as_raw_fd();
    let new_fd = unsafe {
        libc::fflush(0 as *mut libc::FILE);
        libc::dup2(raw_fd, libc::STDERR_FILENO)
    };
    if new_fd != libc::STDERR_FILENO {
        Err(io::Error::new(io::ErrorKind::Other, format!("Could not call dup2. Ended up redirecting fd {} to {} instead of {}.", raw_fd, new_fd, libc::STDERR_FILENO)))
    }
    else { 
        Ok(outfile) 
    }
}


//#[cfg(not(any(feature="conrod-test", feature="sysmodule-test")))]
pub fn example() {
    let mut test_counter : usize = 0;
    let mut test_counters : [usize ; 4096] = [0 ; 4096];

    loop {
        test_counter += 1;
        if test_counter > 1024 || test_counters[test_counter] > 0 { 
            break ;
        }
        test_counters[test_counter] = 1;
        test_counter += 1;
    }
    test_counter *= 2;
    unsafe {
        gfxInitDefault();
        consoleInit(ptr::null_mut());

        println!("Press PLUS to exit.\n{}\n", test_counter);
        
        let mut vec = Vec::new();
        let mut prevKDown = 0;
        let mut frames = 0;
        let mut err_file : Option<File> = None;
        loop {
            if frames == 120 {
                println!("Attempting to redirect stderr.");
                let stderrRes = redirect_stderr("sderr.txt");
                err_file = match stderrRes{
                    Ok(fl) => {
                        Some(fl)
                    },
                    Err(err_struct) => {
                        println!("Got error with message:\n{:?}", err_struct);
                        None
                    }
                }
            }
            if err_file.is_some() {
                if frames == 200 {
                    println!("Attempting to libc::dprintf to the file.");
                }
                else if frames == 240 {
                    eprintln!("Test output.");
                    println!("Finished dprintf to the error file.");
                    println!("Attempting to libc::write to the file.");
                }
                else if frames == 360 {
                    let retval = libc::write(libc::STDERR_FILENO, "Test write.\n\0".as_ptr() as *const libc::c_void, 8);
                    let msg = format!("Finished libc::write with retval {}.\nTrying eprint with catch_unwind in 120 frames.\n\0", retval);
                    println!("{}", msg);
                }
                else if frames == 400 {
                    println!("Setting panic hook.");
                }
                else if frames == 560 {
                    panic::set_hook(Box::new(|info : &panic::PanicInfo| {
                        let fl = if let Ok(f) = redirect_stderr("sderr.txt") {
                            f
                        } 
                        else {
                            return;
                        };
                        let out_msg = format!("\n\n{}\n\0", info);
                        eprintln!("{}", out_msg);
                        libc::fflush(0 as *mut libc::FILE);
                    }));
                    println!("Finished setting panic hook.");
                }
            }

            if frames == 1080 {
                break;
            }
            hidScanInput();

            let kNum = hidKeysDown(HidControllerID::CONTROLLER_P1_AUTO) as u32;
            let kDown = HidControllerKeys(kNum);
            if kNum != prevKDown {
                vec.push(kNum);

                if kDown == HidControllerKeys::KEY_PLUS {
                    break;
                }

                prevKDown = kNum;
            }

            gfxFlushBuffers();
            gfxSwapBuffers();
            gfxWaitForVsync();
            frames += 1;
        }
        gfxExit();
    }
}

/*
#[cfg(feature = "conrod-test")]
#[macro_use]
extern crate conrod;

#[cfg(feature="conrod-test")]
mod conrod_example;

#[cfg(feature="conrod-test")]
pub fn example() {
    conrod_example::example();
}
*/
