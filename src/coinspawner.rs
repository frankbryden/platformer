use ggez::{Context, GameResult};
use ggez::graphics::{Rect, DrawParam, Drawable};
use ggez::graphics::spritebatch::SpriteBatch;
use ggez::timer;
use ggez::graphics;
use ggez::nalgebra;

use rand::rngs::ThreadRng;
use rand::Rng;

use nalgebra as na;

const GRAVITY: f32 = 1.0;
const COIN_MAX_BOUNCES: usize = 7;

pub struct ItemSet {
    sprite_batch: SpriteBatch,
	gold_coin_def: Rect,
	gold_coin_center: na::Point2<f32>,
	silver_coin_def: Rect,
	silver_coin_center: na::Point2<f32>,
}



pub enum CoinType {
    Silver,
    Gold
}


pub struct CoinSpawner {
    item_set: ItemSet,
    spawn_timer: i32, //Timer to keep track of where we are
    spawn_time: i32,  //Time between coin spawns
    pub coins: Vec<Coin>,
    world_width: f32,
	world_height: f32,
	ground: f32,
	rng: ThreadRng,
	pub coin_radius: f32,
	rot: f32,
}

pub struct Coin {
    pub pos: na::Point2<f32>,
	vel: na::Vector2<f32>,
	torque: f32,
	angle: f32,
	bounce_count: usize,
	coin_type: CoinType,
}

impl CoinSpawner {
	pub fn new(ctx: &mut Context, world_width: i32, world_height: i32, ground: f32) -> CoinSpawner {
		CoinSpawner {
			item_set: ItemSet::new(ctx).unwrap(),
            spawn_timer: 0,
            spawn_time: 100,
            coins: Vec::new(),
            rng: rand::thread_rng(),
            world_width: world_width as f32,
			world_height: world_height as f32,
			ground: ground,
			coin_radius: 70.0,
			rot: 0.0,
        }
	}
    pub fn update(&mut self, ctx: &mut Context) {
        self.spawn_timer -= timer::delta(ctx).as_millis() as i32;
        if self.spawn_timer <= 0 {
            self.spawn_timer = self.spawn_time;
            for _ in 0..2 {
                self.spawn_coin(ctx);
            }
            println!("Spawn coin! {}", self.coins.len());
        }
        for coin in &mut self.coins{
            coin.pos.x += coin.vel.x;
			coin.pos.y += coin.vel.y;
			coin.vel.y += GRAVITY;
			coin.angle += coin.torque;
			if coin.pos.x + self.coin_radius > self.world_width || coin.pos.x < 0.0 {
				coin.vel.x *= -1.0;
				coin.torque *= -1.0;
			}

			if coin.pos.y + self.coin_radius > self.ground {
				coin.vel.y *= -0.7;
				coin.pos.y = self.ground - self.coin_radius;
				println!("Clamping coin to y = {}", coin.pos.y);
				coin.bounce_count += 1;
			}
			
			if coin.bounce_count >= COIN_MAX_BOUNCES {
				coin.vel.y = 0.0;
				//coin.vel.x *= 0.6;
			}
        }
    }

    pub fn draw(&mut self, ctx: &mut Context) {
        self.item_set.sprite_batch.clear();
        for coin in &self.coins {
            let (sprite_def, offset) = match coin.coin_type {
                CoinType::Gold => {
                    (self.item_set.gold_coin_def, self.item_set.gold_coin_center)
                },
                CoinType::Silver => {
                    (self.item_set.silver_coin_def, self.item_set.silver_coin_center)
                }
            };
            let p = DrawParam::new()
                .src(sprite_def)
				.dest(coin.pos)
				.offset(offset)
				.rotation(coin.angle);
			self.item_set.sprite_batch.add(p);
			
            //self.item_set.sprite_batch.add((coin.pos,));
		}
        graphics::draw(ctx, &self.item_set.sprite_batch, (na::Point2::new(0.0, 0.0),)).unwrap();
    }

    pub fn spawn_coin(&mut self, ctx: &mut Context) {
        let vel_x = (self.rng.gen::<f32>() * 6.0) - 3.0;
        let vel_y = self.rng.gen::<f32>() * 5.0 + 3.0;
        let (width, _) = graphics::drawable_size(ctx);
        let x = width * self.rng.gen::<f32>() * 0.8 + (0.1*width);
        let y = - 10.0;
        let coin_type = if self.rng.gen::<f32>() < 0.5 {
            CoinType::Gold
        } else {
            CoinType::Silver
        };
        let new_coin = Coin {
            vel: na::Vector2::new(vel_x, vel_y),
			pos: na::Point2::new(x, y),
			torque: vel_x / 100.0,
			angle: 0.0,
			bounce_count: 0,
            coin_type: coin_type,
        };
        self.coins.push(new_coin);
	}
	
	pub fn reset_timer(&mut self){
		self.spawn_timer = 1;
	}
}

impl ItemSet {
    fn new(ctx: &mut Context) -> GameResult<ItemSet> {
        let image = graphics::Image::new(ctx, "/Items/items_spritesheet.png").unwrap();
        //let image = graphics::Image::new(ctx, "/Items/coinGold.png").unwrap();

        let width = image.width() as f32;
        let height = image.height() as f32;

        let sprite_batch = SpriteBatch::new(image);

		let mut gold_coin_def = Rect::new(288.0, 360.0, 70.0, 70.0);
		let mut silver_coin_def = Rect::new(288.0, 288.0, 70.0, 70.0);

		let mut gold_coin_center = na::Point2::new(gold_coin_def.x + gold_coin_def.w/2.0, gold_coin_def.y + gold_coin_def.h/2.0);
		let mut silver_coin_center = na::Point2::new(silver_coin_def.x, silver_coin_def.y);

		println!("{:?}", gold_coin_center);

        gold_coin_def.x /= width;
        gold_coin_def.y /= height;
        gold_coin_def.w /= width;
        gold_coin_def.h /= height;

        silver_coin_def.x /= width;
        silver_coin_def.y /= height;
        silver_coin_def.w /= width;
		silver_coin_def.h /= height;
		
		gold_coin_center.x /= width;
		gold_coin_center.y /= height;

		silver_coin_center.x /= width;
		silver_coin_center.y /= height;

        Ok(ItemSet {
            sprite_batch: sprite_batch,
            gold_coin_def: gold_coin_def,
			silver_coin_def: silver_coin_def,
			gold_coin_center: gold_coin_center,
			silver_coin_center: silver_coin_center,
        })
    }
}