
use libnx_rs::{LibnxError, service, ipc::IpcSession, ipc::Waitable, ipc::IpcSessionList};
use libnx_rs::ipc::{IpcCommandHeader, IpcCommandMessage, IpcCommandReadable, IpcCommandWriteable, RawIpcArgs, get_tls_space, SFCO_MAGIC};
use libnx_rs::fs::FsContext;
use std::time::Duration;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::time::Instant;
use std::slice;

pub fn runner() -> Result<(), LibnxError> {
    let mut sm_ctx = unsafe { service::SmContext::get_handle() } ;
    let mut session = sm_ctx.register_service("lnxrs", false, 32)?;
    let mut service = service::Service::create(session);
    let mut sessions : Vec<IpcSession> = Vec::with_capacity(32);
    sessions.push(service.handle());
    while let ipc_handle = sessions.wait_synchronization(Duration::from_nanos(u64::max_value()))? {
        if ipc_handle == service.handle(){
            let new_session = unsafe { ipc_handle.accept_session()? };
            sessions.push(new_session);
            continue;
        } 
        let rr_1_handle = ipc_handle.reply_and_receive(None, Duration::from_nanos(u64::max_value()))?;
        if rr_1_handle != ipc_handle {
            return Err(LibnxError::from_msg(format!("Error in rar 1: got message from handle {:?} when we were using {:?}", rr_1_handle, ipc_handle)));
        }

        let payload : IpcCommandMessage<RawIpcArgs> = unsafe { IpcCommandMessage::parse_from_tls() };
        
        let tls_buff = unsafe { slice::from_raw_parts_mut(get_tls_space() as *mut u32, 16) };
        tls_buff[0] = 4;
        tls_buff[1] = 6;
        tls_buff[2] = 0;
        tls_buff[3] = 0;

        tls_buff[4] = SFCO_MAGIC;
        tls_buff[5] = 0;
        tls_buff[6] = 0;
        tls_buff[7] = 0;
        tls_buff[8] = 0;
        tls_buff[9] = 0;
        tls_buff[10] = 0;
        tls_buff[11] = 0;
        tls_buff[12] = 0;
        tls_buff[13] = 0;
        tls_buff[14] = 0;
        tls_buff[15] = 0;
        let rr2_raw = ipc_handle.reply_and_receive(Some(ipc_handle), Duration::from_nanos(1000));
        match rr2_raw {
            Ok(_) => {},
            Err(e) => { 
                //TODO: Close handles on 0xf601
            }
        }
    }
    Ok(())
} 

pub static mut FAKE_HEAP : [u8 ; 0x40000] = [0 ; 0x40000];

#[no_mangle]
pub static __nx_applet_type : i32 = -2;
extern "C" {
    pub static __start__ : u32 ;
}
#[no_mangle]
pub unsafe extern "C" fn __libnx_initheap() {
    extern {
        static mut fake_heap_start : *mut u8; 
        static mut fake_heap_end : *mut u8;
    }
    fake_heap_start = &mut FAKE_HEAP[0] as *mut u8;
    fake_heap_end = &mut FAKE_HEAP[0x40000 - 1] as *mut u8;
}

#[no_mangle]
pub unsafe extern "C" fn __appInit() {
}

#[no_mangle]
pub unsafe extern "C" fn __appExit() {
}

pub fn example() {
    let mut sm_ctx = if let Ok(s) = service::SmContext::initialize() { s } else { return; };
    let mut fs_ctx = if let Ok(f) = FsContext::initialize() { f } else { return; };
    let mount_err = unsafe{ libnx_rs::libnx::fsdevMountSdmc()};
    if mount_err != 0 {
        ;
    }
    if let Err(e) = runner() {
        ; 
    }
    unsafe {libnx_rs::libnx::fsdevUnmountAll()};
    fs_ctx.exit();
    sm_ctx.exit();
}