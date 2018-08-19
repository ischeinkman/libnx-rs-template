#![allow(dead_code)]
#![crate_type = "staticlib"]

extern crate libnx_rs;
pub use libnx_rs::libnx::*;
pub use libnx_rs::libc::*;

#[no_mangle]
pub extern fn main(_argc : isize, _argv : * const * const u8) ->isize { unsafe {

    gfxInitDefault();
    consoleInit(_NULL as *mut PrintConsole);

    printf("Press PLUS to exit.".as_ptr() as *const u8);
    let mut vec = Vec::new();
    let mut prevKDown = 0;
    while appletMainLoop() {
        hidScanInput();

        let kNum = hidKeysDown(HidControllerID::CONTROLLER_P1_AUTO) as u32;
        let kDown = HidControllerKeys(kNum);
        if kNum!= prevKDown {
            vec.push(kNum);

            if kDown == HidControllerKeys::KEY_PLUS {
                break;
            }

            printf("Rec number %d => %d\n".as_ptr() as *const u8, vec.len(), kNum);
            prevKDown = kNum;
        }

        gfxFlushBuffers();
        gfxSwapBuffers();
        gfxWaitForVsync();
    }
	gfxExit();
    0
}}