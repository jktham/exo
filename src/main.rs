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
use glam::{Mat4, Vec3, Vec4};
use web_time::{SystemTime, UNIX_EPOCH};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

struct Camera {
    position: Vec3,
    direction: Vec3,
    fov: f32,
}

struct Ship {
    position: Vec3,
    velocity: Vec3,
    thrust: Vec3,
}

struct Game {
    ship: Ship,
    camera: Camera,
}

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
            .with_title("Hello Pixels + Web")
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
                // Draw the current frame
                dt = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64() - t) as f32;
                t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();

                game.draw(pixels.frame_mut());
                if let Err(err) = pixels.render() {
                    log_error("pixels.render", err);
                    elwt.exit();
                    return;
                }

                // Update internal state and request a redraw
                game.update(dt);
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

impl Game {
    fn new() -> Self {
        Self {
            ship: Ship {
                position: Vec3::new(0.0, 0.0, 0.0),
                velocity: Vec3::new(0.0, 0.0, 0.0),
                thrust: Vec3::new(0.0, 0.0, 0.0),
            },
            camera: Camera {
                position: Vec3::new(0.0, 0.0, 1.0),
                direction: Vec3::new(0.0, 0.0, -1.0),
                fov: 90.0,
            },
        }
    }

    fn update(&mut self, dt: f32) {
        self.ship.velocity += self.ship.thrust * dt;
        self.ship.position += self.ship.velocity * dt;
    }

    fn draw(&self, frame: &mut [u8]) {
        let p0 = self.transform(self.ship.position);
        let p1 = self.transform(self.ship.position + Vec3::new(1.0, -1.0, 0.0));

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as f32;
            let y = (i / WIDTH as usize) as f32;

            let inside_the_box = 
                x >= p0.x && x < p1.x &&
                y >= p0.y && y < p1.y;

            let rgba = if inside_the_box {
                if p0.z < 1.0 {
                    [0xff, 0x00, 0x00, 0xff]
                } else {
                    [0x5e, 0x48, 0xe8, 0xff]
                }
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }

    fn transform(&self, pos: Vec3) -> Vec3 {
        let w = WIDTH as f32;
        let h = HEIGHT as f32;
        let n = 0.1;
        let f = 1000.0;
        let phi = self.camera.fov / 180.0 * 3.1415;
        let r = f32::tan(phi/2.0) * n;
        let t = r * h/w;

        let model = Mat4::from_translation(pos);
        let view = Mat4::look_at_lh(self.camera.position, self.camera.position + self.camera.direction, Vec3::new(0.0, 1.0, 0.0));
        let projection = Mat4::from_cols_array(&[
            n/r, 0.0, 0.0, 0.0,
            0.0, n/t, 0.0, 0.0,
            0.0, 0.0, -(f+n)/(f-n), -2.0*f*n/(f-n),
            0.0, 0.0, -1.0, 0.0,
        ]).transpose();

        let world = model * Vec4::new(0.0, 0.0, 0.0, 1.0);
        let eye = view * world;
        let clip = projection * eye;
        let ndc = Vec3::new(clip.x/clip.w, clip.y/clip.w, clip.z/clip.w);
        let mut screen = Vec3::new(w/2.0 * ndc.x + w/2.0, h/2.0 * ndc.y + h/2.0, (f-n)/2.0 * ndc.z + (f+n)/2.0);
        screen.z /= f;
        // println!("world: {}", world);
        // println!("eye: {}", eye);
        // println!("clip: {}", clip);
        // println!("ndc: {}", ndc);
        // println!("screen: {}", screen);

        return screen;
    }
}