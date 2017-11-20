use std::rc::Rc;

use cgmath::{self, Vector2};
use cgmath::prelude::*;
use midgar::{self, KeyCode, Midgar, Surface};
use midgar::graphics::sprite::{DrawTexture, MagnifySamplerFilter, SpriteDrawParams, SpriteRenderer};
use midgar::graphics::texture::TextureRegion;

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
                pos: cgmath::vec2(67.0, 183.0),
            },
            player2: Paddle {
                pos: cgmath::vec2(577.0, 187.0),
            },
            ball: Ball {
                pos: cgmath::vec2(320.0, 188.0),
            },

            sprite: SpriteRenderer::new(midgar.graphics().display(), projection),
            left_tex,
            right_tex,
            ball_tex,
            separator_tex,
        }
    }

    fn step(&mut self, midgar: &mut Midgar) {
        if midgar.input().was_key_pressed(KeyCode::Escape) {
            midgar.set_should_exit();
            return;
        }

        let dt = midgar.time().delta_time();

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

struct Paddle {
    pos: Vector2<f32>,
}

struct Ball {
    pos: Vector2<f32>,
}
