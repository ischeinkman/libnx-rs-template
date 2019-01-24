
extern crate piston_window;
extern crate libnx_rs_window;
extern crate input as einp;
use super::support;

use super::libnx_rs_window::NxGlWindow;
use super::libnx_rs_window::LibnxButtonId;

use super::piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
use super::piston_window::{Flip, G2d, G2dTexture, Texture, TextureSettings};
use super::piston_window::OpenGL;
use super::piston_window::texture::UpdateTexture;
use super::redirect_stderr;
use super::redirect_stdout;
use conrod;
use conrod::text::*;
use conrod::position::{Align, Direction, Padding, Position, Relative};
use conrod::{widget, Colorable, Labelable, Positionable, Sizeable, Widget};
use conrod::event;
use conrod::input;

pub fn example() {
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