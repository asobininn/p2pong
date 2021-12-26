use bevy::{prelude::*, sprite::collide_aabb::*};
use bevy_ggrs::*;
use ggrs::*;
use std::hash::Hash;

const PADDLE_SPEED: f32 = 4.;
const BALL_SPEED: f32 = 6.;
const HORIZONTAL_WALL_WIDTH: f32 = 10.;
const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
pub const LEFT_PADDLE: f32 = 1.;
pub const RIGHT_PADDLE: f32 = -1.;
pub const NUM_PLAYERS: usize = 2;

// region: component
#[derive(Default, Component)]
pub struct Paddle {
	pub handle: u32,
}

#[derive(Component)]
pub struct Ball {
	pub velocity: Vec3,
}

#[derive(Component)]
pub enum Collider {
	Solid,
	LeftGoal,
	RightGoal,
}


#[derive(Default, Reflect, Hash, Component)]
pub struct FrameCount {
	pub frame: u32,
}
// endregion: component


// region: resource
pub struct ActiveBalls(pub i32);

pub struct Scoreboard {
	pub left: u32,
	pub right: u32,
}

pub struct LastWinner(pub f32);

pub struct WinSize {
	pub w: f32,
	pub h: f32,
}
// endregion: resource


// region: system
pub fn setup_system(
	mut commands: Commands,
	mut rip: ResMut<RollbackIdProvider>,
	windows: Res<Windows>,
	asset_server: Res<AssetServer>,
) {
	let window = windows.get_primary().unwrap();
	let win_size = WinSize {
		w: window.width(),
		h: window.height(),
	};

	// camera
	commands.spawn_bundle(OrthographicCameraBundle::new_2d());
	commands.spawn_bundle(UiCameraBundle::default());

	// wall
	let spawn_horizontal_wall = |commands: &mut Commands, y: f32| {
		commands
			.spawn_bundle(SpriteBundle {
				transform: Transform {
					translation: Vec3::new(0., y, 0.),
					scale: Vec3::new(win_size.w, HORIZONTAL_WALL_WIDTH, 0.),
					..Default::default()
				},
				sprite: Sprite {
					color: Color::DARK_GRAY,
					..Default::default()
				},
				..Default::default()
			})
			.insert(Collider::Solid);
	};
	let spawn_virtical_wall = |commands: &mut Commands, x: f32, collider: Collider| {
		commands
			.spawn_bundle(SpriteBundle {
				transform: Transform {
					translation: Vec3::new(x, 0., 0.),
					scale: Vec3::new(10., win_size.h, 0.),
					..Default::default()
				},
				sprite: Sprite {
					color: Color::BLACK,
					..Default::default()
				},
				..Default::default()
			})
			.insert(collider);
	};
	// left
	spawn_virtical_wall(&mut commands, -(win_size.w / 2.), Collider::LeftGoal);
	// right
	spawn_virtical_wall(&mut commands, win_size.w / 2., Collider::RightGoal);
	// top
	spawn_horizontal_wall(&mut commands, win_size.h / 2.);
	// bottom
	spawn_horizontal_wall(&mut commands, -(win_size.h / 2.));
	
	// line
	commands
		.spawn_bundle(SpriteBundle {
			transform: Transform {
				translation: Vec3::new(0., 0., 0.),
				scale: Vec3::new(1., win_size.h, 0.),
				..Default::default()
			},
			sprite: Sprite {
				color: Color::DARK_GRAY,
				..Default::default()
			},
			..Default::default()
		});

	// paddle
	let mut dir = -1.;
	let x_diff = win_size.w / 2.5;
	for handle in 0..(NUM_PLAYERS as u32) {
		commands
			.spawn_bundle(SpriteBundle {
				transform: Transform {
					translation: Vec3::new(x_diff * dir, 0., 1.),
					scale: Vec3::new(15., 70., 0.),
					..Default::default()
				},
				sprite: Sprite {
					color: Color::WHITE,
					..Default::default()
				},
				..Default::default()
			})
			.insert(Paddle {handle})
			.insert(Collider::Solid)
			.insert(Rollback::new(rip.next_id()));
		dir *= -1.;
	}

	// scoreboard
	let font = asset_server.load("fonts/FiraMono-Medium.ttf");
	commands
		.spawn_bundle(TextBundle {
			text: Text {
				sections: vec![
					TextSection {
						value: "0".to_string(),
						style: TextStyle {
							font: font.clone(),
							font_size: 100.,
							color: Color::WHITE,
						},
					},
					TextSection {
						value: "          0".to_string(),
						style: TextStyle {
							font,
							font_size: 100.,
							color: Color::WHITE,
						},
					},
				],
				..Default::default()
			},
			style: Style {
				position_type: PositionType::Absolute,
				position: Rect {
					top: Val::Px(win_size.h / 8.),
					left: Val::Px(win_size.w / 4.),
					..Default::default()
				},
				..Default::default()
			},
			..Default::default()
		});

	// resoureces
	commands.insert_resource(Scoreboard{ left: 0, right: 0});
	commands.insert_resource(win_size);
}

pub fn spawn_ball_system(
	mut commands: Commands,
	mut rip: ResMut<RollbackIdProvider>,
	mut active_balls: ResMut<ActiveBalls>,
	last_winner: Res<LastWinner>,
) {
	if active_balls.0 < 1 {
		commands
			.spawn_bundle(SpriteBundle {
				transform: Transform {
					translation: Vec3::new(0., 0., 1.),
					scale: Vec3::new(10., 10., 0.),
					..Default::default()
				},
				sprite: Sprite {
					color: Color::YELLOW,
					..Default::default()
				},
				..Default::default()
			})
			.insert(Ball {
				velocity: BALL_SPEED * Vec3::new(0.5, 0.5, 0.).normalize() * last_winner.0,
			})
			.insert(Rollback::new(rip.next_id()));
		active_balls.0 += 1;
	}
}

pub fn move_paddle_system(
	mut query: Query<(&mut Transform, &Paddle), With<Rollback>>,
	win_size: Res<WinSize>,
	inputs: Res<Vec<GameInput>>,
) {
	for(mut ts, paddle) in query.iter_mut() {
		let input = inputs[paddle.handle as usize].buffer[0];
		let direction = match input {
			INPUT_UP => 1.,
			INPUT_DOWN => -1.,
			_ => 0.,
		};
		let paddle_y_length: f32 = ts.scale.y.clone();
		let translation = &mut ts.translation;
		let bound = win_size.h / 2. - paddle_y_length / 2. - HORIZONTAL_WALL_WIDTH / 2.;
		translation.y += direction * PADDLE_SPEED;
		translation.y = translation.y
			.min(bound)
			.max(-bound);
	}
}

pub fn move_ball_system(
	mut ball_query: Query<(&Ball, &mut Transform)>,
) {
	if let Ok((ball, mut transform)) = ball_query.get_single_mut() {
		transform.translation += ball.velocity;
	}
}

pub fn ball_collision_system(
	mut commands: Commands,
	mut scoreboard: ResMut<Scoreboard>,
	mut active_balls: ResMut<ActiveBalls>,
	mut last_winner: ResMut<LastWinner>,
	mut ball_query: Query<(Entity, &mut Ball, &Transform)>,
	collider_query: Query<(&Collider, &Transform)>,
) {
	if let Ok((ball_entity, mut ball, ball_transform)) = ball_query.get_single_mut() {
		let ball_size = ball_transform.scale.truncate();
		let velocity = &mut ball.velocity;

		for (collider, transform) in collider_query.iter() {
			let collision = collide(
				ball_transform.translation,
				ball_size,
				transform.translation,
				transform.scale.truncate(),
			);
			if let Some(collision) = collision {
				if let Collider::LeftGoal = *collider {
					scoreboard.right += 1;
					commands.entity(ball_entity).despawn();
					active_balls.0 -= 1;
					last_winner.0 = RIGHT_PADDLE;
					break;
				} else if let Collider::RightGoal = *collider {
					scoreboard.left += 1;
					commands.entity(ball_entity).despawn();
					active_balls.0 -= 1;
					last_winner.0 = LEFT_PADDLE;
					break;
				} 

				let mut reflect_x = false;
				let mut reflect_y = false;
				match collision {
					Collision::Left => reflect_x = velocity.x > 0.0,
					Collision::Right => reflect_x = velocity.x < 0.0,
					Collision::Top => reflect_y = velocity.y < 0.0,
					Collision::Bottom => reflect_y = velocity.y > 0.0,
				}
				if reflect_x {
					velocity.x = -velocity.x;
				}
				if reflect_y {
					velocity.y = -velocity.y;
				}
				break;
			}
		}
	}
}

pub fn scoreboard_system(
	scoreboard: Res<Scoreboard>,
	mut query: Query<&mut Text>,
) {
	let mut text = query.single_mut();
	text.sections[0].value = format!("{}", scoreboard.left);
	text.sections[1].value = format!("          {}", scoreboard.right);
}

pub fn increase_frame_system(mut frame_count: ResMut<FrameCount>) {
	frame_count.frame += 1;
}
// endregion: system

pub fn input(
	_handle: In<PlayerHandle>,
	keyboard_input: Res<Input<KeyCode>>,
) -> Vec<u8> {
	let mut input: u8 = 0;

	if keyboard_input.pressed(KeyCode::W) ||
			keyboard_input.pressed(KeyCode::Up) {
		input |= INPUT_UP;
	}
	if keyboard_input.pressed(KeyCode::S) ||
			keyboard_input.pressed(KeyCode::Down) {
		input |= INPUT_DOWN;
	}

	vec![input]
}