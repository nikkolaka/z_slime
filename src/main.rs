#![windows_subsystem = "windows"]
#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::time::Duration;

use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{ VirtualKeyCode};
use winit::event_loop::{ EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use rand::Rng;
use game_loop::{game_loop, Time, TimeTrait as _};
use rand::seq::SliceRandom;


const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 900;
const CELLS_WIDTH: i16 = 300;
const CELLS_HEIGHT: i16 = 300;
const CELLS_X: usize = 100;
const CELLS_Y: usize = 200;
const SCALE: f32 = 2.0;
const SPAWN_RATE: f32 = 0.1;
const SURVIVAL_RATE: f32 = 0.99;
const FPS: f64 = 20.0;
pub const TIME_STEP: Duration = Duration::from_nanos(1_000_000_000 / FPS as u64);




/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    density: f32,
    width: i16,
    height: i16,  
    scale: f32, 
    zoom: i16,
    tiles: Vec<Vec<Cell>>,

}

fn main() -> Result<(), Error> {
    env_logger::init();
    let event_loop = EventLoop::new();
    
    let window = {
        let size = LogicalSize::new(SCREEN_WIDTH as f64, SCREEN_HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Z Life")
            .with_inner_size(size)
            .with_min_inner_size(size)
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

    impl Game{
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
            g.game.world.update_life();
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
            if g.game.input.key_pressed(VirtualKeyCode::Escape) || g.game.input.close_requested() {
                g.exit();
                return;
            }
            if g.game.input.key_pressed(VirtualKeyCode::Left) {
                g.game.world.update_seed();
                return;
            }

            if g.game.input.key_pressed(VirtualKeyCode::Up) {
                g.game.world.density += 0.02;
                g.game.world.update_seed();
                return;
            }

            if g.game.input.key_pressed(VirtualKeyCode::Down) {
                g.game.world.density -= 0.02;
                g.game.world.update_seed();
                return;
            }

            if g.game.input.key_pressed(VirtualKeyCode::Right) {

                g.game.world.update_map_iteration();
                return;
            }

            if g.game.input.key_pressed(VirtualKeyCode::Z) {
                g.game.world.zoom += 1;
                g.game.world.update_seed();
                return;
            }

            if g.game.input.key_pressed(VirtualKeyCode::X) {
                if g.game.world.zoom > 1{
                    g.game.world.zoom -= 1;
                    g.game.world.update_seed();
                }
                
                return;
            }

            if g.game.input.mouse_released(0){
                let Some((x, y)) = g.game.input.mouse() else { return; };

                let cells_pixel_width = (CELLS_WIDTH as f32* g.game.world.scale) as i16;
                let cells_pixel_height = (CELLS_HEIGHT as f32 * g.game.world.scale) as i16;
                let inside_cells = (x as i16) > CELLS_X.try_into().unwrap() && (x as i16) < (CELLS_X as i16) + cells_pixel_width  
                && (y as i16) > CELLS_Y.try_into().unwrap() && (y as i16) < (CELLS_Y as i16) + cells_pixel_height;

                if inside_cells {
                    let row: usize = ((y as i16 - CELLS_Y as i16) as f32 / g.game.world.scale) as usize % CELLS_HEIGHT as usize;
                    let col: usize = ((x as i16 - CELLS_X as i16) as f32 / g.game.world.scale) as usize % CELLS_WIDTH as usize;
                    if g.game.world.tiles[row][col] == Cell::Wall {
                        return;
                    }


                    g.game.world.tiles[row][col] = Cell::Life(rand::thread_rng().gen_range(0..=255), rand::thread_rng().gen_range(0..=255), rand::thread_rng().gen_range(0..=255));
                }

            }


            // Resize the window
            // if let Some(size) = g.game.input.window_resized() {
            //     if let Err(err) = g.game.pixels.resize_surface(size.width, size.height) {
            //         log_error("pixels.resize_surface", err);
            //         g.exit();
            //     }
            // }
        }
    }
    );


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
            zoom: 1,
            tiles: vec![vec![Cell::Empty; CELLS_HEIGHT.try_into().unwrap()]; CELLS_WIDTH.try_into().unwrap()],

        }
    }

    fn update_life(&mut self){
        let mut new_tiles = self.tiles.clone();
        for x in 0..self.width{
            for y in 0..self.height{
                let mut neighbors = 0;
                let mut neighbor_list: Vec<Cell> = Vec::new();

                match self.tiles[y as usize][x as usize]{
                    Cell::Wall => continue,
                    _ => (),
                }



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

                        match self.tiles[y as usize][x as usize]{
                            Cell::Life(r, g, b) => {
                                neighbors += 1;
                                neighbor_list.push(Cell::Life(r, g, b));
                            },
                            _ => (),
                        }

                    }
                }
                let will_survive = rand::random::<f32>() < SURVIVAL_RATE;
                let will_spawn = rand::random::<f32>() < SPAWN_RATE;
                if (neighbors > 5 || neighbors < 3) && !will_survive{

                    // match self.tiles[y as usize][x as usize]{
                    //     Cell::Life(_, _, _) => new_tiles[y as usize][x as usize] = Cell::Empty,
                    //     _ => (),
                    // }
                    
                }else if neighbors > 0 && will_spawn{
                    match neighbor_list.choose( &mut rand::thread_rng()){
                        Some(Cell::Life(r, g, b)) => {
                            let (r, g, b) = mutate_life_cell((*r, *g, *b));
                            new_tiles[y as usize][x as usize] = Cell::Life(r, g, b);
                        },
                        _ => (),
                    }
                        
                }
            }
            
        }
        self.tiles = new_tiles;
    }
    

    /// Update the `World` internal state; bounce the box around the screen.
    fn update_seed(&mut self) {
        for x in 0..self.width/self.zoom{
            for y in 0..self.height/self.zoom{
                let tile = if rand::random::<f32>() > self.density {Cell::Empty}else{Cell::Wall};

                for i in 0..self.zoom{
                    for j in 0..self.zoom{
                        self.tiles[(y*self.zoom + j) as usize][(x*self.zoom + i) as usize] = tile;
                    }
                }

                
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
    /// 
    fn draw(&mut self, frame: &mut [u8]) {

        // clear(frame);
        
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

                match tile{
                    Cell::Empty => [0xff, 0xff, 0xff, 0xff],
                    Cell::Wall => [0x00, 0x00, 0x00, 0xff],
                    Cell::Life(r, g, b) => [r.try_into().unwrap(), g.try_into().unwrap(), b.try_into().unwrap(), 0xff]
                }

                
                
            } else{
                //white background
                [0xff, 0xff, 0xff, 0xff]
            };

            pixel.copy_from_slice(&rgba);

        }
    }
}



fn mutate_life_cell((r, g, b) : (i16, i16, i16)) -> (i16, i16, i16){
    let mut rng = rand::thread_rng();

    let r_mutate = rng.gen_range(-2..3);
    let g_mutate = rng.gen_range(-2..3);
    let b_mutate = rng.gen_range(-2..3);
    

    let r = if r + r_mutate > 0 && r + r_mutate <= 255  {r + r_mutate} else {r};
    let g = if g + g_mutate > 0 && g + g_mutate <= 255  {g + g_mutate} else {g};
    let b = if b + b_mutate > 0 && b + b_mutate <= 255  {b + b_mutate} else {b};

    (r, g, b)

}





#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell{
    Empty,
    Wall,
    Life (i16, i16, i16)
}