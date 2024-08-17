use bevy::prelude::*;
use bevy_mod_picking::picking_core::Pickable;
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

#[derive(Clone, Component)]
pub struct Piece {
    pub colour: PieceColour,
    pub piece_type: PieceType,
    pub x: u8,
    pub y: u8,
    pub transform: Transform,
    pub offset: Vec3,
    pub squares_occupied: Vec<(i8, i8)>,
}

fn piece_colour_on_square(pos: (u8, u8), pieces: &Vec<Piece>) -> Option<PieceColour> {
    for piece in pieces {
        if piece.x == pos.0 && piece.y == pos.1 {
            return Some(piece.colour);
        }
    }
    None
}

fn is_path_empty(begin: (u8, u8), end: (u8, u8), pieces: &Vec<Piece>) -> bool {
    // Same column
    if begin.0 == end.0 {
        for piece in pieces {
            if piece.x == begin.0
                && ((piece.y > begin.1 && piece.y < end.1)
                    || (piece.y > end.1 && piece.y < begin.1))
            {
                return false;
            }
        }
    }
    // Same row
    if begin.1 == end.1 {
        for piece in pieces {
            if piece.y == begin.1
                && ((piece.x > begin.0 && piece.x < end.0)
                    || (piece.x > end.0 && piece.x < begin.0))
            {
                return false;
            }
        }
    }

    // Diagonals
    let x_diff = (begin.0 as i8 - end.0 as i8).abs();
    let y_diff = (begin.1 as i8 - end.1 as i8).abs();
    if x_diff == y_diff {
        for i in 1..x_diff {
            let pos = if begin.0 < end.0 && begin.1 < end.1 {
                // left bottom - right top
                (begin.0 + i as u8, begin.1 + i as u8)
            } else if begin.0 < end.0 && begin.1 > end.1 {
                // left top - right bottom
                (begin.0 + i as u8, begin.1 - i as u8)
            } else if begin.0 > end.0 && begin.1 < end.1 {
                // right bottom - left top
                (begin.0 - i as u8, begin.1 + i as u8)
            } else {
                // begin.0 > end.0 && begin.1 > end.1
                // right top - left bottom
                (begin.0 - i as u8, begin.1 - i as u8)
            };

            if piece_colour_on_square(pos, pieces).is_some() {
                return false;
            }
        }
    }

    true
}

impl Piece {
    /// Returns the possible_positions that are available
    pub fn is_move_valid(&self, new_position: (u8, u8), pieces: Vec<Piece>) -> bool {
        // If there's a piece of the same color in the same square, it can't move
        if piece_colour_on_square(new_position, &pieces) == Some(self.colour) {
            return false;
        }

        match self.piece_type {
            PieceType::King => {
                // Horizontal
                ((self.x as i8 - new_position.0 as i8).abs() == 1
                    && (self.y == new_position.1))
                // Vertical
                || ((self.y as i8 - new_position.1 as i8).abs() == 1
                    && (self.x == new_position.0))
                // Diagonal
                || ((self.x as i8 - new_position.0 as i8).abs() == 1
                    && (self.y as i8 - new_position.1 as i8).abs() == 1)
            }
            PieceType::Queen => {
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && ((self.x as i8 - new_position.0 as i8).abs()
                        == (self.y as i8 - new_position.1 as i8).abs()
                        || ((self.x == new_position.0 && self.y != new_position.1)
                            || (self.y == new_position.1 && self.x != new_position.0)))
            }
            PieceType::Bishop => {
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && (self.x as i8 - new_position.0 as i8).abs()
                        == (self.y as i8 - new_position.1 as i8).abs()
            }
            PieceType::Knight => {
                ((self.x as i8 - new_position.0 as i8).abs() == 2
                    && (self.y as i8 - new_position.1 as i8).abs() == 1)
                    || ((self.x as i8 - new_position.0 as i8).abs() == 1
                        && (self.y as i8 - new_position.1 as i8).abs() == 2)
            }
            PieceType::Rook => {
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && ((self.x == new_position.0 && self.y != new_position.1)
                        || (self.y == new_position.1 && self.x != new_position.0))
            }
            PieceType::Pawn => {
                if self.colour == PieceColour::White {
                    // Normal move
                    if new_position.1 as i8 - self.y as i8 == 1 && (self.x == new_position.0) {
                        if piece_colour_on_square(new_position, &pieces).is_none() {
                            return true;
                        }
                    }

                    // Move 2 squares
                    if self.y == 1
                        && new_position.1 as i8 - self.y as i8 == 2
                        && (self.x == new_position.0)
                        && is_path_empty((self.x, self.y), new_position, &pieces)
                    {
                        if piece_colour_on_square(new_position, &pieces).is_none() {
                            return true;
                        }
                    }

                    // Take piece
                    if new_position.1 as i8 - self.y as i8 == 1
                        && (self.x as i8 - new_position.0 as i8).abs() == 1
                    {
                        if piece_colour_on_square(new_position, &pieces) == Some(PieceColour::Black) {
                            return true;
                        }
                    }
                } else {
                    // Normal move
                    if new_position.1 as i8 - self.y as i8 == -1 && (self.x == new_position.0) {
                        if piece_colour_on_square(new_position, &pieces).is_none() {
                            return true;
                        }
                    }

                    // Move 2 squares
                    if self.y == 6
                        && new_position.1 as i8 - self.y as i8 == -2
                        && (self.x == new_position.0)
                        && is_path_empty((self.x, self.y), new_position, &pieces)
                    {
                        if piece_colour_on_square(new_position, &pieces).is_none() {
                            return true;
                        }
                    }

                    // Take piece
                    if new_position.1 as i8 - self.y as i8 == -1
                        && (self.x as i8 - new_position.0 as i8).abs() == 1
                    {
                        if piece_colour_on_square(new_position, &pieces) == Some(PieceColour::White) {
                            return true;
                        }
                    }
                }

                false
            }
        }
    }

    pub fn update_transform(&mut self) {
        // set offset to the centre of squares_occupied
        self.offset = Vec3::ZERO;
        for (x, y) in &self.squares_occupied {
            self.offset.x += *x as f32;
            self.offset.z += *y as f32;
        }
        self.offset.x /= self.squares_occupied.len() as f32;
        self.offset.z /= self.squares_occupied.len() as f32;

        // if squares_occupied is symmetric around offset, rotation = 0
        // else rotation = -pi/4
        let mut is_symmetric = true;
        for (x, y) in &self.squares_occupied {
            is_symmetric = false;
            for (x2, y2) in &self.squares_occupied {
                if (*x as f32 - self.offset.x == self.offset.x - *x2 as f32 && *y as f32 == *y2 as f32)
                    || (*y as f32 - self.offset.z == self.offset.z - *y2 as f32 && *x as f32 == *x2 as f32) {
                        // println!("{}, {}, {}, {} {} {}", *x as f32, self.offset.x, *x2 as f32,
                        //                 *y as f32, self.offset.z, *y2 as f32);
                    is_symmetric = true;
                    break;
                }
            }
            if !is_symmetric {
                break;
            }
        }
        println!("{:?}", is_symmetric);
        if is_symmetric {
            self.transform.rotation = Quat::IDENTITY;

            // transform.scale.x = 2 * length from centre in direction of rotation
            // transform.scale.z = 2 * length from centre perp. to direction of rotation
            let mut max_x = self.offset.x;
            let mut max_y = self.offset.z;
            for (x, y) in &self.squares_occupied {
                if (*y as f32) == self.offset.z && (*x as f32) > max_x {
                    max_x = *x as f32;
                }
                if (*x as f32) == self.offset.x && (*y as f32) > max_y {
                    max_y = *y as f32;
                }
            }

            self.transform.scale.x = max_x - self.offset.x + 1.0;
            self.transform.scale.z = max_y - self.offset.z + 1.0;
        } else {
            self.transform.rotation = Quat::from_rotation_y(-PI/4.0);

            // transform.scale.x = 2 * length from centre in direction of rotation
            // transform.scale.z = 2 * length from centre perp. to direction of rotation
            let mut max_x = self.offset.x;
            let mut max_y = self.offset.z;
            for (x, y) in &self.squares_occupied {
                if (*y as f32) - self.offset.z == (*x as f32) - self.offset.x && (*x as f32) > max_x {
                    max_x = *x as f32;
                }
                if (*y as f32) - self.offset.z == self.offset.x - (*x as f32) && (*y as f32) > max_y {
                    max_y = *y as f32;
                }
            }

            self.transform.scale.x = (max_x - self.offset.x + 1.0) * 1.414;
            self.transform.scale.z = (max_y - self.offset.z + 1.0) * 1.414;
        }
    }

    pub fn consume_piece(&mut self, x: u8, y: u8) {
        if x as i8 - self.x as i8 == y as i8 - self.y as i8 {
            // diagonal capture
            if x < self.x {
                self.squares_occupied.push((1, 1));
            } else {
                self.squares_occupied.push((-1, -1));
            }
        } else if x as i8 - self.x as i8 == self.y as i8 - y as i8 {
            // diagonal capture (the other way)
            if x < self.x {
                self.squares_occupied.push((1, -1));
            } else {
                self.squares_occupied.push((-1, 1));
            }
        }
        self.update_transform();
        self.transform.translation += self.offset;
    }
}

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        let direction = piece.transform.translation - transform.translation;
        let scale_diff = piece.transform.scale - transform.scale;
        if direction.length() > 0.01 {
            transform.translation += direction.normalize() * time.delta_seconds() * 2.0;
        }
        if scale_diff.length() > 0.01 {
            transform.scale += scale_diff.normalize() * time.delta_seconds() * 2.0;
        }
    }
}

fn transform_pieces(mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        //transform.scale = piece.transform.scale;
        transform.rotation = piece.transform.rotation;
        //transform.translation = piece.transform.translation;
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
        Piece {
            colour, piece_type, x, y,
            transform: Transform::from_translation(Vec3::new(x as f32, 0.0, y as f32)),
            offset: Vec3::ZERO,
            squares_occupied: vec![(0, 0)],
        },
        Pickable::IGNORE,
    )).with_children(|parent| {
        parent.spawn((
            PbrBundle {
                mesh,
                material,
                transform: piece_transform,
                ..Default::default()
            },
            Pickable::IGNORE,
        ));
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

pub struct PiecesPlugin;
impl Plugin for PiecesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, create_pieces)
            .add_systems(Update, (move_pieces, transform_pieces));
    }
}
