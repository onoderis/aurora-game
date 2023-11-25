use std::time::Duration;
use bevy::math::{vec3};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, input_to_event)
        .add_systems(Update, player_move_event_handler)
        .add_systems(Update, gravity)
        .add_systems(Update, start_jump)
        .add_systems(Update, jump_lift)
        .add_event::<PlayerMoveEvent>()
        .add_event::<PlayerJumpEvent>()
        .run();
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    sprite_bundle: SpriteBundle,
}

/// Controllable entity by a player.
#[derive(Component)]
struct Player {
    jumping_timer: Option<Timer>,
}

/// Obstacle for a player.
#[derive(Component)]
struct Obstacle;

#[derive(Event)]
struct PlayerMoveEvent {
    direction: MoveDirection
}

#[derive(Event)]
struct PlayerJumpEvent;

#[derive(Copy, Clone)]
enum MoveDirection {
    Left,
    Right,
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Player
    commands.spawn(
        PlayerBundle {
            player: Player { jumping_timer: None },
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0., 10.),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(100., 100., 0.),
                },
                sprite: Sprite {
                    color: Color::GOLD,
                    ..default()
                },
                ..default()
            },
        }
    );

    // Floor
    commands.spawn((
        Obstacle {},
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0., -300., 0.),
                scale: Vec3::new(10000., 20., 0.),
                ..default()
            },
            sprite: Sprite {
                color: Color::YELLOW_GREEN,
                ..default()
            },
            ..default()
        },
    ));
}

fn input_to_event(
    key_input: Res<Input<KeyCode>>,
    mut event_player_move: EventWriter<PlayerMoveEvent>,
    mut event_player_jump: EventWriter<PlayerJumpEvent>,
) {
    if key_input.pressed(KeyCode::Space) {
        event_player_jump.send(PlayerJumpEvent)
    }
    if key_input.pressed(KeyCode::A) {
        event_player_move.send(PlayerMoveEvent { direction: MoveDirection::Left })
    }
    if key_input.pressed(KeyCode::D) {
        event_player_move.send(PlayerMoveEvent { direction: MoveDirection::Right })
    }
}

fn player_move_event_handler(
    mut event_player_move: EventReader<PlayerMoveEvent>,
    mut player: Query<&mut Transform, (With<Player>, Without<Obstacle>)>,
    obstacles: Query<&Transform, (With<Obstacle>, Without<Player>)>,
) {
    for event in event_player_move.read() {
        for mut player_transform in player.iter_mut() {
            for obstacle_transform in obstacles.iter() {
                let new_player_pos = player_transform.translation + map_direction_to_vec3(event.direction);
                let opt_collision = collide(
                    new_player_pos,
                    player_transform.scale.xy(),
                    obstacle_transform.translation,
                    obstacle_transform.scale.truncate()
                );
                if opt_collision.is_none() {
                    player_transform.translation = new_player_pos;
                }
            }
        }
    }
}

fn gravity(
    mut player: Query<&mut Transform, (With<Player>, Without<Obstacle>)>,
    obstacles: Query<&Transform, (With<Obstacle>, Without<Player>)>,
) {
    for mut player_transform in player.iter_mut() {
        for obstacle_transform in obstacles.iter() {
            let new_player_pos = player_transform.translation + vec3(0., -5., 0.);
            let opt_collision = collide(
                new_player_pos,
                player_transform.scale.xy(),
                obstacle_transform.translation,
                obstacle_transform.scale.xy()
            );
            if opt_collision.is_none() {
                player_transform.translation = new_player_pos;
            } else {
                // ground the player
                player_transform.translation.y = obstacle_transform.translation.y +
                    obstacle_transform.scale.y / 2.0 + player_transform.scale.y / 2.0;
            }
        }
    }
}

fn start_jump(
    mut event_jump: EventReader<PlayerJumpEvent>,
    mut players: Query<&mut Player>,
) {
    for _ in event_jump.read() {
        for mut player in players.iter_mut() {
            player.jumping_timer = Some(Timer::from_seconds(0.5, TimerMode::Once));
        }
    }
}

fn jump_lift(
    mut players: Query<(&mut Transform, &mut Player)>,
    time: Res<Time>,
) {
    for (mut transform, mut player) in players.iter_mut() {
        if let Some(timer) = player.jumping_timer.as_mut() {
            timer.tick(time.delta());
            if timer.remaining() > Duration::ZERO {
                transform.translation.y += 11.0; // 6 + gravity
            }
        }
    }
}



// util functions

fn map_direction_to_vec3(direction: MoveDirection) -> Vec3 {
    match direction {
        MoveDirection::Left => vec3(-5., 0., 0.),
        MoveDirection::Right => vec3(5., 0., 0.),
    }
}
