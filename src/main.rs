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
        .add_event::<PlayerMoveEvent>()
        .run();
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    sprite_bundle: SpriteBundle,
}

/// Controllable entity by a player.
#[derive(Component)]
struct Player;

/// Obstacle for a player.
#[derive(Component)]
struct Obstacle;

#[derive(Event)]
struct PlayerMoveEvent {
    direction: MoveDirection
}

#[derive(Copy, Clone)]
enum MoveDirection {
    Up,
    Down,
    Left,
    Right,
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Player
    commands.spawn(
        PlayerBundle {
            player: Player {},
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
) {
    if key_input.pressed(KeyCode::W) {
        event_player_move.send(PlayerMoveEvent { direction: MoveDirection::Up })
    }
    if key_input.pressed(KeyCode::S) {
        event_player_move.send(PlayerMoveEvent { direction: MoveDirection::Down })
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
                let collide = is_collide(new_player_pos, player_transform.scale, obstacle_transform.clone());
                if !collide {
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
            let collide = is_collide(new_player_pos, player_transform.scale, obstacle_transform.clone());
            if !collide {
                player_transform.translation = new_player_pos;
            }
        }
    }
}



// util functions

fn is_collide(translation1: Vec3, scale1: Vec3, t2: Transform) -> bool {
    return collide(
        translation1,
        scale1.truncate(),
        t2.translation,
        t2.scale.truncate()
    ).is_some();
}

fn map_direction_to_vec3(direction: MoveDirection) -> Vec3 {
    match direction {
        MoveDirection::Up => vec3(0., 3., 0.),
        MoveDirection::Down => vec3(0., -3., 0.),
        MoveDirection::Left => vec3(-3., 0., 0.),
        MoveDirection::Right => vec3(3., 0., 0.),
    }
}
