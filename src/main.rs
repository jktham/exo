#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use pixels::{PixelsBuilder, SurfaceTexture};
use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use glam::Vec3;
use web_time::{SystemTime, UNIX_EPOCH};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

mod graphics;

mod game;
use game::*;

mod sprites;

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Trace).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(run());
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();

        pollster::block_on(run());
    }
}

#[cfg(target_arch = "wasm32")]
/// Retrieve current width and height dimensions of browser client window
fn get_window_size() -> LogicalSize<f64> {
    let client_window = web_sys::window().unwrap();
    LogicalSize::new(
        client_window.inner_width().unwrap().as_f64().unwrap(),
        client_window.inner_height().unwrap().as_f64().unwrap(),
    )
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Exo")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .expect("WindowBuilder error")
    };

    let window = Rc::new(window);

    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        // Attach winit canvas to body element
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(window.canvas().unwrap()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");

        // Listen for resize event on browser client. Adjust winit window dimensions
        // on event trigger
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new({
            let window = Rc::clone(&window);
            move |_e: web_sys::Event| {
                let _ = window.request_inner_size(get_window_size());
            }
        }) as Box<dyn FnMut(_)>);
        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();

        // Trigger initial resize event
        let _ = window.request_inner_size(get_window_size());
    }

    let mut input = WinitInputHelper::new();
    let mut pixels = {
        #[cfg(not(target_arch = "wasm32"))]
        let window_size = window.inner_size();

        #[cfg(target_arch = "wasm32")]
        let window_size = get_window_size().to_physical::<u32>(window.scale_factor());

        let surface_texture =
            SurfaceTexture::new(window_size.width, window_size.height, window.as_ref());
        let builder = PixelsBuilder::new(WIDTH, HEIGHT, surface_texture);

        #[cfg(target_arch = "wasm32")]
        let builder = {
            // Web targets do not support the default texture format
            let texture_format = pixels::wgpu::TextureFormat::Rgba8Unorm;
            builder
                .texture_format(texture_format)
                .surface_texture_format(texture_format)
        };

        builder.build_async().await.expect("Pixels error")
    };
    let mut game = Game::new();

    let mut t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    let mut dt = 0.0;

    let res = event_loop.run(|event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Update internal state
                game.update(dt);

                // Draw the current frame
                dt = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64() - t) as f32;
                t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();

                game.draw(pixels.frame_mut());
                if let Err(err) = pixels.render() {
                    log_error("pixels.render", err);
                    elwt.exit();
                    return;
                }

                window.request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                // Resize the window
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }

            _ => (),
        }

        if input.update(&event) {
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
            }

            game.ship.thrust = Vec3::new(0.0, 0.0, 0.0);
            if input.key_held(KeyCode::KeyA) {
                game.ship.thrust.x -= 10.0;
            }
            if input.key_held(KeyCode::KeyD) {
                game.ship.thrust.x += 10.0;
            }
            if input.key_held(KeyCode::KeyR) {
                game.ship.thrust.y += 10.0;
            }
            if input.key_held(KeyCode::KeyF) {
                game.ship.thrust.y -= 10.0;
            }
            if input.key_held(KeyCode::KeyW) {
                game.ship.thrust.z -= 10.0;
            }
            if input.key_held(KeyCode::KeyS) {
                game.ship.thrust.z += 10.0;
            }
            if input.key_held(KeyCode::Space) {
                game.ship.thrust = Vec3::ZERO;
                game.ship.velocity = Vec3::ZERO;
            }
        }
    });
    res.unwrap();
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}
