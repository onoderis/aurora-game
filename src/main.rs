use std::time::Duration;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::sprite::collide_aabb::{collide, Collision};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, input_to_event)
        .add_systems(Update, player_movement_intention)
        .add_systems(Update, gravity)
        .add_systems(Update, start_jump)
        .add_systems(Update, jump_lift)
        .add_systems(Update, player_movement)
        .add_event::<PlayerMoveEvent>()
        .add_event::<PlayerJumpEvent>()
        .run();
}

const GRAVITY: f32 = -9.;
const JUMP_DURATION: Duration = Duration::from_millis(750);

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    sprite_bundle: SpriteBundle,
}

/// Controllable entity by a player.
#[derive(Component)]
struct Player {
    on_ground: bool,
    jumping_timer: Option<Timer>,
    movement_vec: Vec2,
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
            player: Player {
                on_ground: false,
                jumping_timer: None,
                movement_vec: Vec2 { x: 0., y: 0. }
            },
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(-300., 0., 10.),
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
                translation: Vec3::new(0., -330., 0.),
                scale: Vec3::new(10000., 60., 0.),
                ..default()
            },
            sprite: Sprite {
                color: Color::YELLOW_GREEN,
                ..default()
            },
            ..default()
        },
    ));

    // Box
    commands.spawn((
        Obstacle {},
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(101., -200., 0.),
                scale: Vec3::new(200., 200., 0.),
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

fn player_movement_intention(
    mut event_player_move: EventReader<PlayerMoveEvent>,
    mut players: Query<&mut Player>,
) {
    let event = if let Some(event) = event_player_move.read().next() {
        event
    } else {
        return;
    };

    for mut player in players.iter_mut() {
        player.movement_vec += map_direction_to_vec2(event.direction);
    }
}

fn gravity(mut players: Query<&mut Player>) {
    for mut player in players.iter_mut() {
        player.movement_vec += vec2(0., GRAVITY);
    }
}

fn player_movement(
    mut players: Query<(&mut Player, &mut Transform), Without<Obstacle>>,
    obstacles: Query<&Transform, (With<Obstacle>, Without<Player>)>,
) {
    for (mut player, mut p_transform) in players.iter_mut() {
        for o_transform in obstacles.iter() {
            let new_player_pos = p_transform.translation + player.movement_vec.to_vec3();
            let collision_opt = collide(
                o_transform.translation,
                o_transform.scale.xy(),
                new_player_pos,
                p_transform.scale.xy(),
            );

            if let Some(collision) = collision_opt {
                match collision {
                    Collision::Left  => {
                        player.movement_vec.x = o_transform.translation.x
                            - p_transform.translation.x
                            + p_transform.scale.x / 2.
                            + o_transform.scale.x / 2.;
                    }
                    Collision::Right => {
                        player.movement_vec.x = o_transform.translation.x
                            - p_transform.translation.x
                            - p_transform.scale.x / 2.
                            - o_transform.scale.x / 2.;
                    }
                    Collision::Top => {
                        player.movement_vec.y = o_transform.translation.y
                            - p_transform.translation.y
                            - p_transform.scale.y / 2.
                            - o_transform.scale.y / 2.;
                    }
                    Collision::Bottom => {
                        player.movement_vec.y = o_transform.translation.y
                            - p_transform.translation.y
                            + p_transform.scale.y / 2.
                            + o_transform.scale.y / 2.;
                        player.on_ground = true;
                    }
                    Collision::Inside => {}
                }
            }
        }

        p_transform.translation += player.movement_vec.to_vec3();
        player.movement_vec = Vec2::ZERO;
    }
}

fn start_jump(
    mut event_jump: EventReader<PlayerJumpEvent>,
    mut players: Query<&mut Player>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if event_jump.read().next().is_none() {
        return
    }
    for mut player in players.iter_mut() {
        if !player.on_ground {
            continue;
        }
        player.jumping_timer = Some(Timer::new(JUMP_DURATION, TimerMode::Once));
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/jump.ogg"),
            ..default()
        });
    }
}

fn jump_lift(
    mut players: Query<&mut Player>,
    time: Res<Time>,
) {
    for mut player in players.iter_mut() {
        if let Some(timer) = player.jumping_timer.as_mut() {
            timer.tick(time.delta());
            if timer.remaining() > Duration::ZERO {
                player.movement_vec.y += timer.remaining().as_secs_f32() * 35.;
                player.on_ground = false;
            }
        }
    }
}



// util functions

fn map_direction_to_vec2(direction: MoveDirection) -> Vec2 {
    match direction {
        MoveDirection::Left => vec2(-5., 0.),
        MoveDirection::Right => vec2(5., 0.),
    }
}

trait Vec2Extension {
    fn to_vec3(self) -> Vec3;
}

impl Vec2Extension for Vec2 {
    fn to_vec3(self) -> Vec3 {
        Vec3 { x: self.x, y: self.y, z: 0. }
    }
}
