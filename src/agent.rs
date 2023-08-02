
use rand::Rng;

const AGENT_SPEED: f32 = 1.0;


#[derive(Clone)]
pub struct Agent {
    pub x: f32,
    pub y: f32,
    pub rgb: (u8, u8, u8),
    velocity: (f32, f32),
}

impl Agent {
    pub fn new(x: f32, y: f32, rgb: (u8, u8, u8)) -> Self {
        Self {
            x,
            y,
            rgb,
            velocity: (random_float(0.0, 1.0), random_float(0.0, 1.0))
        }
    }

    pub fn update(&mut self, world_height: usize, world_width: usize) {
        self.x = (self.x as f32 + self.velocity.0)*AGENT_SPEED;
        self.y = (self.y as f32 + self.velocity.1)*AGENT_SPEED;

        if self.x >= (world_width as f32) || self.x <= 0.0 {
            self.velocity.0 = self.velocity.0*-1.0;
        }
        if self.y >= (world_height as f32) || self.y <= 0.0 {
            self.velocity.1 = self.velocity.1*-1.0;
        }
    }
}

fn random_float(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

