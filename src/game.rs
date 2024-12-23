use std::f32::consts::PI;

use raylib::prelude::*;
use rand::prelude::*;


/// Model points are relative to zero
pub struct Model<const SIZE: usize>
{
    points: [Vector2; SIZE],
    position: Vector2,
    rotation: f32,
    scale: f32,
    color: Color
}

impl<const SIZE: usize> Model<SIZE>  {

    pub fn new(points: [Vector2; SIZE],position: Vector2) -> Self {
         Self { points, position, rotation: 0.0, scale: 1.0, color: Color::WHITE } 
    }

    pub fn draw_points(&self) -> Vec<Vector2> {
        let mut batch = vec![Vector2 { x: 0.0, y: 0.0 }; SIZE + 1];
        for i in 0..SIZE {
            batch[i] = self.points[i].clone();
            batch[i].rotate(self.rotation);
            batch[i].scale(self.scale);
            batch[i] += self.position;
        }
        batch[SIZE] = batch[0];
        return batch;
    }

    pub fn render(&self,d: &mut impl RaylibDraw)
    {
        d.draw_line_strip(&self.draw_points(), self.color);
    }

    pub fn get_direction(&self) -> Vector2
    {
        Vector2 { x: self.rotation.sin(), y: -self.rotation.cos() }
    }

    pub fn apply_constraints(&mut self){
        if self.position.x < 0.0 {
            self.position.x += Game::WIDTH as f32;
        } else if self.position.x > Game::WIDTH as f32 {
            self.position.x -= Game::WIDTH as f32; 
        }
        if self.position.y < 0.0 {
            self.position.y += Game::HEIGHT as f32; 
        } else if self.position.y > Game::HEIGHT as f32 {
            self.position.y -= Game::HEIGHT as f32; 
        }
    }
}

pub trait Entity {
    fn spawn(position: Vector2) -> Self;
    fn render(&self,d: &mut impl RaylibDraw);
    fn apply(&mut self,delta_time: f32);
}

pub struct Player 
{
    model: Model<4>,
    force: Vector2
}

impl Player {
    const SPEED: f32 = 5.0;
    const MAX_SPEED: f32 = 300.0;
    const ROTATIONAL_SPEED: f32 = 7.5;

    const POINTS: [Vector2; 4] = [
        Vector2 {x: 0.0, y: -20.0},
        Vector2 {x: -10.0, y: 10.0},
        Vector2 {x: 0.0, y: 5.0},
        Vector2 {x: 10.0, y: 10.0}
    ];
}

impl Entity for Player {

    fn spawn(position: Vector2) -> Self {
        Self { model: Model::new(Self::POINTS, position), force: Vector2 { x: 0.0, y: 0.0 } }
    }

    fn render(&self,d: &mut impl RaylibDraw) {
        self.model.render(d);
        // TODO render HUD
    }

    fn apply(&mut self,delta_time: f32) {
        self.model.position += self.force * delta_time;
        self.model.apply_constraints();
    }
}

#[derive(Clone)]
pub enum AsteroidType {
    Small,
    Medium,
    Large
}

impl AsteroidType {
    pub fn size(&self) -> f32 {
        match self {
            Self::Small => 0.3,
            Self::Medium => 1.0,
            Self::Large => 2.0,
        }
    }
    pub fn degrade(&self) -> Option<AsteroidType> {
        match self {
            Self::Small => None,
            Self::Medium => Some(Self::Small),
            Self::Large => Some(Self::Medium),
        }
    }
    pub fn random(rng: &mut ThreadRng) -> Self {
        let variants = [AsteroidType::Small, AsteroidType::Medium, AsteroidType::Large];
        variants[rng.gen_range(0..variants.len())].clone()
    }
}

pub struct Asteroid
{
    model: Model<10>,
    direction: Vector2,
    class: AsteroidType,
}

impl Asteroid {
    const DEFAULT_MAX: f32 = 80.0;
    const DEFAULT_MIN: f32 = 20.0;
    const SPEED: f32 = 100.0;
}

impl Entity for Asteroid {

    fn spawn(position: Vector2) -> Self {
        let mut rng = rand::thread_rng();

        let class = AsteroidType::random(&mut rng);
        let direction = Vector2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0));

        let mut points = [Vector2::zero(); 10];

        let increment = (PI * 2.0) / (points.len() - 1) as f32;
        let mut current: f32 = 0.0;

        let min = Self::DEFAULT_MIN * class.size();
        let max = Self::DEFAULT_MAX * class.size();

        for i in 0..points.len() {
            points[i].x = current.sin() * rng.gen_range(min..max);
            points[i].y = current.cos() * rng.gen_range(min..max);
            current += increment;
        }

        Self { model: Model::new(points, position), direction, class }
    }

    fn render(&self,d: &mut impl RaylibDraw) {
        self.model.render(d);
    }

    fn apply(&mut self,delta_time: f32) {
        self.model.position += (self.direction / self.class.size()) * Self::SPEED * delta_time;
        self.model.apply_constraints();
    }
}

pub struct Game 
{
    pub player: Player,
    pub asteroids: Vec<Asteroid>,
    level: u32,
    // all time is in seconds
    pause: bool,
    pause_time: f64,
    pause_end: f64,
    // UI components
    show_level: bool,
}

impl Game {
    
    pub const WIDTH: u32 = 800;
    pub const HEIGHT: u32 = 800;
    pub const DIFFICULTY: f32 = 2.5;
    const MID_X: f32 = Game::WIDTH as f32 / 2.0;
    const MID_Y: f32 = Game::HEIGHT as f32 / 2.0;

    pub fn new() -> Self {
        let player = Player::spawn(Vector2::new(Game::WIDTH as f32 / 2.0, Game::HEIGHT as f32 / 2.0));

        Self { player, asteroids: Vec::new(), level: 0, pause: false, pause_time: 0.0, pause_end: 0.0, show_level: false }
    }

    pub fn levelup(&mut self){
        self.level += 1;

        self.player.model.position = Vector2::new(Self::MID_X, Self::MID_Y);

        let count = (self.level as f32 * Self::DIFFICULTY).round();
        let mut rng = rand::thread_rng();

        const CLOSEST: f32 = 150.0;
        
        let increment = (PI * 2.0) / count;
        let mut current: f32 = 0.0;

        self.asteroids.reserve(count as usize);

        for _ in 0..count as u32 {
            let x = current.sin() * rng.gen_range(CLOSEST..(CLOSEST*2.0)) + Self::MID_X;
            let y = current.cos() * rng.gen_range(CLOSEST..(CLOSEST*2.0)) + Self::MID_Y;

            let asteriod = Asteroid::spawn(Vector2 { x, y });

            self.asteroids.push(asteriod);

            current += increment;
        }

        self.pause = true;
        self.pause_end = 2.0; // 2 seconds
        self.show_level = true;
    }

    pub fn update(&mut self,rl: &RaylibHandle){

        if self.pause {
            if self.pause_time == 0.0 {
                self.pause_time = rl.get_time();
            }
            if rl.get_time() - self.pause_time >= self.pause_end {
                self.pause = false;
                self.show_level = false;
                self.pause_time = 0.0;
            }
            return;
        }

        let dt = rl.get_frame_time();

        if rl.is_key_down(KeyboardKey::KEY_A)
        {
            self.player.model.rotation -= Player::ROTATIONAL_SPEED * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_D)
        {
            self.player.model.rotation += Player::ROTATIONAL_SPEED * dt;
        }
        if rl.is_key_down(KeyboardKey::KEY_W)
        {
            self.player.force += self.player.model.get_direction() * Player::SPEED;
            self.player.force = self.player.force.clamp(-Player::MAX_SPEED..Player::MAX_SPEED);
        } else 
        {
            // drag
            self.player.force.scale(0.98);
        }
        // apply physics
        self.player.apply(dt);
        for asteroid in &mut self.asteroids {
            asteroid.apply(dt);
        }
    }

    pub fn render(&self,d: &mut impl RaylibDraw){

        if self.show_level {
            let text = format!("Level: {}",self.level);
            const FONT_SIZE: i32 = 80;
            let mid = (text.len() as i32 * FONT_SIZE) / 4;
            d.draw_text(text.as_str(), Self::MID_X as i32 - mid, Self::MID_Y as i32 + FONT_SIZE, FONT_SIZE, Color::WHITE);
        }

        self.player.render(d);

        for asteroid in &self.asteroids {
            asteroid.render(d);
        }
    }

    pub fn pause(&mut self,time: f64){
        self.pause_end = time;
        self.pause = true;
    }
}