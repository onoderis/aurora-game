use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        // .add_system(input)
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
                    color: Color::BLUE,
                    ..default()
                },
                ..default()
            },
        }
    );
}
