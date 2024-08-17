use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use crate::piece::Piece;

#[derive(Default, Resource)]
pub struct SelectedSquare {
    pub entity: Option<Entity>,
}

#[derive(Default, Resource)]
pub struct SelectedPiece {
    pub entity: Option<Entity>,
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
    mut commands: Commands,
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
            	let pieces_entity_vec: Vec<(Entity, Piece)> = pieces_query
                    .iter_mut()
                    .map(|(entity, piece)| {
                        (
                            entity,
                            *piece,
                        )
                    })
                    .collect();
            	let pieces_vec = pieces_query.iter_mut().map(|(_, piece)| *piece).collect();

                // Move the selected piece to the selected square
                if let Ok((_piece_entity, mut piece)) = pieces_query.get_mut(selected_piece_entity)
                {
                	if piece.is_move_valid((square.x, square.y), pieces_vec) {
                        // Check if a piece of the opposite color exists in this square and despawn it
                        for (other_entity, other_piece) in pieces_entity_vec {
                            if other_piece.x == square.x
                                && other_piece.y == square.y
                                && other_piece.colour != piece.colour
                            {
                                // Despawn piece
                                commands.entity(other_entity).despawn_recursive();
                            }
                        }

                        // move piece
	                    piece.x = square.x;
	                    piece.y = square.y;
	                }
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

fn setup_squares(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let board_square_mesh = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));
    let board_black = materials.add(Color::srgb(0.1, 0.1, 0.0));
    let board_white = materials.add(Color::srgb(0.9, 0.9, 1.0));

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
}

pub struct SquaresPlugin;
impl Plugin for SquaresPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup_squares)
            .add_systems(Update, select_square);
    }
}
