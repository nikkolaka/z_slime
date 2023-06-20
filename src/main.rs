#![deny(clippy::all)]
#![forbid(unsafe_code)]

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 900;
const CELLS_WIDTH: i16 = 300;
const CELLS_HEIGHT: i16 = 300;
const CELLS_X: usize = 100;
const CELLS_Y: usize = 200;
const SCALE: f32 = 2.0;

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    density: f32,
    width: i16,
    height: i16,  
    scale: f32, 
    tiles: Vec<Vec<Cell>>,
}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Z Life")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.close_requested() {
                *control_flow = ControlFlow::Exit;
                return;
            }
            if input.key_pressed(VirtualKeyCode::Left) {
                world.update_seed();
                return;
            }

            if input.key_pressed(VirtualKeyCode::Up) {
                world.density += 0.02;
                world.update_seed();
                return;
            }

            if input.key_pressed(VirtualKeyCode::Down) {
                world.density -= 0.02;
                world.update_seed();
                return;
            }

            if input.key_pressed(VirtualKeyCode::Right) {
                world.update_map_iteration();
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
            
            window.request_redraw();

            // Update internal state and request a redraw
            
        }
    });
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            density: 0.4,
            width: CELLS_WIDTH,
            height: CELLS_HEIGHT,
            scale: SCALE,   
            tiles: vec![vec![Cell::Empty; CELLS_HEIGHT.try_into().unwrap()]; CELLS_WIDTH.try_into().unwrap()],
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update_seed(&mut self) {
        for x in 0..self.width{
            for y in 0..self.height{
                let tile = if rand::random::<f32>() > self.density {Cell::Empty}else{Cell::Wall};

                self.tiles[y as usize][x as usize] = tile;
                
            }
        }
    }

    fn update_map_iteration(&mut self) {
        let mut new_tiles = vec![vec![Cell::Empty; CELLS_HEIGHT.try_into().unwrap()]; CELLS_WIDTH.try_into().unwrap()];
        for x in 0..self.width{
            for y in 0..self.height{
                let mut neighbors = 0;
                for i in -1..2{
                    for j in -1..2{
                        if i == 0 && j == 0{
                            continue;
                        }
                        let x = x + i;
                        let y = y + j;
                        if x < 0 || x >= self.width || y < 0 || y >= self.height{
                            continue;
                        }
                        if self.tiles[y as usize][x as usize] == Cell::Empty{
                            neighbors += 1;
                        }
                    }
                }
                let mut tile = self.tiles[y as usize][x as usize];
                if neighbors > 4{
                    // empty
                    tile = Cell::Empty;
                } else if neighbors < 4{
                    // wall
                    tile = Cell::Wall;
                    
                }
                new_tiles[y as usize][x as usize] = tile;
            }
        }
        self.tiles = new_tiles;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        
        let cells_pixel_width = (CELLS_WIDTH as f32* self.scale) as i16;
        let cells_pixel_height = (CELLS_HEIGHT as f32 * self.scale) as i16;
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % SCREEN_WIDTH as usize) as i16;
            let y = (i / SCREEN_WIDTH as usize) as i16;
            let inside_cells = x > CELLS_X.try_into().unwrap() && x < CELLS_X as i16 + cells_pixel_width  
                && y > CELLS_Y.try_into().unwrap() && y < CELLS_Y as i16 + cells_pixel_height;

           let rgba = if inside_cells{
                let row: usize = ((y - CELLS_Y as i16) as f32 / self.scale) as usize % CELLS_HEIGHT as usize;
                let col: usize = ((x - CELLS_X as i16) as f32 / self.scale) as usize % CELLS_WIDTH as usize;
                let tile = self.tiles[row][col];
                if tile == Cell::Empty{
                    //white empty cell
                    [0xff, 0xff, 0xff, 0xff]
                } else {
                    //black wall cell
                    [0x00, 0x00, 0x00, 0xff]
                }
                
                
            } else{
                //white background
                [0xff, 0xff, 0xff, 0xff]
            };

            pixel.copy_from_slice(&rgba);

        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell{
    Empty,
    Wall,
    Life (i16, i16, i16)
}