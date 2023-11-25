use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, input)
        .run();
}

#[derive(Bundle)]
struct PlayerBundle {
    controllable: Controllable,
    position: Position,
    sprite_bundle: SpriteBundle,
}

/// Controllable entity by a player.
#[derive(Component)]
struct Controllable {}

#[derive(Component)]
struct Position { x: f32, y: f32 }

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Player
    commands.spawn(
        PlayerBundle {
            controllable: Controllable {},
            position: Position { x: 0., y: 0. },
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

fn input(
    key_input: Res<Input<KeyCode>>,
    mut pts: Query<&mut Transform, With<Controllable>>,
) {
    if key_input.pressed(KeyCode::W) {
        for mut pt in &mut pts {
            pt.translation.y += 3.;
        }
    }
    if key_input.pressed(KeyCode::S) {
        for mut pt in &mut pts {
            pt.translation.y -= 3.;
        }
    }
    if key_input.pressed(KeyCode::A) {
        for mut pt in &mut pts {
            pt.translation.x -= 3.;
        }
    }
    if key_input.pressed(KeyCode::D) {
        for mut pt in &mut pts {
            pt.translation.x += 3.;
        }
    }
}
