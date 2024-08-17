use bevy::color::palettes::css::PURPLE;
use bevy::pbr::light_consts::lux::OVERCAST_DAY;
use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use core::f32::consts::PI;

#[derive(Clone, Copy, PartialEq)]
pub enum PieceColour {
    White,
    Black,
}

#[derive(Clone, Copy, PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

#[derive(Clone, Copy, Component)]
pub struct Piece {
    pub colour: PieceColour,
    pub piece_type: PieceType,
    pub x: u8,
    pub y: u8,
}

#[derive(Default, Resource)]
struct SelectedSquare {
    entity: Option<Entity>,
}
#[derive(Default, Resource)]
struct SelectedPiece {
    entity: Option<Entity>,
}

#[derive(Component)]
pub struct Square {
    pub x: u8,
    pub y: u8,
}

impl Square {
    fn is_white(&self) -> bool {
        (self.x + self.y + 1) % 2 == 0
    }
}

fn select_square(
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut click_event: EventReader<Pointer<Click>>,
    squares_query: Query<(Entity, &Square)>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
) {

    for event in click_event.read() {
        if let Ok((square_entity, square)) = squares_query.get(event.target) {
            selected_square.entity = Some(square_entity);

            if let Some(selected_piece_entity) = selected_piece.entity {
                // Move the selected piece to the selected square
                if let Ok((_piece_entity, mut piece)) = pieces_query.get_mut(selected_piece_entity)
                {
                    piece.x = square.x;
                    piece.y = square.y;
                }
                selected_square.entity = None;
                selected_piece.entity = None;
            } else {
                // Select the piece in the currently selected square
                for (piece_entity, piece) in pieces_query.iter_mut() {
                    if piece.x == square.x && piece.y == square.y {
                        // piece_entity is now the entity in the same square
                        selected_piece.entity = Some(piece_entity);
                        break;
                    }
                }
            }
        }
    }
}

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        let direction = Vec3::new(piece.x as f32, 0., piece.y as f32) - transform.translation;
        if direction.length() > 0.1 {
            transform.translation += direction.normalize() * time.delta_seconds() * 2.0;
        }
    }
}

fn spawn_piece(
    commands: &mut Commands,
    material: Handle<StandardMaterial>,
    mesh: Handle<Mesh>,
    piece_type: PieceType,
    colour: PieceColour,
    x: u8,
    y: u8,
) {
    let piece_transform = Transform::from_scale(Vec3::new(0.3, 0.3, 0.3));

    commands.spawn((
        PbrBundle {
            transform: Transform::from_translation(Vec3::new(x as f32, 0.0, y as f32)),
            ..Default::default()
        },
        Piece { colour, piece_type, x, y },
    )).with_children(|parent| {
        parent.spawn(PbrBundle {
            mesh,
            material,
            transform: piece_transform,
            ..Default::default()
        });
    });
}

fn create_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let king_handle: Handle<Mesh> = asset_server.load("Chess.glb#Mesh2/Primitive1");
    let queen_handle: Handle<Mesh> = asset_server.load("Chess.glb#Mesh3/Primitive1");
    let pawn_handle: Handle<Mesh> = asset_server.load("Chess.glb#Mesh4/Primitive0");
    let knight_handle: Handle<Mesh> = asset_server.load("Chess.glb#Mesh5/Primitive0");
    let bishop_handle: Handle<Mesh> = asset_server.load("Chess.glb#Mesh6/Primitive0");
    let rook_handle: Handle<Mesh> = asset_server.load("Chess.glb#Mesh7/Primitive0");
    let chrome_handle: Handle<StandardMaterial> = asset_server.load("Chess.glb#Material6");
    let brass_handle: Handle<StandardMaterial> = asset_server.load("Chess.glb#Material9");

    // white king
    spawn_piece(&mut commands, chrome_handle.clone(), king_handle.clone(), PieceType::King, PieceColour::White, 3, 0);

    // black king
    spawn_piece(&mut commands, brass_handle.clone(), king_handle.clone(), PieceType::King, PieceColour::Black, 3, 7);

    // white queen
    spawn_piece(&mut commands, chrome_handle.clone(), queen_handle.clone(), PieceType::Queen, PieceColour::White, 4, 0);

    // black queen
    spawn_piece(&mut commands, brass_handle.clone(), queen_handle.clone(), PieceType::Queen, PieceColour::Black, 4, 7);

    // white rooks
    spawn_piece(&mut commands, chrome_handle.clone(), rook_handle.clone(), PieceType::Rook, PieceColour::White, 0, 0);
    spawn_piece(&mut commands, chrome_handle.clone(), rook_handle.clone(), PieceType::Rook, PieceColour::White, 7, 0);

    // black rooks
    spawn_piece(&mut commands, brass_handle.clone(), rook_handle.clone(), PieceType::Rook, PieceColour::Black, 0, 7);
    spawn_piece(&mut commands, brass_handle.clone(), rook_handle.clone(), PieceType::Rook, PieceColour::Black, 7, 7);

    // white knights
    spawn_piece(&mut commands, chrome_handle.clone(), knight_handle.clone(), PieceType::Knight, PieceColour::White, 1, 0);
    spawn_piece(&mut commands, chrome_handle.clone(), knight_handle.clone(), PieceType::Knight, PieceColour::White, 6, 0);

    // black knights
    spawn_piece(&mut commands, brass_handle.clone(), knight_handle.clone(), PieceType::Knight, PieceColour::Black, 1, 7);
    spawn_piece(&mut commands, brass_handle.clone(), knight_handle.clone(), PieceType::Knight, PieceColour::Black, 6, 7);

    // white bishops
    spawn_piece(&mut commands, chrome_handle.clone(), bishop_handle.clone(), PieceType::Bishop, PieceColour::White, 2, 0);
    spawn_piece(&mut commands, chrome_handle.clone(), bishop_handle.clone(), PieceType::Bishop, PieceColour::White, 5, 0);

    // black bishops
    spawn_piece(&mut commands, brass_handle.clone(), bishop_handle.clone(), PieceType::Bishop, PieceColour::Black, 2, 7);
    spawn_piece(&mut commands, brass_handle.clone(), bishop_handle.clone(), PieceType::Bishop, PieceColour::Black, 5, 7);

    // white pawns
    spawn_piece(&mut commands, chrome_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::White, 0, 1);
    spawn_piece(&mut commands, chrome_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::White, 1, 1);
    spawn_piece(&mut commands, chrome_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::White, 2, 1);
    spawn_piece(&mut commands, chrome_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::White, 3, 1);
    spawn_piece(&mut commands, chrome_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::White, 4, 1);
    spawn_piece(&mut commands, chrome_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::White, 5, 1);
    spawn_piece(&mut commands, chrome_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::White, 6, 1);
    spawn_piece(&mut commands, chrome_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::White, 7, 1);

    // black pawns
    spawn_piece(&mut commands, brass_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::Black, 0, 6);
    spawn_piece(&mut commands, brass_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::Black, 1, 6);
    spawn_piece(&mut commands, brass_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::Black, 2, 6);
    spawn_piece(&mut commands, brass_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::Black, 3, 6);
    spawn_piece(&mut commands, brass_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::Black, 4, 6);
    spawn_piece(&mut commands, brass_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::Black, 5, 6);
    spawn_piece(&mut commands, brass_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::Black, 6, 6);
    spawn_piece(&mut commands, brass_handle.clone(), pawn_handle.clone(), PieceType::Pawn, PieceColour::Black, 7, 6);

}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let board_square_mesh = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));
    let board_black = materials.add(Color::srgb(0.1, 0.1, 0.0));
    let board_white = materials.add(Color::srgb(0.9, 0.9, 1.0));
    let board_centre = Vec3::new(4., 0., 4.);

    for i in 0..8 {
        for j in 0..8 {
            commands.spawn((
                PbrBundle {
                    mesh: board_square_mesh.clone(),
                    material: if (i + j) % 2 == 0 {
                        board_white.clone()
                    } else {
                        board_black.clone()
                    },
                    transform: Transform::from_translation(Vec3::new(i as f32, 0., j as f32)),
                    ..Default::default()
                },
                PickableBundle::default(),
                Square { x: i, y: j },
            ));
        }
    }
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
        .insert_resource(SelectedSquare { entity: None })
        .insert_resource(SelectedPiece { entity: None })
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "chess?".into(),
                    resolution: (800., 800.).into(),
                    ..default()
                }),
                ..default()
            }),
            DefaultPickingPlugins
        ))
        .add_systems(Startup, (setup, create_pieces))
        .add_systems(Update, (move_pieces, select_square))
        .run();
}
