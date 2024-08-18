mod piece;
mod square;

use bevy::color::palettes::css::PURPLE;
use bevy::pbr::light_consts::lux::OVERCAST_DAY;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use piece::PieceColour;
use core::f32::consts::PI;
use square::PlayerTurn;


fn setup(mut commands: Commands) {
    let board_centre = Vec3::new(3.5, 0.0, 3.5);

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(3.5, 10.0, -7.5).looking_at(board_centre, Vec3::Y),
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

fn swivel_camera(
    time: Res<Time>,
    turn: ResMut<PlayerTurn>,
    mut camera_transform_query: Query<&mut Transform, With<Camera>>,
) {
    let radius = 11.0;
    let white_camera_pos = Vec3::new(3.5, 10.0, 3.5 - radius);
    let black_camera_pos = Vec3::new(3.5, 10.0, 3.5 + radius);
    let board_centre = Vec3::new(3.5, 0.0, 3.5);

    let mut camera_transform = camera_transform_query.get_single_mut().unwrap();

    let dist = match turn.0 {
        PieceColour::White => camera_transform.translation - white_camera_pos,
        PieceColour::Black => camera_transform.translation - black_camera_pos,
    };

    if dist.length_squared() > 0.01 {
        // rotate counterclockwise about the centre
        let angle_delta = -time.delta_seconds() * dist.length();
        let new_x = angle_delta.cos() * (camera_transform.translation.x - board_centre.x)
            - angle_delta.sin() * (camera_transform.translation.z - board_centre.z) + board_centre.x;
        let new_z = angle_delta.sin() * (camera_transform.translation.x - board_centre.x)
            + angle_delta.cos() * (camera_transform.translation.z - board_centre.z) + board_centre.z;
        *camera_transform = Transform::from_xyz(new_x, 10.0, new_z).looking_at(board_centre, Vec3::Y);
    }
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
        .add_systems(Update, swivel_camera)
        .run();
}
