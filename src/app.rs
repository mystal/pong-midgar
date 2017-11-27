use std::rc::Rc;

use cgmath::{self, Vector2};
use cgmath::prelude::*;
use midgar::{self, KeyCode, Midgar, Surface};
use midgar::graphics::sprite::{DrawTexture, MagnifySamplerFilter, SpriteDrawParams, SpriteRenderer};
use midgar::graphics::text::{self, Font, TextRenderer};
use midgar::graphics::texture::TextureRegion;
use noise::{NoiseModule, Perlin, Seedable};
use rand;

const SCREEN_SIZE: Vector2<f32> = Vector2 {
    x: 640.0,
    y: 400.0,
};

// Initial ball speed (pixels/second)
const INITIAL_BALL_SPEED: f32 = 100.0;
// One day const fn will be stable...
//const INITIAL_BALL_POS: Vector2<f32> = SCREEN_SIZE * 0.5;
const INITIAL_BALL_POS: Vector2<f32> = Vector2 {
    x: 320.0,
    y: 200.0,
};

const INITIAL_PLAYER1_POS: Vector2<f32> = Vector2 {
    x: 67.0,
    y: 200.0,
};
const INITIAL_PLAYER2_POS: Vector2<f32> = Vector2 {
    x: 573.0,
    y: 200.0,
};
// Paddle speed (pixels/second)
const PADDLE_SPEED: f32 = 150.0;
const MAX_BALL_BOUNCE_ANGLE: f32 = 75.0;
const WINNING_SCORE: u8 = 11;

const CAMERA_SHAKE_MAX_ANGLE: f32 = 15.0;
const CAMERA_SHAKE_MAX_OFFSET: f32 = 30.0;

enum Players {
    Player1,
    Player2,
}

struct Player {
    score: u8,
    pos: Vector2<f32>,
}

impl Player {
    fn new(pos: Vector2<f32>) -> Self {
        Player {
            score: 0,
            pos,
        }
    }
}

struct Ball {
    pos: Vector2<f32>,
    direction: Vector2<f32>,
    speed: f32,
}

struct Camera {
    pos: Vector2<f32>,
    angle: f32,

    // The base magnitude value for camera shake calculations
    trauma: f32,
    noise_x: Perlin,
    noise_y: Perlin,
}

impl Camera {
    fn new(pos: Vector2<f32>) -> Self {
        let seed = rand::random();

        Camera {
            pos,
            angle: 0.0,

            trauma: 0.0,
            noise_x: Perlin::new().set_seed(seed),
            noise_y: Perlin::new().set_seed(seed + 1),
        }
    }

    fn add_trauma(&mut self, percent: f32) {
        // TODO: Warn if percent is negative?
        self.trauma += percent;
        self.trauma = f32::min(self.trauma, 1.0);
    }

    fn update_trauma(&mut self, dt: f32) {
        self.trauma -= 0.5 * dt;
        self.trauma = f32::max(self.trauma, 0.0);
    }
}

enum GameState {
    Ready,
    Playing,
    End,
}

pub struct GameApp<'a> {
    game_time: f32,
    state: GameState,

    player1: Player,
    player2: Player,
    ball: Ball,
    last_round_winner: Players,

    sprite: SpriteRenderer,
    text_renderer: TextRenderer,
    left_tex: TextureRegion,
    right_tex: TextureRegion,
    ball_tex: TextureRegion,
    separator_tex: TextureRegion,
    font: Font<'a>,

    projection: cgmath::Matrix4<f32>,
    text_projection: cgmath::Matrix4<f32>,
    camera: Camera,
}

impl<'a> GameApp<'a> {
    // Returns a value, [-1.0, 1.0], representing how far from the paddle center the ball hit.
    fn did_ball_hit_paddle(&self) -> Option<f32> {
        if rect_has_point(self.player1.pos, self.left_tex.size().cast::<f32>(), self.ball.pos) && self.ball.direction.x < 0.0 {
            let distance = self.ball.pos.y - self.player1.pos.y;
            Some(distance / (self.left_tex.size().y as f32 / 2.0))
        } else if rect_has_point(self.player2.pos, self.right_tex.size().cast::<f32>(), self.ball.pos) && self.ball.direction.x > 0.0 {
            let distance = self.ball.pos.y - self.player2.pos.y;
            Some(distance / (self.right_tex.size().y as f32 / 2.0))
        } else {
            None
        }
    }
}

impl<'a> midgar::App for GameApp<'a> {
    fn create(midgar: &Midgar) -> Self {
        // Load textures
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

        let projection = cgmath::ortho(-SCREEN_SIZE.x / 2.0, SCREEN_SIZE.x / 2.0,
                                       SCREEN_SIZE.y / 2.0, -SCREEN_SIZE.y / 2.0,
                                       -1.0, 1.0);
        let text_projection = cgmath::ortho(0.0, SCREEN_SIZE.x,
                                            SCREEN_SIZE.y, 0.0,
                                            -1.0, 1.0);

        // Randomize ball's starting direction
        let ball_x_dir = if rand::random() {
            1.0
        } else {
            -1.0
        };
        // TODO: Randomize the ball's starting angle
        //self.ball.direction.y = rand::random::<f32>() * 2.0 - 1.0;
        //self.ball.direction = self.ball.direction.normalize();

        GameApp {
            game_time: 0.0,
            state: GameState::Ready,

            player1: Player::new(INITIAL_PLAYER1_POS),
            player2: Player::new(INITIAL_PLAYER2_POS),
            ball: Ball {
                pos: INITIAL_BALL_POS,
                direction: cgmath::vec2(ball_x_dir, 0.0),
                speed: INITIAL_BALL_SPEED,
            },
            last_round_winner: Players::Player1,

            sprite: SpriteRenderer::new(midgar.graphics().display(), projection),
            text_renderer: TextRenderer::new(midgar.graphics().display()),
            left_tex,
            right_tex,
            ball_tex,
            separator_tex,
            font: text::load_font_from_path("assets/VeraMono.ttf"),

            projection,
            text_projection,
            camera: Camera::new(SCREEN_SIZE * 0.5),
        }
    }

    fn step(&mut self, midgar: &mut Midgar) {
        // Update!
        if midgar.input().was_key_pressed(KeyCode::Escape) {
            midgar.set_should_exit();
            return;
        }

        // Check if we should resize the screen
        if midgar.input().was_key_pressed(KeyCode::Num1) {
            let scale = 1;
            midgar.graphics_mut().set_size(SCREEN_SIZE.x as u32 * scale, SCREEN_SIZE.y as u32 * scale);
        } else if midgar.input().was_key_pressed(KeyCode::Num2) {
            let scale = 2;
            midgar.graphics_mut().set_size(SCREEN_SIZE.x as u32 * scale, SCREEN_SIZE.y as u32 * scale);
        } else if midgar.input().was_key_pressed(KeyCode::Num3) {
            let scale = 3;
            midgar.graphics_mut().set_size(SCREEN_SIZE.x as u32 * scale, SCREEN_SIZE.y as u32 * scale);
        }

        // FIXME: Remove this debug feature.
        if midgar.input().was_key_pressed(KeyCode::Tab) {
            self.camera.add_trauma(0.5);
        }

        let dt = midgar.time().delta_time() as f32;
        self.game_time += dt;
        let (message, dt) = match self.state {
            GameState::Ready => {
                if midgar.input().was_key_pressed(KeyCode::Space) {
                    self.state = GameState::Playing;
                }
                ("Press Space to play!", 0.0)
            },
            GameState::Playing => {
                if midgar.input().was_key_pressed(KeyCode::Space) {
                    self.state = GameState::Playing;
                }
                ("", dt)
            },
            GameState::End => {
                // TODO: Don't check for player scoring!
                if midgar.input().was_key_pressed(KeyCode::Space) {
                    self.state = GameState::Playing;

                    // Reset the ball
                    self.ball.pos = INITIAL_BALL_POS;
                    self.ball.speed = INITIAL_BALL_SPEED;

                    // Reset the players
                    self.player1.pos = INITIAL_PLAYER1_POS;
                    self.player1.score = 0;
                    self.player2.pos = INITIAL_PLAYER2_POS;
                    self.player2.score = 0;
                }
                let message = match self.last_round_winner {
                    Players::Player1 => "Player 1 wins!\nPress Space to play again!",
                    Players::Player2 => "Player 2 wins!\nPress Space to play again!",
                };
                (message, 0.0)
            },
        };

        // Integrate new ball position
        self.ball.pos += self.ball.direction * self.ball.speed * dt;

        // Flip ball when touching roof or floor
        if (self.ball.pos.y < 0.0 && self.ball.direction.y < 0.0) ||
           (self.ball.pos.y > SCREEN_SIZE.y && self.ball.direction.y > 0.0) {
            self.ball.direction.y = -self.ball.direction.y
        }

        // Flip, change direction, and increase speed when ball touches a paddle
        if let Some(distance_ratio) = self.did_ball_hit_paddle() {
            // Bounce the ball at an angle based on where on the paddle it hit
            let new_x_multiplier = -self.ball.direction.x.signum();
            let angle = distance_ratio * MAX_BALL_BOUNCE_ANGLE.to_radians();
            self.ball.direction.x = angle.cos() * new_x_multiplier;
            self.ball.direction.y = angle.sin();
            self.ball.speed *= 1.1;
        }

        // Check if a player scored
        // TODO: Remove this hacky dt check!
        if dt > 0.0 && (self.ball.pos.x < 0.0 || self.ball.pos.x > SCREEN_SIZE.x) {
            if self.ball.pos.x < 0.0 {
                self.last_round_winner = Players::Player2;
                self.player2.score += 1;
                self.ball.direction = cgmath::vec2(-1.0, 0.0);
            } else {
                self.last_round_winner = Players::Player1;
                self.player1.score += 1;
                self.ball.direction = cgmath::vec2(1.0, 0.0);
            }

            // Camera shake!
            self.camera.add_trauma(0.5);

            // Check game over
            if self.player1.score >= WINNING_SCORE || self.player2.score >= WINNING_SCORE {
                self.state = GameState::End;
            } else {
                // Reset the ball
                self.ball.pos = INITIAL_BALL_POS;
                self.ball.speed = INITIAL_BALL_SPEED;

                // Reset the players
                self.player1.pos = INITIAL_PLAYER1_POS;
                self.player2.pos = INITIAL_PLAYER2_POS;
            }
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

        // Update camera shake!
        self.camera.update_trauma(dt);
        let shake = self.camera.trauma.powi(2);
        let shake_offset = if shake > 0.0 {
            shake * cgmath::vec2(CAMERA_SHAKE_MAX_OFFSET * self.camera.noise_x.get([3.0 * self.game_time, 3.0 * self.game_time]),
                                 CAMERA_SHAKE_MAX_OFFSET * self.camera.noise_y.get([3.0 * self.game_time, 3.0 * self.game_time]))
        } else {
            Vector2::zero()
        };
        // TODO: Add shake rotation.

        // Render!
        // Update the combined view-projection matrix!
        let camera_pos = (self.camera.pos + shake_offset).extend(0.0);
        let view = cgmath::Matrix4::look_at(cgmath::Point3::from_vec(camera_pos),
                                            cgmath::Point3::new(0.0, 0.0, -1.0) + camera_pos,
                                            cgmath::vec3(0.0, 1.0, 0.0));
        let combined = self.projection * view;
        self.sprite.set_projection_matrix(combined);

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

        // Draw each player's score.
        self.text_renderer.draw_text(&format!("{:02}", self.player1.score), &self.font, [1.0, 1.0, 1.0],
                                     20, 160.0, 30.0, 300, &self.text_projection, &mut target);
        self.text_renderer.draw_text(&format!("{:02}", self.player2.score), &self.font, [1.0, 1.0, 1.0],
                                     20, 480.0, 30.0, 300, &self.text_projection, &mut target);

        // Draw the message
        if !message.is_empty() {
            self.text_renderer.draw_text(message, &self.font, [1.0, 1.0, 1.0],
                                         20, 220.0, 140.0, 300, &self.text_projection, &mut target);
        }

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
