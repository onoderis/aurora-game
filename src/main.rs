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
        .add_systems(Update, ceiling_jump_stop)
        .add_systems(Update, start_dash)
        .add_systems(Update, dash_move)
        .add_systems(Update, player_movement)
        .add_systems(Update, reset_dash)
        .add_event::<PlayerMoveEvent>()
        .add_event::<PlayerJumpEvent>()
        .add_event::<PlayerDashEvent>()
        .add_event::<CeilingBumpEvent>()
        .run();
}

const GRAVITY: f32 = -9.;
const JUMP_DURATION: Duration = Duration::from_millis(750);
const JUMP_POWER: f32 = 35.;
const DASH_DURATION: Duration = Duration::from_millis(300);

/// The ending duration when gravity power is greater than a jump power.
const JUMP_ENDING_DURATION: Duration = Duration::from_millis(((GRAVITY * -1.) / JUMP_POWER * 1000.) as u64);

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    sprite_bundle: SpriteBundle,
}

/// Controllable entity by a player.
#[derive(Component)]
struct Player {
    on_ground: bool,
    movement_vec: Vec2,
    jumping_timer: Option<Timer>,
    dashing: Option<Dashing>,
    can_dash: bool,
}

/// Obstacle for a player.
#[derive(Component)]
struct Obstacle;

#[derive(Event)]
struct PlayerMoveEvent {
    direction: MoveDirection,
}

#[derive(Copy, Clone)]
enum MoveDirection {
    Left,
    Right,
}

#[derive(Event)]
struct PlayerJumpEvent;

#[derive(Event)]
struct CeilingBumpEvent;

#[derive(Event)]
struct PlayerDashEvent {
    direction: DashDirection,
}

#[derive(Copy, Clone)]
enum DashDirection {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

struct Dashing {
    direction: DashDirection,
    timer: Timer,
}

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Player
    commands.spawn(
        PlayerBundle {
            player: Player {
                on_ground: false,
                movement_vec: Vec2 { x: 0., y: 0. },
                jumping_timer: None,
                dashing: None,
                can_dash: false,
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
        Obstacle,
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
        Obstacle,
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

    // Box in air
    commands.spawn((
        Obstacle,
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(-450., 0., 0.),
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
    mut event_player_dash: EventWriter<PlayerDashEvent>,
) {
    if key_input.just_pressed(KeyCode::X) {
        event_player_jump.send(PlayerJumpEvent)
    }
    if key_input.pressed(KeyCode::Left) {
        event_player_move.send(PlayerMoveEvent { direction: MoveDirection::Left })
    }
    if key_input.pressed(KeyCode::Right) {
        event_player_move.send(PlayerMoveEvent { direction: MoveDirection::Right })
    }

    if key_input.just_pressed(KeyCode::Z) {
        let direction = match (key_input.pressed(KeyCode::Up),
               key_input.pressed(KeyCode::Down),
               key_input.pressed(KeyCode::Left),
               key_input.pressed(KeyCode::Right),
        ) {
            (true, false, false, false) => Some(DashDirection::Up),
            (true, false, false, true) => Some(DashDirection::UpRight),
            (false, false, false, true) => Some(DashDirection::Right),
            (false, true, false, true) => Some(DashDirection::DownRight),
            (false, true, false, false) => Some(DashDirection::Down),
            (false, true, true, false) => Some(DashDirection::DownLeft),
            (false, false, true, false) => Some(DashDirection::Left),
            (true, false, true, false) => Some(DashDirection::UpLeft),
            _ => None
        };
        if let Some(direction) = direction {
            event_player_dash.send(PlayerDashEvent { direction });
        }
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
        if player.dashing.is_some() {
            // ignore side movements while dashing
            continue;
        }
        player.movement_vec += map_move_direction_to_vec2(event.direction);
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
    mut ceiling_event: EventWriter<CeilingBumpEvent>,
) {
    for (mut player, mut p_transform) in players.iter_mut() {
        let mut has_bottom_collision = false;
        let mut ceiling_bump = false;

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
                        ceiling_bump = true
                    }
                    Collision::Bottom => {
                        player.movement_vec.y = o_transform.translation.y
                            - p_transform.translation.y
                            + p_transform.scale.y / 2.
                            + o_transform.scale.y / 2.;
                        has_bottom_collision = true;
                    }
                    Collision::Inside => {}
                }
            }
        }

        p_transform.translation += player.movement_vec.to_vec3();
        player.movement_vec = Vec2::ZERO;
        player.on_ground = has_bottom_collision;
        if ceiling_bump {
            ceiling_event.send(CeilingBumpEvent);
        }
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
                player.movement_vec.y += timer.remaining().as_secs_f32() * JUMP_POWER;
            } else {
                player.jumping_timer = None;
            }
        }
    }
}

fn ceiling_jump_stop(
    mut ceiling_event: EventReader<CeilingBumpEvent>,
    mut players: Query<&mut Player>,
) {
    if ceiling_event.read().next().is_none() {
        return;
    }

    for mut player in players.iter_mut() {
        if let Some(timer) = player.jumping_timer.as_mut() {
            if timer.remaining() > JUMP_ENDING_DURATION {
                timer.set_duration(JUMP_ENDING_DURATION);
            }
        }
    }
}

fn start_dash(
    mut dash_event: EventReader<PlayerDashEvent>,
    mut players: Query<&mut Player>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let direction = if let Some(d) = dash_event.read().next() {
        d.direction
    } else {
        return;
    };

    for mut player in players.iter_mut() {
        if !player.can_dash {
            continue;
        }

        player.dashing = Some(Dashing {
            direction,
            timer: Timer::new(DASH_DURATION, TimerMode::Once)
        });
        player.can_dash = false;
        commands.spawn(AudioBundle {
            source: asset_server.load("sounds/dash.ogg"),
            ..default()
        });

        // cancel jumping
        player.jumping_timer = None;
    }
}

fn dash_move(
    mut players: Query<&mut Player>,
    time: Res<Time>,
) {
    for mut player in players.iter_mut() {
        let mut active_dash_direction = None;
        if let Some(dashing) = player.dashing.as_mut() {
            dashing.timer.tick(time.delta());
            if dashing.timer.remaining() > Duration::ZERO {
                active_dash_direction = Some(dashing.direction);
            } else {
                player.dashing = None;
            }
        }
        if let Some(direction) = active_dash_direction {
            player.movement_vec += map_dash_direction_to_vec2(direction) + vec2(0., -1. * GRAVITY);
        }
    }
}

fn reset_dash(mut players: Query<&mut Player>) {
    for mut player in players.iter_mut() {
        if player.on_ground {
            player.can_dash = true;
        }
    }
}

// util functions

fn map_move_direction_to_vec2(direction: MoveDirection) -> Vec2 {
    match direction {
        MoveDirection::Left => vec2(-5., 0.),
        MoveDirection::Right => vec2(5., 0.),
    }
}

fn map_dash_direction_to_vec2(direction: DashDirection) -> Vec2 {
    match direction {
        DashDirection::Up => vec2(0., 10.),
        DashDirection::UpRight => vec2(10., 10.),
        DashDirection::Right => vec2(10., 0.),
        DashDirection::DownRight => vec2(10., -10.),
        DashDirection::Down => vec2(0., -10.),
        DashDirection::DownLeft => vec2(-10., -10.),
        DashDirection::Left => vec2(-10., 0.),
        DashDirection::UpLeft => vec2(-10., 10.),
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
