use std::rc::Rc;

use cgmath::{self, Vector2};
use cgmath::prelude::*;
use midgar::{self, KeyCode, Midgar, Surface};
use midgar::graphics::sprite::{DrawTexture, MagnifySamplerFilter, SpriteDrawParams, SpriteRenderer};
use midgar::graphics::texture::TextureRegion;
use rand;

const SCREEN_SIZE: Vector2<f32> = Vector2 {
    x: 640.0,
    y: 400.0,
};

// Initial ball speed (pixels/second)
const INITIAL_BALL_SPEED: f32 = 80.0;
// Paddle speed (pixels/second)
const PADDLE_SPEED: f32 = 150.0;

pub struct GameApp {
    player1: Paddle,
    player2: Paddle,
    ball: Ball,

    sprite: SpriteRenderer,
    left_tex: TextureRegion,
    right_tex: TextureRegion,
    ball_tex: TextureRegion,
    separator_tex: TextureRegion,
}

impl midgar::App for GameApp {
    fn create(midgar: &Midgar) -> Self {
        let left_tex = {
            let texture = Rc::new(midgar.graphics().load_texture("assets/left_pallete.png", true));
            TextureRegion::new(texture)
        };
        let right_tex = {
            let texture = Rc::new(midgar.graphics().load_texture("assets/right_pallete.png", true));
            TextureRegion::new(texture)
        };
        let ball_tex = {
            let texture = Rc::new(midgar.graphics().load_texture("assets/ball.png", true));
            TextureRegion::new(texture)
        };
        let separator_tex = {
            let texture = Rc::new(midgar.graphics().load_texture("assets/separator.png", true));
            TextureRegion::new(texture)
        };
        let projection = cgmath::ortho(0.0, 640.0,
                                       400.0, 0.0,
                                       -1.0, 1.0);

        GameApp {
            player1: Paddle {
                pos: cgmath::vec2(67.0, 200.0),
            },
            player2: Paddle {
                pos: cgmath::vec2(577.0, 200.0),
            },
            ball: Ball {
                pos: cgmath::vec2(320.0, 200.0),
                direction: cgmath::vec2(-1.0, 0.0),
                speed: INITIAL_BALL_SPEED,
            },

            sprite: SpriteRenderer::new(midgar.graphics().display(), projection),
            left_tex,
            right_tex,
            ball_tex,
            separator_tex,
        }
    }

    fn step(&mut self, midgar: &mut Midgar) {
        // Update!
        if midgar.input().was_key_pressed(KeyCode::Escape) {
            midgar.set_should_exit();
            return;
        }

        let dt = midgar.time().delta_time() as f32;

        // Integrate new ball position
        self.ball.pos += self.ball.direction * self.ball.speed * dt;

        // Flip when touching roof or floor
        if (self.ball.pos.y < 0.0 && self.ball.direction.y < 0.0) ||
           (self.ball.pos.y > SCREEN_SIZE.y && self.ball.direction.y > 0.0) {
            self.ball.direction.y = -self.ball.direction.y
        }

        // Flip, change direction and increase speed when touching pads
        if (rect_has_point(self.player1.pos, self.left_tex.size().cast::<f32>(), self.ball.pos) && self.ball.direction.x < 0.0) ||
           (rect_has_point(self.player2.pos, self.right_tex.size().cast::<f32>(), self.ball.pos) && self.ball.direction.x > 0.0) {
            self.ball.direction.x = -self.ball.direction.x;
            self.ball.direction.y = rand::random::<f32>() * 2.0 - 1.0;
            self.ball.direction = self.ball.direction.normalize();
            self.ball.speed *= 1.1;
        }

        // Check game over
        if self.ball.pos.x < 0.0 || self.ball.pos.x > SCREEN_SIZE.x {
            self.ball.pos = SCREEN_SIZE * 0.5;
            self.ball.speed = INITIAL_BALL_SPEED;
            self.ball.direction = cgmath::vec2(-1.0, 0.0);
        }

        // Move left paddle
        if self.player1.pos.y > 0.0 && midgar.input().is_key_held(KeyCode::Z) {
            self.player1.pos.y += -PADDLE_SPEED * dt;
        }
        if self.player1.pos.y < SCREEN_SIZE.y && midgar.input().is_key_held(KeyCode::X) {
            self.player1.pos.y += PADDLE_SPEED * dt;
        }

        // Move right paddle
        if self.player2.pos.y > 0.0 && midgar.input().is_key_held(KeyCode::Right) {
            self.player2.pos.y += -PADDLE_SPEED * dt;
        }
        if self.player2.pos.y < SCREEN_SIZE.y && midgar.input().is_key_held(KeyCode::Left) {
            self.player2.pos.y += PADDLE_SPEED * dt;
        }

        // Render!
        let mut target = midgar.graphics().display().draw();

        target.clear_color(0.0, 0.0, 0.0, 1.0);

        let draw_params = SpriteDrawParams::new()
            .magnify_filter(MagnifySamplerFilter::Nearest)
            .alpha(true);

        // Draw separator.
        self.sprite.draw(&self.separator_tex.draw(320.0, 200.0),
                         draw_params, &mut target);
        // Draw player 1.
        self.sprite.draw(&self.left_tex.draw(self.player1.pos.x, self.player1.pos.y),
                         draw_params, &mut target);
        // Draw player 2.
        self.sprite.draw(&self.right_tex.draw(self.player2.pos.x, self.player2.pos.y),
                         draw_params, &mut target);
        // Draw ball.
        self.sprite.draw(&self.ball_tex.draw(self.ball.pos.x, self.ball.pos.y),
                         draw_params, &mut target);

        target.finish()
            .expect("target.finish() failed");
    }
}

fn rect_has_point(rect_pos: Vector2<f32>, rect_size: Vector2<f32>, point: Vector2<f32>) -> bool {
    let left = rect_pos.x - rect_size.x / 2.0;
    let right = rect_pos.x + rect_size.x / 2.0;
    let top = rect_pos.y - rect_size.y / 2.0;
    let bottom = rect_pos.y + rect_size.y / 2.0;

    left <= point.x && right >= point.x && top <= point.y && bottom >= point.y
}

struct Paddle {
    pos: Vector2<f32>,
}

struct Ball {
    pos: Vector2<f32>,
    direction: Vector2<f32>,
    speed: f32,
}
