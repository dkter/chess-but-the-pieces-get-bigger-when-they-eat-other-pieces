mod piece;
mod square;

use bevy::color::palettes::css::PURPLE;
use bevy::pbr::light_consts::lux::OVERCAST_DAY;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use core::f32::consts::PI;


fn setup(mut commands: Commands) {
    let board_centre = Vec3::new(4., 0., 4.);

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(4.0, 10.0, -7.0).looking_at(board_centre, Vec3::Y),
        ..Default::default()
    });
    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(board_centre + Vec3::new(0., 4., -8.)),
        point_light: PointLight {
            intensity: 1_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        ..Default::default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: OVERCAST_DAY,
            //shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 12.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
    commands.insert_resource(AmbientLight {
        color: PURPLE.into(),
        brightness: 20.,
    });
}

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .insert_resource(square::SelectedSquare { entity: None })
        .insert_resource(square::SelectedPiece { entity: None })
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "chess?".into(),
                    resolution: (800., 800.).into(),
                    ..default()
                }),
                ..default()
            }),
            DefaultPickingPlugins,
            piece::PiecesPlugin,
            square::SquaresPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}
