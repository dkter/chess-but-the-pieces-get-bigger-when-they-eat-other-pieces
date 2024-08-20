mod piece;
mod square;

use bevy::color::palettes::css::PURPLE;
use bevy::pbr::light_consts::lux::OVERCAST_DAY;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_mod_picking::prelude::*;
use piece::{is_colour_in_checkmate, Piece, PieceColour};
use core::f32::consts::PI;
use square::{CheckmateEvent, PlayerTurn};


#[derive(Component)]
struct SwivelDelay {
    time: Stopwatch,
}


#[derive(Component)]
struct GameStatusText;


fn setup(mut commands: Commands) {
    let board_centre = Vec3::new(3.5, 0.0, 3.5);

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(3.5, 10.0, -7.5).looking_at(board_centre, Vec3::Y),
            ..Default::default()
        },
        SwivelDelay { time: Stopwatch::new() },
    ));
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
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(board_centre + Vec3::new(0., 4., 8.)),
        point_light: PointLight {
            intensity: 1_000_000.0,
            shadows_enabled: true,
            ..default()
        },
        ..Default::default()
    });
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            //shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 12.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 2.),
            ..default()
        },
        ..default()
    });
    commands.insert_resource(AmbientLight {
        color: PURPLE.into(),
        brightness: 20.,
    });
    // UI
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    }).with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "",
                TextStyle::default(),
            ),
            GameStatusText,
        ));
    });
}

fn swivel_camera(
    time: Res<Time>,
    turn: ResMut<PlayerTurn>,
    mut camera_transform_query: Query<(&mut Transform, &mut SwivelDelay), With<Camera>>,
) {
    let radius = 11.0;
    let white_camera_pos = Vec3::new(3.5, 10.0, 3.5 - radius);
    let black_camera_pos = Vec3::new(3.5, 10.0, 3.5 + radius);
    let board_centre = Vec3::new(3.5, 0.0, 3.5);

    let (mut camera_transform, mut swivel_delay) = camera_transform_query.get_single_mut().unwrap();

    let dist = match turn.0 {
        PieceColour::White => camera_transform.translation - white_camera_pos,
        PieceColour::Black => camera_transform.translation - black_camera_pos,
    };

    if dist.length_squared() > 0.01 {
        if swivel_delay.time.elapsed_secs() > 1.0 {
            // rotate counterclockwise about the centre
            let angle_delta = -time.delta_seconds() * dist.length();
            let new_x = angle_delta.cos() * (camera_transform.translation.x - board_centre.x)
                - angle_delta.sin() * (camera_transform.translation.z - board_centre.z) + board_centre.x;
            let new_z = angle_delta.sin() * (camera_transform.translation.x - board_centre.x)
                + angle_delta.cos() * (camera_transform.translation.z - board_centre.z) + board_centre.z;
            *camera_transform = Transform::from_xyz(new_x, 10.0, new_z).looking_at(board_centre, Vec3::Y);
        } else {
            swivel_delay.time.tick(time.delta());
        }
    } else {
        swivel_delay.time.reset();
    }
}

fn update_game_status(
    mut game_status_text: Query<&mut Text, With<GameStatusText>>,
    mut checkmate_event: EventReader<CheckmateEvent>,
) {
    for ev in checkmate_event.read() {
        let mut text = game_status_text.get_single_mut().unwrap();

        match ev.0 {
            PieceColour::White => { text.sections[0].value = "Black wins!".to_string(); },
            PieceColour::Black => { text.sections[0].value = "White wins!".to_string(); },
        }
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
        .add_systems(Update, (swivel_camera, update_game_status))
        .run();
}
