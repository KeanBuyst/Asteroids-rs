use game::*;
use raylib::prelude::*;
use raylib::ffi;

pub mod game;

fn main() {
    let (mut rl, thread) = raylib::init()
        .vsync()
        .size(Game::WIDTH as i32, Game::HEIGHT as i32)
        .title("Asteroids")
        .build();

    unsafe {
        ffi::rlEnableSmoothLines();
        ffi::rlSetLineWidth(1.5);
    }

    let mut game = Game::new();

    game.levelup();

    let mut render_texture = rl.load_render_texture(&thread, Game::WIDTH, Game::HEIGHT).unwrap();

    while !rl.window_should_close() {

        let screen_width = rl.get_screen_width();
        let screen_height = rl.get_screen_height();

        game.update(&rl);

        let mut d = rl.begin_drawing(&thread);

        {
            let mut rt = d.begin_texture_mode(&thread, &mut render_texture);
            rt.clear_background(Color::BLACK);

            game.render(&mut rt);
        }

        d.draw_texture_pro(&render_texture,
             Rectangle::new(0.0, 0.0, Game::WIDTH as f32,-(Game::HEIGHT as f32)),
              Rectangle::new(0.0, 0.0, screen_width as f32, screen_height as f32),
              Vector2::zero(), 0.0, Color::WHITE);
    }
}