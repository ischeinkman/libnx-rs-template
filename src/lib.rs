#![allow(dead_code)]

extern crate libnx_rs_window;
use libnx_rs_window::{LibnxButtonId, NxFullWindow};

extern crate framebuffer_graphics;
use framebuffer_graphics::{RgbaBufferGraphics, RgbaTexture};

#[macro_use]
extern crate conrod;
use conrod::backend::piston::draw::{Context};
mod support;

extern crate piston;
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderEvent, UpdateEvent};
use piston::window::{Size, WindowSettings, Window};

extern crate graphics;
use graphics::{Graphics};

extern crate libnx_rs;
pub use libnx_rs::libnx::*;

extern crate libc;

extern crate rusttype;
use rusttype::Font;

extern crate image;
use image::ImageResult;

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::result::Result;
use std::error::Error;
use std::io::Write;
use std::panic;


pub fn main() {
    let mut stdout = if let Ok(fl) = redirect_stdout("stdout.txt") { fl } else { return };
    let mut stderr = if let Ok(fl) = redirect_stderr("stderr.txt") { fl } else { return };
    println!("Writing line to stdout!");
    eprintln!("Writing line to stderr!");
    runner();
}

pub fn redirect_stdout (filename : &str) -> Result<File, String> {
    let mut outfile = File::open(Path::new(filename)).map_err(|e| e.description().to_owned())?;
    outfile.write_fmt(format_args!("Redirecting standard output to {}.", filename));
    outfile.sync_all();
    outfile.flush();
    let raw_fd = outfile.as_raw_fd();
    let new_fd = unsafe {
        libc::dup2(libc::STDOUT_FILENO, raw_fd)
    };
    if new_fd != raw_fd {
        Err("Could not open file!".to_owned())
    }
    else { 
        Ok(outfile) 
    }
}

pub fn redirect_stderr (filename : &str) -> Result<File, String> {
    let mut outfile = File::open(Path::new(filename)).map_err(|e| e.description().to_owned())?;
    outfile.write_fmt(format_args!("Redirecting standard err to {}.", filename));
    outfile.sync_all();
    outfile.flush();
    let raw_fd = outfile.as_raw_fd();
    let new_fd = unsafe {
        libc::dup2(libc::STDERR_FILENO, raw_fd)
    };
    if new_fd != raw_fd {
        Err("Could not open file!".to_owned())
    }
    else { 
        Ok(outfile) 
    }
}

pub fn runner() {

    const WIDTH: u32 = support::WIN_W;
    const HEIGHT: u32 = support::WIN_H;

    // Construct the window.
    let mut window = NxFullWindow::new();
    let mut event_loop = Events::new(EventSettings::new());
    let (w, h) = (window.size().width as usize, window.size().height as usize);
    let mut graphics = unsafe { RgbaBufferGraphics::new(w, h, window.get_framebuffer()) };
    
    // construct our `Ui`.
    let mut ui = conrod::UiBuilder::new([WIDTH as f64, HEIGHT as f64])
        .theme(support::theme())
        .build();

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    let font_bytes = include_bytes!("../assets/fonts/NotoSans-Regular.ttf");
    let font = if let Ok(ft) = Font::from_bytes(&font_bytes[..]) { ft } else { return; };
    ui.fonts.insert(font);

    // Create a texture to use for efficiently caching text on the GPU.
    let mut glyph_cache = {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        conrod::text::GlyphCache::new(WIDTH, HEIGHT, SCALE_TOLERANCE, POSITION_TOLERANCE)
    };

    //Dummy texture cache
    let mut text_texture_cache = RgbaTexture::empty(0, 0);

    // Instantiate the generated list of widget identifiers.
    let ids = support::Ids::new(ui.widget_id_generator());

    // Load the rust logo from file to a piston_window texture.
    let rust_logo = {
        let image_bytes = include_bytes!("../assets/images/rust.png");
        let res = image::load_from_memory(image_bytes);
        if let Ok(img) = res {
            RgbaTexture::from_piston_image(img.to_rgba())
        } else {
            return;
        }
    };

    // Create our `conrod::image::Map` which describes each of our widget->image mappings.
    // In our case we only have one image, however the macro may be used to list multiple.
    let mut image_map = conrod::image::Map::new();
    let rust_logo = image_map.insert(rust_logo);

    // A demonstration of some state that we'd like to control with the App.
    let mut app = support::DemoApp::new(rust_logo);

    // Poll events from the window.
    while let Some(event) = event_loop.next(&mut window) {
        // Convert the piston event to a conrod event.
        let size = window.size();
        let (win_w, win_h) = (size.width as conrod::Scalar, size.height as conrod::Scalar);
        if let Some(e) = conrod::backend::piston::event::convert(event.clone(), win_w, win_h) {
            ui.handle_event(e);
        }

        event.update(|_| {
            let mut ui = ui.set_widgets();
            support::gui(&mut ui, &ids, &mut app);
        });
        
        if let Some(args) = event.render_args(){
            let context = Context::new_viewport(args.viewport());
            if let Some(primitives) = ui.draw_if_changed() {
                
                // Dummy function for texture caching.
                // Since we don't actually need to it does nothing. 
                let cache_queued_glyphs = |graphics : &mut RgbaBufferGraphics, 
                                           texture : &mut RgbaTexture, 
                                           rect : conrod::text::rt::Rect<u32>, 
                                           data : &[u8] | 
                {
                    
                };

                            
                fn texture_from_image (img: &RgbaTexture) -> &RgbaTexture {
                    img
                }

                // Draw the conrod `render::Primitives`.
                conrod::backend::piston::draw::primitives(
                    primitives,
                    context,
                    &mut graphics,
                    &mut text_texture_cache,
                    &mut glyph_cache,
                    &image_map,
                    cache_queued_glyphs,
                    texture_from_image,
                );
            }
        }
    }
}
