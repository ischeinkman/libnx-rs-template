#![allow(dead_code)]


extern crate libnx_rs;
use libnx_rs::libnx::*;
use libnx_rs::libc::{_NULL, dprintf};

extern crate libc;


use std::result::Result;
use std::path::Path;
use std::fs::File;
use std::fs::OpenOptions;
use std::error::Error;
use std::io::Write;
use std::io;
use std::os::unix::io::AsRawFd;
use std::panic;

#[cfg(feature = "conrod-test")]
#[macro_use]
extern crate conrod;

#[no_mangle]
pub extern "C" fn main() {
    runner();
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

#[cfg(not(feature = "conrod-test"))]
pub fn runner() {
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
        consoleInit(_NULL as *mut PrintConsole);

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
                        dprintf(libc::STDERR_FILENO , out_msg.as_ptr() as *const u8);
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

#[cfg(feature = "conrod-test")]
pub use feature::runner;
#[cfg(feature = "conrod-test")]
mod support;
#[cfg(feature = "conrod-test")]
mod feature {
    extern crate piston_window;
    extern crate libnx_rs_window;
    extern crate input as einp;
    use super::support;

    use self::libnx_rs_window::NxGlWindow;
    use self::libnx_rs_window::LibnxButtonId;

    use self::piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
    use self::piston_window::{Flip, G2d, G2dTexture, Texture, TextureSettings};
    use self::piston_window::OpenGL;
    use self::piston_window::texture::UpdateTexture;
    use super::redirect_stderr;
    use super::redirect_stdout;
    use conrod;
    use conrod::text::*;
    use conrod::position::{Align, Direction, Padding, Position, Relative};
    use conrod::{widget, Colorable, Labelable, Positionable, Sizeable, Widget};
    use conrod::event;
    use conrod::input;

    pub fn runner() {
        let errfl = match redirect_stderr("conrod-stderr.txt") {
            Ok(f) => f, 
            Err(er) => {return;}
        };
        let outfl = match redirect_stdout("conrod-stdout.txt") {
            Ok(f) => f, 
            Err(er) => {
                eprintln!("Redirection error: {:?}", er);
                return;
            }
        };
        const WIDTH: u32 = support::WIN_W;
        const HEIGHT: u32 = support::WIN_H;

        // Construct the window.
        let mut window: PistonWindow<NxGlWindow> = 
            match WindowSettings::new("", [WIDTH, HEIGHT])
                .opengl(OpenGL::V3_2) // If not working, try `OpenGL::V2_1`.
                .samples(4)
                .exit_on_esc(true)
                .vsync(true)
                .build() {
                    Ok(w) => w, 
                    Err(e) => {
                        return;
                    }
                };

        // construct our `Ui`.
        let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64])
            .theme(support::theme())
            .build();

        // Add a `Font` to the `Ui`'s `font::Map` from file.
        use std::fs::{File, read};
        use std::path::{Path, PathBuf};
        let font_data = match read(Path::new("assets/NotoSans-Regular.ttf")) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading font data file: {}", e);
                return;
            }
        };
        let collection = match FontCollection::from_bytes(font_data) {
            Ok(fc) => fc, 
            Err(e) => {
                eprintln!("error constructing a FontCollection from bytes: {}", e);
                return;
            }
        };
        
        let font = match collection.into_font() {// only succeeds if collection consists of one font
            Ok(f) => f,
            Err(e) => {
                eprintln!("error turning FontCollection into a Font: {}", e);
                return;
            }
        };

        ui.fonts.insert(font);

        // Create a texture to use for efficiently caching text on the GPU.
        let mut text_vertex_data = Vec::new();
        let (mut glyph_cache, mut text_texture_cache) = {
            const SCALE_TOLERANCE: f32 = 0.1;
            const POSITION_TOLERANCE: f32 = 0.1;
            let cache = conrod::text::GlyphCache::new(WIDTH, HEIGHT, SCALE_TOLERANCE, POSITION_TOLERANCE);
            let buffer_len = WIDTH as usize * HEIGHT as usize;
            let init = vec![128; buffer_len];
            let settings = TextureSettings::new();
            let factory = &mut window.factory;
            let texture = G2dTexture::from_memory_alpha(factory, &init, WIDTH, HEIGHT, &settings).unwrap();
            (cache, texture)
        };

        // Instantiate the generated list of widget identifiers.
        let ids = support::Ids::new(ui.widget_id_generator());

        // Load the rust logo from file to a piston_window texture.
        let rust_logo: G2dTexture = {
            let img = match read(Path::new("assets/rust.png")) {
                Ok(bts) => bts,
                Err(e) => {
                    eprintln!("error reading image bytes: {}", e);
                    return;
                }
            };
            let factory = &mut window.factory;
            let settings = TextureSettings::new();
            G2dTexture::from_memory_alpha(factory, &img, 500, 274, &settings).unwrap()
        };

        // Create our `conrod::image::Map` which describes each of our widget->image mappings.
        // In our case we only have one image, however the macro may be used to list multiple.
        let mut image_map = conrod::image::Map::new();
        let rust_logo = image_map.insert(rust_logo);

        // A demonstration of some state that we'd like to control with the App.
        let mut app = support::DemoApp::new(rust_logo);

        // Poll events from the window.
        while let Some(event) = window.next() {

            // Convert the piston event to a conrod event.
            let size = window.size();
            let (win_w, win_h) = (size.width as conrod::Scalar, size.height as conrod::Scalar);
            if let Some(e) = conrod::backend::piston::event::convert(event.clone(), win_w, win_h) {
                match e {
                    event::Input::Press(input::Button::Hat(ht)) => {
                        let rootid = ids.canvas;
                        if ht.state == einp::HatState::Up {
                            let scrl = [-80.0, -80.0];
                            ui.scroll_widget(rootid, scrl);
                        }
                        else if ht.state == einp::HatState::Down {
                            let scrl = [80.0, 80.0];
                            ui.scroll_widget(rootid, scrl);
                        }
                        ui.handle_event(e);

                    },
                    e => {
                        ui.handle_event(e);
                    }
                }
            }

            {
                let mut ui = ui.set_widgets();
                support::gui(&mut ui, &ids, &mut app);
            };

            window.draw_2d(&event, |context, graphics| {
                if let Some(primitives) = ui.draw_if_changed() {

                    // A function used for caching glyphs to the texture cache.
                    let cache_queued_glyphs = |graphics: &mut G2d,
                                               cache: &mut G2dTexture,
                                               rect: conrod::text::rt::Rect<u32>,
                                               data: &[u8]|
                    {
                        let offset = [rect.min.x, rect.min.y];
                        let size = [rect.width(), rect.height()];
                        let format = piston_window::texture::Format::Rgba8;
                        let encoder = &mut graphics.encoder;
                        text_vertex_data.clear();
                        text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                        let res = UpdateTexture::update(cache, encoder, format, &text_vertex_data[..], offset, size)
                            .expect("failed to update texture");
                        res
                    };

                    // Specify how to get the drawable texture from the image. In this case, the image
                    // *is* the texture.
                    fn texture_from_image<T>(img: &T) -> &T { img }

                    // Draw the conrod `render::Primitives`.
                    conrod::backend::piston::draw::primitives(primitives,
                                                              context,
                                                              graphics,
                                                              &mut text_texture_cache,
                                                              &mut glyph_cache,
                                                              &image_map,
                                                              cache_queued_glyphs,
                                                              texture_from_image);
                }
            });
        }
    }
}
