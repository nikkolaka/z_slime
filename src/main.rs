#![windows_subsystem = "windows"]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

use agent::Agent;
mod agent;
use std::time::Duration;
use std::vec;

use error_iter::ErrorIter as _;
use game_loop::{game_loop, Time, TimeTrait as _};
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::Rng;
use winit::dpi::LogicalSize;
use winit::event::VirtualKeyCode;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 900;
const CELLS_WIDTH: usize = 300;
const CELLS_HEIGHT: usize = 300;
const CELLS_X: usize = 100;
const CELLS_Y: usize = 200;
const SCALE: f32 = 2.0;
const FPS: f64 = 20.0;


pub const TIME_STEP: Duration = Duration::from_nanos(1_000_000_000 / FPS as u64);

/// Representation of the application state. In this example, a box will bounce around the screen.

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();

    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Z Slime")
            .with_resizable(false)
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)?
    };

    struct Game {
        pixels: Pixels,
        input: WinitInputHelper,
        world: World,
    }

    impl Game {
        fn new(pixels: Pixels) -> Self {
            Self {
                pixels,
                input: WinitInputHelper::new(),
                world: World::new(),
            }
        }
    }

    let game = Game::new(pixels);

    game_loop(
        event_loop,
        window,
        game,
        FPS as u32,
        0.1,
        move |g| {
            // Update the world
            g.game.world.update();
        },
        move |g| {
            // Drawing

            g.game.world.draw(g.game.pixels.frame_mut());

            if let Err(err) = g.game.pixels.render() {
                log_error("pixels.render", err);
                g.exit();
            }

            // Sleep the main thread to limit drawing to the fixed time step.
            // See: https://github.com/parasyte/pixels/issues/174
            let dt = TIME_STEP.as_secs_f64() - Time::now().sub(&g.current_instant());
            if dt > 0.0 {
                std::thread::sleep(Duration::from_secs_f64(dt));
            }
        },
        |g, event| {
            // Let winit_input_helper collect events to build its state.
            //     // Handle input events
            if g.game.input.update(event) {
                // Close events
                if g.game.input.key_pressed(VirtualKeyCode::Escape)
                    || g.game.input.close_requested()
                {
                    g.exit();
                    return;
                }

                if g.game.input.mouse_released(0) {
                    let Some((x, y)) = g.game.input.mouse() else { return; };

                    g.game.world.mouse_action(x as i16, y as i16);
                }
            }
        },
    );
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

struct World {
    width: usize,
    height: usize,
    draw_scale: f32,
    tiles: Vec<Cell>,
    agents: Vec<Agent>,
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            width: CELLS_WIDTH,
            height: CELLS_HEIGHT,
            draw_scale: SCALE,
            tiles: vec![Cell::Empty; CELLS_WIDTH.checked_mul(CELLS_HEIGHT).expect("overflow")],
            agents: Vec::new(),
        }
    }

    fn mouse_inside_world(&self, x: i16, y: i16) -> bool {
        let cells_pixel_width = (CELLS_WIDTH as f32 * self.draw_scale) as i16;
        let cells_pixel_height = (CELLS_HEIGHT as f32 * self.draw_scale) as i16;
        let inside_cells = x > CELLS_X.try_into().unwrap()
            && x < CELLS_X as i16 + cells_pixel_width
            && y > CELLS_Y.try_into().unwrap()
            && y < CELLS_Y as i16 + cells_pixel_height;

        inside_cells
    }

    fn mouse_action(&mut self, x: i16, y: i16) {
        let inside_cells = self.mouse_inside_world(x, y);
        if inside_cells {
            let agent = Agent::new(
                x as f32,
                y as f32,
                (random_int(0, 255), random_int(0, 255), random_int(0, 255)),
            );
            self.agents.push(agent);
        }
    }

    fn update(&mut self) {
        self.update_agents();
        self.update_tiles();
    }

    fn update_agents(&mut self) {
        for agent in self.agents.iter_mut() {
            agent.update(self.height, self.width);
            self.tiles[(agent.x.round() * agent.y.round()) as usize] =
                Cell::Heat(agent.rgb.0, agent.rgb.1, agent.rgb.2);
        }
    }

    fn update_tiles(&mut self) {
        let mut write_tiles = self.tiles.clone();
        for x in 0..self.width {
            for y in 0..self.height {
                self.diffuse(x, y, &mut write_tiles)
            }
        }
        self.tiles = write_tiles;
    }

    fn diffuse(&mut self, x: usize, y: usize, write_tiles: &mut Vec<Cell>) {
        let idx = x + y * self.width;
        let mut r_sum = 0;
        let mut g_sum = 0;
        let mut b_sum = 0;
        match self.tiles[idx] {
            Cell::Empty => {}
            Cell::Heat(cr, cg, cb) => {
                r_sum += cr;
                g_sum += cg;
                b_sum += cb;
            }
        }

        for i in x - 1..x + 1 {
            for j in y - 1..y + 1 {
                match self.tiles[i + j * self.width] {
                    Cell::Empty => {}
                    Cell::Heat(r, g, b) => {
                        r_sum += r;
                        g_sum += g;
                        b_sum += b;
                    }
                }
            }
        }
        write_tiles[idx] = Cell::Heat(r_sum / 9, g_sum / 9, b_sum / 9);
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    ///
    fn draw(&mut self, frame: &mut [u8]) {
        // clear(frame);

        let cells_pixel_width = (CELLS_WIDTH as f32 * self.draw_scale) as i16;
        let cells_pixel_height = (CELLS_HEIGHT as f32 * self.draw_scale) as i16;
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % SCREEN_WIDTH as usize) as i16;
            let y = (i / SCREEN_WIDTH as usize) as i16;
            let inside_cells = x > CELLS_X.try_into().unwrap()
                && x < CELLS_X as i16 + cells_pixel_width
                && y > CELLS_Y.try_into().unwrap()
                && y < CELLS_Y as i16 + cells_pixel_height;

            let rgba = if inside_cells {
                let row: usize = ((y - CELLS_Y as i16) as f32 / self.draw_scale) as usize
                    % CELLS_HEIGHT as usize;
                let col: usize =
                    ((x - CELLS_X as i16) as f32 / self.draw_scale) as usize % CELLS_WIDTH as usize;
                let tile = self.tiles[row * CELLS_WIDTH + col];

                match tile {
                    Cell::Empty => [0xff, 0xff, 0xff, 0xff],
                    Cell::Heat(r, g, b) => [
                        r.try_into().unwrap(),
                        g.try_into().unwrap(),
                        b.try_into().unwrap(),
                        0xff,
                    ],
                }
            } else {
                //white background
                [0xff, 0xff, 0xff, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}

fn random_int(min: u8, max: u8) -> u8 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Empty,
    Heat(u8, u8, u8),
}





