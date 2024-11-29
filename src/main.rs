extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
// use piston::event_loop::{EventSettings, Events, EventLoop};
use piston::input::*;
use piston::event_loop::*;
use rand::prelude::*;
// use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

const PLAYER_MOVE_STEP: f64 = 5.0;
const PLAYER_SIZE: f64 = 20.0;
const BULLET_SPEED: f64 = 10.0;
const FIRE_RATE: u32 = 20; // Number of update cycles per shot 

const MAX_ENEMIES: usize = 50;
const ENEMY_SPAWN_PROB: f64 = 0.05;
const ENEMY_MOVE_SPEED: f64 = 1.0;

pub struct Bullet {
    x_pos: f64,
    y_pos: f64,
    size: f64,
    hit: bool,
}

pub struct Enemy {
    x_pos: f64,
    y_pos: f64,
    size: f64,
    killed: bool,
}

pub struct Multiplier {
    y_pos: f64,
    multiplier: u8
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    num_columns: u8,
    player_x: f64,  // X position of the player square.
    player_y: f64,
    window_width: f64,
    window_height: f64,
    left_down: bool,
    right_down: bool,
    fire_counter: u32,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    multipliers: Vec<Multiplier>,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const ZOMBIE_GREEN: [f32; 4] = [0.4, 0.6, 0.1, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        self.window_width = args.window_size[0];
        let width = self.window_width;
        self.window_height = args.window_size[1];
        let height = self.window_height;

        let player = rectangle::square(0.0, 0.0, PLAYER_SIZE);
        let x = self.player_x;
        self.player_y = args.window_size[1] * 5.0 / 6.0;
        let y = self.player_y;

        let cols = self.num_columns; // This feels wrong - should be a different way to get self.num_columsn into the lambda below
        let mut bullet_recs: Vec<[f64; 4]> = Vec::new();
        for bullet in &self.bullets {
            let x = bullet.x_pos;
            let y = bullet.y_pos;
            bullet_recs.push(ellipse::circle(x, y, bullet.size/2.0));
        }

        let mut enemy_recs: Vec<[f64; 4]> = Vec::new();
        for enemy in &self.enemies {
            let x = enemy.x_pos;
            let y = enemy.y_pos;
            enemy_recs.push(ellipse::circle(x, y, enemy.size/2.0));
        }

        // let mut multiplier_recs: Vec<[f64; 4]> = Vec::new();
        
        let multiplier_recs = &self.multipliers;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(WHITE, gl);

            // Draw the columns
            let mut col: u8 = 0;
            while col < cols {
                let col_x = (col as f64 + 1.0) * width / (cols as f64);
                line(BLACK, 1.0, [col_x, 0.0, col_x, height * 3.0 / 4.0], c.transform, gl);
                col += 1;
            }

            // Draw the player box
            let transform = c
                .transform
                .trans(x, y)
                .trans(-PLAYER_SIZE/2.0, -PLAYER_SIZE/2.0); //translate to center of box
            rectangle(BLACK, player, transform, gl);

            // Draw bullets
            for bullet in bullet_recs {
                ellipse(RED, bullet, c.transform, gl);
            }

            // Draw enemies
            for enemy in enemy_recs {
                ellipse(ZOMBIE_GREEN, enemy, c.transform, gl);
            }

            // Draw multipliers
            for multiplier in multiplier_recs {
                Rectangle::new(BLUE).draw_from_to([0.0, multiplier.y_pos-5.0], 
                                        [width/2.0, multiplier.y_pos+5.0],
                                        &Default::default(),
                                        c.transform, gl);                   
            }        

        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        // Update player position if left or right is pressed
        if self.left_down {
            self.player_x -= PLAYER_MOVE_STEP;
        }
        if self.right_down {
            self.player_x += PLAYER_MOVE_STEP;
        }

        // make sure position is not off screen
        if self.player_x < PLAYER_SIZE/2.0 {
            self.player_x = PLAYER_SIZE/2.0;
        }
        if self.player_x > (self.window_width-PLAYER_SIZE/2.0) {
            self.player_x = self.window_width-PLAYER_SIZE/2.0;
        }

        // Update shot counter & fire new bullet if needed
        self.fire_counter += 1;
        if self.fire_counter >= FIRE_RATE
        {
            self.fire();
            self.fire_counter = 0;
        }

        // Update bullet positions
        for bullet in &mut self.bullets {
            bullet.y_pos -= BULLET_SPEED;
            if bullet.y_pos < 0.0 {
                bullet.hit = true;
            }
        }

        let mut new_bullets = Vec::<Bullet>::new();
        for mult in &self.multipliers {
            for bullet in &mut self.bullets {
                if bullet.x_pos < self.window_width/2.0 && 
                mult.y_pos - bullet.y_pos < BULLET_SPEED && 
                mult.y_pos > bullet.y_pos {
                    bullet.hit = true;
                    let mut new_bullet_count : u8 = 0;
                    while new_bullet_count < mult.multiplier {
                        let x_mod = (new_bullet_count as f64 - (mult.multiplier as f64/2.0)) * 7.0;
                        new_bullets.push(Bullet{x_pos: bullet.x_pos + x_mod, 
                                            y_pos: bullet.y_pos,
                                            size: PLAYER_SIZE/4.0,
                                            hit: false});
                        new_bullet_count += 1;
                    }
                }
            }
        }
        self.bullets.append(&mut new_bullets);

        // Move enemies
        for enemy in &mut self.enemies {
            enemy.y_pos += ENEMY_MOVE_SPEED;
            if enemy.y_pos >= self.window_height * 9.0 / 10.0 {
                enemy.y_pos = self.window_height * 9.0 / 10.0;
            }
            for bullet in &mut self.bullets {
                // bullet has same x value
                if (enemy.x_pos - bullet.x_pos).abs() < (enemy.size + bullet.size)/2.0 {
                    // bullet has passed through y value...
                    if enemy.y_pos > bullet.y_pos {
                        enemy.killed = true;
                        bullet.hit = true;
                    } 
                }
            }
        }

        // spawn new enemies
        let mut rng = rand::thread_rng(); // random number generator
        let spawn_prob: f64 = rng.gen(); // generates a float between 0 and 1
        if self.enemies.len() < MAX_ENEMIES && spawn_prob < ENEMY_SPAWN_PROB {
            let new_enemy = Enemy{x_pos: (rng.gen::<f64>()*0.8 + 0.1)*self.window_width/2.0,
                                  y_pos: -5.0,
                                  size: PLAYER_SIZE/1.8, 
                                  killed: false};
            self.enemies.push(new_enemy);
        }

        // remove any bullets or enemies that are used / dead
        self.bullets.retain(|bullet|{let retain = bullet.hit == false; return retain;});
        self.enemies.retain(|enemy|{let retain = enemy.killed == false; return retain;});

    }

    fn left_pressed(&mut self) {
        self.left_down = true;
    }

    fn left_released(&mut self) {
        self.left_down = false;
    }

    fn right_pressed(&mut self) {
        self.right_down = true;
    }

    fn right_released(&mut self) {
        self.right_down = false;
    }

    fn set_player_x(&mut self, new_x: f64) {
        self.player_x = new_x;
    }

    fn fire(&mut self) {
        self.bullets.push(Bullet{x_pos: self.player_x, 
                                 y_pos: self.player_y - PLAYER_SIZE/2.0,
                                 size: PLAYER_SIZE/4.0,
                                 hit: false});
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    const WINDOW_X: u32 = 500;
    const WINDOW_Y: u32 = 800;

    // Create a Glutin window
    let mut window: Window = WindowSettings::new("Column Clearer", [WINDOW_X, WINDOW_Y])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        num_columns: 2,
        player_x: WINDOW_X as f64 / 2.0,
        player_y: WINDOW_Y as f64 * 5.0 / 6.0,
        window_width: WINDOW_X as f64,
        window_height: WINDOW_Y as f64,
        left_down: false,
        right_down: false,
        fire_counter: 0,
        bullets: Vec::<Bullet>::new(),
        enemies: vec![Enemy{x_pos: 20.0, y_pos: 10.0, size: PLAYER_SIZE/2.0, killed: false}],
        multipliers: vec![Multiplier{y_pos: WINDOW_Y as f64 * 0.5, multiplier: 3},
                          Multiplier{y_pos: WINDOW_Y as f64 * 0.25, multiplier: 2},]
    };

    let mut events = Events::new(EventSettings::new().ups(60));
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            // println!("Pressed keyboard key '{:?}'", key);
            if key == Key::Left {
                app.left_pressed();
            }
            if key == Key::Right {
                app.right_pressed();
            }
        };

        if let Some(Button::Keyboard(key)) = e.release_args() {
            if key == Key::Left {
                app.left_released();
            }
            if key == Key::Right {
                app.right_released();
            }
        };

        if let Some(touch) = e.touch_args() {
            app.set_player_x(touch.position_3d[0]);
        }
    }
}
