use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use crate::piece::{Piece, PieceColour};

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

#[derive(Resource)]
pub struct PlayerTurn(pub PieceColour);
impl Default for PlayerTurn {
    fn default() -> Self {
        Self(PieceColour::White)
    }
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
    mut turn: ResMut<PlayerTurn>,
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
                            piece.clone(),
                        )
                    })
                    .collect();
            	let pieces_vec = pieces_query.iter_mut().map(|(_, piece)| piece.clone()).collect();

                // Move the selected piece to the selected square
                if let Ok((_piece_entity, mut piece)) = pieces_query.get_mut(selected_piece_entity) {
                	if piece.is_move_valid((square.x, square.y), pieces_vec) {
                		let mut captured_piece = false;
                		for (dx, dy) in piece.squares_occupied.clone() {
                			let square_x = square.x.checked_add_signed(dx).expect("x < 0");
                			let square_y = square.y.checked_add_signed(dy).expect("y < 0");
	                        // Check if a piece of the opposite color exists in this square and despawn it
	                        for (other_entity, other_piece) in &pieces_entity_vec {
	                            if other_piece.x == square_x
	                                && other_piece.y == square_y
	                                && other_piece.colour != piece.colour
	                            {
	                                // Despawn piece
	                                commands.entity(*other_entity).despawn_recursive();
	                                captured_piece = true;
	                            }
	                        }
	                    }
	                    if captured_piece {
                            piece.consume_piece(square.x, square.y);
                        }
	                    piece.transform.translation = Vec3::new(square.x as f32, 0., square.y as f32) + piece.offset;

                        // move piece
	                    piece.x = square.x;
	                    piece.y = square.y;

	                    // switch turns
	                    turn.0 = match turn.0 {
                            PieceColour::White => PieceColour::Black,
                            PieceColour::Black => PieceColour::White,
                        };

                        // deselect square and piece
                        selected_square.entity = None;
                        selected_piece.entity = None;
	                } else {
                        // Select the piece in the currently selected square
                        for (piece_entity, piece) in &pieces_entity_vec {
                            for (dx, dy) in &piece.squares_occupied {
                                if piece.x as i8 + dx == square.x as i8 && piece.y as i8 + dy == square.y as i8 && piece.colour == turn.0 {
                                    // piece_entity is now the entity in the same square
                                    selected_piece.entity = Some(*piece_entity);
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    selected_square.entity = None;
                    selected_piece.entity = None;
                }
            } else {
                // Select the piece in the currently selected square
                for (piece_entity, piece) in pieces_query.iter_mut() {
					for (dx, dy) in &piece.squares_occupied {
	                    if piece.x as i8 + dx == square.x as i8 && piece.y as i8 + dy == square.y as i8 && piece.colour == turn.0 {
	                        // piece_entity is now the entity in the same square
	                        selected_piece.entity = Some(piece_entity);
	                        break;
	                    }
	                }
                }
            }
        }
    }
}


fn highlight_assoc_squares(
	squares_query: Query<(Entity, &Square)>,
	mut squares_query2: Query<(Entity, &Square, &mut PickSelection)>,
    mut click_event: EventReader<Pointer<Click>>,
	pieces_query: Query<(Entity, &Piece)>
) {
	for event in click_event.read() {
		if let Ok((_, square)) = squares_query.get(event.target) {
			for (_, piece) in pieces_query.iter() {
				let mut piece_on_square = false;
				for (dx, dy) in &piece.squares_occupied {
					if square.x as i8 == piece.x as i8 + dx && square.y as i8 == piece.y as i8 + dy {
						piece_on_square = true;
						break;
					}
				}
				if piece_on_square {
					for (dx, dy) in &piece.squares_occupied {
						for (_, square, mut pick_selection) in squares_query2.iter_mut() {
							if square.x as i8 == piece.x as i8 + dx && square.y as i8 == piece.y as i8 + dy {
								pick_selection.is_selected = true;
							}
						}
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
            .insert_resource(PlayerTurn::default())
            .add_systems(Startup, setup_squares)
            .add_systems(Update, (select_square, highlight_assoc_squares));
    }
}
