use std::ops::Neg;
use std::time::Duration;
use bevy::audio::PlaybackMode;

use bevy::math::vec2;
use bevy::prelude::*;
use bevy::reflect::List;
use bevy::sprite::Anchor;
use bevy::sprite::collide_aabb::{collide, Collision};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use lazy_static::lazy_static;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_systems(Startup, setup)
        .configure_sets(
            Update,
            (
                GameSystemSet::Input,
                GameSystemSet::PlayerStateModification,
                GameSystemSet::MovementVecModification,
                GameSystemSet::Movement,
                GameSystemSet::PostMovement,
                GameSystemSet::Debug,
            ).chain(),
        )

        .add_systems(Update, input_to_event.in_set(GameSystemSet::Input))

        .add_systems(
            Update,
            (
                start_jump,
                (start_dash, dash_stop_jump).chain(),
                (climb, climb_stop_jump).chain(),
            )
                .in_set(GameSystemSet::PlayerStateModification),
        )

        .add_systems(
            Update,
            (
                jump_lift,
                player_side_movements,
                dash_move,
                gravity,
            ).in_set(GameSystemSet::MovementVecModification),
        )

        .add_systems(Update, player_movement.in_set(GameSystemSet::Movement))

        .add_systems(
            Update,
            (
                // move_camera,
                reset_dash,
                ceiling_stop_jump,
            ).in_set(GameSystemSet::PostMovement)
        )

        .add_systems(Update, update_player_debug.in_set(GameSystemSet::Debug))

        .add_event::<PlayerMoveInputEvent>()
        .add_event::<PlayerJumpInputEvent>()
        .add_event::<PlayerDashInputEvent>()
        .add_event::<PlayerClimbInputEvent>()
        .add_event::<CeilingBumpEvent>()
        .run();
}

const GRAVITY: f32 = -9.;
const JUMP_DURATION: Duration = Duration::from_millis(750);
const JUMP_POWER: f32 = 25.;
const DASH_DURATION: Duration = Duration::from_millis(300);
const DASH_POWER: f32 = 10.;
lazy_static! {
    static ref DIAGONAL_DASH_POWER: f32 = (DASH_POWER.powf(2.) / 2.).sqrt();
}

/// The ending duration when gravity power is greater than a jump power.
const JUMP_ENDING_DURATION: Duration = Duration::from_millis(((GRAVITY * -1.) / JUMP_POWER * 1000.) as u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
enum GameSystemSet {
    Input,
    PlayerStateModification,
    MovementVecModification,
    Movement,
    PostMovement,
    Debug,
}

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    sprite_bundle: SpriteBundle,
}

/// Controllable entity by a player.
#[derive(Component, Debug)]
struct Player {
    on_ground: bool,
    movement_vec: Vec2,
    jumping_timer: Option<Timer>,
    dashing: Option<Dashing>,
    can_dash: bool,
    climbing: bool,
}

/// Obstacle for a player.
#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Climbable;

#[derive(Event)]
struct PlayerMoveInputEvent {
    direction: MoveDirection,
}

#[derive(Copy, Clone)]
enum MoveDirection {
    Left,
    Right,
}

#[derive(Event)]
struct PlayerJumpInputEvent;

#[derive(Event)]
struct CeilingBumpEvent;

#[derive(Event)]
struct PlayerDashInputEvent {
    direction: DashDirection,
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Debug, Clone)]
struct Dashing {
    direction: DashDirection,
    timer: Timer,
}

#[derive(Event)]
struct PlayerClimbInputEvent;


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
                climbing: false,
            },
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(-300., 0., 10.),
                    rotation: Quat::IDENTITY,
                    scale: Vec3::new(50., 70., 0.),
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
        Climbable,
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

    // Box in the air
    commands.spawn((
        Obstacle,
        Climbable,
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

    // Debug player information
    commands.spawn(
        Text2dBundle {
            text: Text::from_section("-", TextStyle::default()),
            text_anchor: Anchor::TopLeft,
            transform: Transform {
                translation: Vec3::new((-1280. / 2.) + 5., (720. / 2.) - 5., 100.),
                ..default()
            },
            ..default()
        },
    );
}

fn input_to_event(
    key_input: Res<Input<KeyCode>>,
    mut event_player_move: EventWriter<PlayerMoveInputEvent>,
    mut event_player_jump: EventWriter<PlayerJumpInputEvent>,
    mut event_player_dash: EventWriter<PlayerDashInputEvent>,
    mut event_player_climb: EventWriter<PlayerClimbInputEvent>,
) {
    if key_input.pressed(KeyCode::Left) {
        event_player_move.send(PlayerMoveInputEvent { direction: MoveDirection::Left })
    }
    if key_input.pressed(KeyCode::Right) {
        event_player_move.send(PlayerMoveInputEvent { direction: MoveDirection::Right })
    }

    if key_input.just_pressed(KeyCode::C) {
        event_player_jump.send(PlayerJumpInputEvent)
    }

    if key_input.just_pressed(KeyCode::X) {
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
            event_player_dash.send(PlayerDashInputEvent { direction });
        }
    }

    if key_input.pressed(KeyCode::Z) {
        event_player_climb.send(PlayerClimbInputEvent);
    }
}

fn player_side_movements(
    mut event_player_move: EventReader<PlayerMoveInputEvent>,
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
        // disable gravity while dashing
        if player.dashing.is_some() {
            continue;
        }

        // disable gravity while climbing
        if player.climbing {
            continue;
        }

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
    mut event_jump: EventReader<PlayerJumpInputEvent>,
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
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            }
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

fn ceiling_stop_jump(
    mut ceiling_bump_event: EventReader<CeilingBumpEvent>,
    mut players: Query<&mut Player>,
) {
    if ceiling_bump_event.read().next().is_none() {
        return;
    }

    for mut player in players.iter_mut() {
        if let Some(timer) = player.jumping_timer.as_mut() {
            if timer.remaining() > JUMP_ENDING_DURATION {
                timer.set_duration(JUMP_ENDING_DURATION);
                timer.set_elapsed(Duration::ZERO);
            }
        }
    }
}

fn start_dash(
    mut dash_event: EventReader<PlayerDashInputEvent>,
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
            settings: PlaybackSettings {
                mode: PlaybackMode::Despawn,
                ..default()
            }
        });
    }
}

fn dash_stop_jump(mut players: Query<&mut Player>) {
    for mut player in players.iter_mut() {
        if player.dashing.is_some() {
            player.jumping_timer = None;
        }
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
            player.movement_vec += map_dash_direction_to_vec2(direction);
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

fn climb(
    mut climb_event: EventReader<PlayerClimbInputEvent>,
    mut players: Query<(&mut Player, &Transform)>,
    climbables: Query<&Transform, (With<Climbable>, Without<Player>)>
) {
    if climb_event.read().next().is_none() {
        for (mut player, _) in players.iter_mut() {
            player.climbing = false;
        }
        return;
    }

    for (mut player, p_transform) in players.iter_mut() {
        let proper_climbable_opt = climbables.iter().find(|c_transform| {
            let p_left_x = p_transform.translation.x - p_transform.scale.x / 2.;
            let p_right_x = p_transform.translation.x + p_transform.scale.x / 2.;
            let c_left_x = c_transform.translation.x - c_transform.scale.x / 2.;
            let c_right_x = c_transform.translation.x + c_transform.scale.x / 2.;

            let space_between1 = c_left_x - p_right_x;
            let space_between2 = p_left_x - c_right_x;

            if space_between1 == 0. || space_between2 == 0. {
                // x coordinates are appropriate to climb
            } else {
                return false;
            }

            let p_top_y = p_transform.translation.y + p_transform.scale.y / 2.;
            let p_bottom_y = p_transform.translation.y - p_transform.scale.y / 2.;
            let c_top_y = c_transform.translation.y + c_transform.scale.y / 2.;
            let c_bottom_y = c_transform.translation.y - c_transform.scale.y / 2.;

            if (c_bottom_y..=c_top_y).contains(&p_top_y)
                || (c_bottom_y..=c_top_y).contains(&p_bottom_y)
                || (p_bottom_y..=p_top_y).contains(&c_top_y)
                || (p_bottom_y..=p_top_y).contains(&c_bottom_y) {
                return true;
            }
            false
        });

        if proper_climbable_opt.is_some() {
            player.climbing = true;
        } else {
            player.climbing = false;
        }
    }
}

fn climb_stop_jump(mut players: Query<&mut Player>) {
    for mut player in players.iter_mut() {
        if player.climbing {
            player.jumping_timer = None;
        }
    }
}

fn update_player_debug(
    mut texts: Query<&mut Text>,
    players: Query<(&Player, &Transform)>,
) {
    let (player, transform) = if let Some(pt) = players.iter().next() {
        pt
    } else {
        return;
    };

    for mut text in texts.iter_mut() {
        text.sections[0].value = format!("\
            x: {}\n\
            y: {}\n\
            on_ground: {}\n\
            can_dash: {}\n\
            climbing: {}\n\
            jumping_timer.elapsed: {:?}\n\
            jumping_timer.duration: {:?}\n\
            dashing.direction: {:?}\n\
            dashing.elapsed: {:?}\n\
            dashing.duration: {:?}\n\
            ",
            transform.translation.x,
            transform.translation.y,
            player.on_ground,
            player.can_dash,
            player.climbing,
            player.jumping_timer.clone().map(|it| { it.elapsed() }),
            player.jumping_timer.clone().map(|it| { it.duration() }),
            player.dashing.clone().map(|it| { it.direction }),
            player.dashing.clone().map(|it| { it.timer.elapsed() }),
            player.dashing.clone().map(|it| { it.timer.duration() }),
        );
    }
}

fn move_camera(
    mut camera_transform: Query<&mut Transform, With<Camera>>,
    player_transform: Query<&Transform, (With<Player>, Without<Camera>)>,
) {
    let mut camera_transform = if let Some(ct) = camera_transform.iter_mut().next() {
        ct
    } else {
        return;
    };

    let player_transform = if let Some(pt) = player_transform.iter().next() {
        pt
    } else {
        return;
    };

    camera_transform.translation.x = player_transform.translation.x;
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
        DashDirection::Up => vec2(0., DASH_POWER),
        DashDirection::UpRight => vec2(*DIAGONAL_DASH_POWER, *DIAGONAL_DASH_POWER),
        DashDirection::Right => vec2(DASH_POWER, 0.),
        DashDirection::DownRight => vec2(*DIAGONAL_DASH_POWER, DIAGONAL_DASH_POWER.neg()),
        DashDirection::Down => vec2(0., DASH_POWER.neg()),
        DashDirection::DownLeft => vec2(DIAGONAL_DASH_POWER.neg(), DIAGONAL_DASH_POWER.neg()),
        DashDirection::Left => vec2(DASH_POWER.neg(), 0.),
        DashDirection::UpLeft => vec2(DIAGONAL_DASH_POWER.neg(), *DIAGONAL_DASH_POWER),
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
