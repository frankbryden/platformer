use ggez::*;
use mint;
use mint::{Vector2, Point2};
use ggez::event::EventHandler;
use ggez::input::keyboard;
use ggez::graphics::{Rect, DrawParam, Mesh, MeshBuilder, Color, DrawMode};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::{Context, GameResult};
use ggez::input::keyboard::*;
use ggez::audio::SoundSource;
use std::path;

use nalgebra as na;

mod coinspawner;
mod sounds;

use sounds::SoundSet;
use coinspawner::{Coin, CoinSpawner, ItemSet};

struct State {
    sprite_batch: SpriteBatch,
    sprite_defs_walking: Vec<Rect>,
    sprite_def_idle: Rect,
    sprite_def_jump: Rect,
    player_sprite_width: f32,
    player_sprite_height: f32,
    player: Player,
    ground: usize,
    ground_mesh: Mesh,
    bomb_sprite: graphics::Image,
    coin_spawner: CoinSpawner,
    sound_set: SoundSet,
}

struct Player {
    state: PlayerState,
    vel: mint::Vector2<f32>,
    pos: mint::Point2<f32>,
    score: usize,
    walk_frame: usize,
    last_swap: usize, //number of milliseconds since last frame swap
    frame_duration: usize, //number of milliseconds per frame
}

#[derive(PartialEq)]
enum PlayerState {
    Stand,
    Walk,
    Jump,
}

impl State {
    fn new(ctx: &mut Context) -> GameResult<State> {
        let (w, h) = graphics::drawable_size(ctx);
        let world_width = w as i32;
        let world_height = h as i32;
        let ground = 500.0;

        println!("World width/height: {}, {}", world_width, world_height);

        let mut walking_defs = Vec::new();
        walking_defs.push(Rect::new(0.0, 0.0, 72.0, 97.0));
        walking_defs.push(Rect::new(146.0, 0.0, 72.0, 97.0));
        walking_defs.push(Rect::new(0.0, 98.0, 72.0, 97.0));
        walking_defs.push(Rect::new(73.0, 98.0, 72.0, 97.0));
        walking_defs.push(Rect::new(146.0, 98.0, 72.0, 97.0));
        walking_defs.push(Rect::new(219.0, 0.0, 72.0, 97.0));
        walking_defs.push(Rect::new(292.0, 0.0, 72.0, 97.0));
        let mut idle_def = Rect::new(67.0, 196.0, 66.0, 92.0);
        let mut jump_def = Rect::new(438.0, 93.0, 67.0, 94.0);
        let player_sprite_width = 72.0;
        let player_sprite_height = 97.0;



        let image = graphics::Image::new(ctx, "/Player/p1_spritesheet.png").unwrap();
        let bomb = graphics::Image::new(ctx, "/Items/bomb.png").unwrap();
        let img_width = image.width() as f32;
        let img_height = image.height() as f32;

        let batch = SpriteBatch::new(image);

        for def in walking_defs.iter_mut(){
            def.x /= img_width;
            def.y /= img_height;
            def.w /= img_width;
            def.h /= img_height;
        }

        idle_def.x /= img_width;
        idle_def.y /= img_height;
        idle_def.w /= img_width;
        idle_def.h /= img_height;
        
        jump_def.x /= img_width;
        jump_def.y /= img_height;
        jump_def.w /= img_width;
        jump_def.h /= img_height;

        println!("{:?}", walking_defs);

        let coin_spawner = CoinSpawner::new(ctx, world_width, world_height, ground);

        let ground_mesh = MeshBuilder::new()
                .rectangle(DrawMode::fill(), Rect::new(0.0, ground, world_width as f32, 300.0), Color::new(0.2, 0.5, 0.3, 1.0))
                .build(ctx)?;

        let state = State { 
            sprite_batch: batch,
            sprite_defs_walking: walking_defs,
            sprite_def_idle: idle_def,
            sprite_def_jump: jump_def,
            player_sprite_width: player_sprite_width,
            player_sprite_height: player_sprite_height,
            player: Player::new(),
            ground: ground as usize,
            ground_mesh: ground_mesh,
            bomb_sprite: bomb,
            coin_spawner: coin_spawner,
            sound_set: SoundSet::new(ctx),
        };

        
        Ok(state)
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if timer::ticks(ctx) % 100 == 0 {
            println!("Delta frame time: {:?} ", timer::delta(ctx));
            println!("Average FPS: {}", timer::fps(ctx));
        }
        let mult = (timer::delta(ctx).as_millis() as f32)/3.0;
        self.player.pos.x += self.player.vel.x * mult;
        match self.player.state {
            PlayerState::Walk => {
                self.player.last_swap += timer::delta(ctx).as_millis() as usize;
                if self.player.last_swap > self.player.frame_duration {
                    self.player.walk_frame += 1;
                    if self.player.walk_frame >= self.sprite_defs_walking.len() {
                        self.player.walk_frame = 0;
                    }
                    self.player.last_swap = 0;
                }
            },
            PlayerState::Jump => {
                self.player.pos.y += self.player.vel.y * mult;
                self.player.vel.y += 0.2;
                if self.player.pos.y + self.player_sprite_height >= self.ground as f32 {
                    self.player.pos.y = self.ground as f32 - self.player_sprite_height;
                    self.player.vel.y = 0.0;
                    self.player.state = if keyboard::is_key_pressed(ctx, KeyCode::D) || 
                                            keyboard::is_key_pressed(ctx, KeyCode::Q) {
                                                PlayerState::Walk
                                            } else {
                                                PlayerState::Stand
                                            };
                }
            },
            _ => (),
        };
        self.coin_spawner.update(ctx);
        let player_rect = Rect::new(self.player.pos.x - self.coin_spawner.coin_radius/2.0, self.player.pos.y - self.coin_spawner.coin_radius/2.0, self.player_sprite_width + self.coin_spawner.coin_radius/2.0, self.player_sprite_height + self.coin_spawner.coin_radius/2.0);
        let coins_before = self.coin_spawner.coins.len();
        self.coin_spawner.coins.retain(|coin| {
            !player_rect.contains(coin.pos)
        });
        //println!("{} - {}", self.coin_spawner.coins.len(), coins_before);
        let score_delta = coins_before - self.coin_spawner.coins.len();
        if score_delta > 0 {
            self.sound_set.coin_collect.play();
        }
        self.player.score += score_delta;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        graphics::draw(ctx, &self.bomb_sprite, DrawParam::default().dest(mint::Point2{x:200.0, y:300.0}))?;

        let player_sprite_rect = match self.player.state {
            PlayerState::Stand => self.sprite_def_idle.clone(),
            PlayerState::Walk => self.sprite_defs_walking.get(self.player.walk_frame).unwrap().clone(),
            PlayerState::Jump => self.sprite_def_jump.clone(),
        };

        let mut p = DrawParam::new()
            .src(player_sprite_rect);
        
        if self.player.vel.x < 0.0 {
            p.scale.x *= -1.0;
            p.dest.x += self.player_sprite_width as f32;
        }
        
        self.sprite_batch.add(p);
        
        let on_screen = DrawParam::new()
            .dest(self.player.pos);
        
        graphics::draw(ctx, &self.sprite_batch, on_screen)?;

        let text = graphics::Text::new("Score ".to_string() + &(self.player.score.to_string()));

        graphics::draw(ctx, &text, (Point2 {x: 10.0, y: 50.0},))?;
        graphics::draw(ctx, &self.ground_mesh, (Point2 {x: 0.0, y: 0.0},))?;

        let player_rect = Rect::new(self.player.pos.x - self.coin_spawner.coin_radius/2.0, self.player.pos.y - self.coin_spawner.coin_radius/2.0, self.player_sprite_width + self.coin_spawner.coin_radius/2.0, self.player_sprite_height + self.coin_spawner.coin_radius/2.0);
        let r = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::stroke(1.0), player_rect, graphics::WHITE)?;
        graphics::draw(ctx, &r, DrawParam::default())?;
        self.sprite_batch.clear();

        self.coin_spawner.draw(ctx);

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::D => {
                if self.player.state != PlayerState::Jump {
                    self.player.state = PlayerState::Walk;
                }
                self.player.vel.x = 1.0;
                
            },
            KeyCode::Q => {
                if self.player.state != PlayerState::Jump {
                    self.player.state = PlayerState::Walk;
                }
                self.player.vel.x = -1.0;
            }
            KeyCode::Space => {
                if self.player.state != PlayerState::Jump {
                    self.player.state = PlayerState::Jump;
                    self.player.vel.y = -4.0;
                }
            }
            _ => (),
        }
    }

    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymods: KeyMods) {
        match keycode {
            KeyCode::D => {
                if self.player.state != PlayerState::Jump{
                    self.player.state = PlayerState::Stand;
                    
                }
                if !keyboard::is_key_pressed(ctx, KeyCode::Q) {
                    self.player.vel.x = 0.0;
                }
                
            },
            KeyCode::Q => {
                if self.player.state != PlayerState::Jump{
                    self.player.state = PlayerState::Stand;
                }
                if !keyboard::is_key_pressed(ctx, KeyCode::D) {
                    self.player.vel.x = 0.0;
                }

            },
            KeyCode::P => {
                self.coin_spawner.reset_timer();
            }
            _ => (),
        }
    }
}

impl Player {
    fn new() -> Player {
        Player {
            state: PlayerState::Stand,
            vel: Vector2 {
                x: 0.0,
                y: 0.0,
            },
            pos: Point2 {
                x: 100.0,
                y: 200.0,
            },
            score: 0,
            walk_frame: 0,
            last_swap: 0,
            frame_duration: 40, //ms
        }
    }
}



fn main() {
    println!("Hello, world!");
    
    let resource_dir = path::PathBuf::from("./resources");

    let c = conf::Conf::new();
    let ws = ggez::conf::WindowSetup {
        title: "My Game for bae".to_string(),
        samples: ggez::conf::NumSamples::One,
        vsync: true,
        icon: "".to_string(),
        srgb: true
    };
    let wm = ggez::conf::WindowMode::default();
    wm.dimensions(720.0, 1080.0);
    //c.window_setup = ws;
    //c.window_mode = wm;
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("hello_ggez", "frank_bryden")
        .window_mode(ggez::conf::WindowMode::default()
                    .fullscreen_type(conf::FullscreenType::Windowed)
                    .resizable(true),)
        .window_setup(ggez::conf::WindowSetup::default()
                    .vsync(true)
                    .title("Bae I miss you"))                      
        .add_resource_path(resource_dir)
        //.conf(c)
        .build()
        .unwrap();
    println!("{:?}", ctx);
    
    let state = &mut State::new(ctx).unwrap();

    event::run(ctx, event_loop, state).unwrap();
}
