use bevy::prelude::*;
use bevy_mod_picking::picking_core::Pickable;
use core::f32::consts::PI;
use std::collections::HashSet;

use crate::square;

#[derive(Clone, Copy, PartialEq)]
pub enum PieceColour {
    White,
    Black,
}

impl PieceColour {
    pub fn opposite(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
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

#[derive(Clone, Component, PartialEq)]
pub struct Piece {
    pub colour: PieceColour,
    pub piece_type: PieceType,
    pub x: u8,
    pub y: u8,
    pub transform: Transform,
    pub offset: Vec3,
    pub squares_occupied: HashSet<(i8, i8)>,
}

fn piece_colour_on_square(pos: (u8, u8), pieces: &Vec<Piece>) -> Option<PieceColour> {
    for piece in pieces {
        for (dx, dy) in &piece.squares_occupied {
            let x = piece.x.checked_add_signed(*dx).expect("Board x position of piece was <0");
            let y = piece.y.checked_add_signed(*dy).expect("Board y position of piece was <0");
            if x == pos.0 && y == pos.1 {
                return Some(piece.colour);
            }
        }
    }
    None
}

fn piece_on_square(pos: (u8, u8), pieces: &Vec<Piece>) -> Option<&Piece> {
    for piece in pieces {
        for (dx, dy) in &piece.squares_occupied {
            let x = piece.x.checked_add_signed(*dx).expect("Board x position of piece was <0");
            let y = piece.y.checked_add_signed(*dy).expect("Board y position of piece was <0");
            if x == pos.0 && y == pos.1 {
                return Some(&piece);
            }
        }
    }
    None
}

fn is_path_empty(begin: (u8, u8), end: (u8, u8), pieces: &Vec<Piece>) -> bool {
    // Same column
    if begin.0 == end.0 {
        for piece in pieces {
            for (dx, dy) in &piece.squares_occupied {
                let x = piece.x.checked_add_signed(*dx).expect("Board x position of piece was <0");
                let y = piece.y.checked_add_signed(*dy).expect("Board y position of piece was <0");
                if x == begin.0
                    && ((y > begin.1 && y < end.1)
                        || (y > end.1 && y < begin.1))
                {
                    return false;
                }
            }
        }
    }
    // Same row
    if begin.1 == end.1 {
        for piece in pieces {
            for (dx, dy) in &piece.squares_occupied {
                let x = piece.x.checked_add_signed(*dx).expect("Board x position of piece was <0");
                let y = piece.y.checked_add_signed(*dy).expect("Board y position of piece was <0");
                if y == begin.1
                    && ((x > begin.0 && x < end.0)
                        || (x > end.0 && x < begin.0))
                {
                    return false;
                }
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

pub fn is_square_defended(pos: (u8, u8), colour: PieceColour, pieces: &Vec<Piece>) -> bool {
    for piece in pieces {
        if piece.colour == colour {
            if piece.occupies_square(pos) {
                // a piece cannot defend itself
                continue;
            }
            let pieces_without_self = pieces.iter()
                .filter_map(|p| if p != piece { Some(p.clone()) } else { None })
                .collect();
            for (move_dx, move_dy) in piece.valid_captures(&pieces_without_self) {
                for &(piece_dx, piece_dy) in &piece.squares_occupied {
                    let move_x = piece.x.checked_add_signed(piece_dx + move_dx).unwrap();
                    let move_y = piece.y.checked_add_signed(piece_dy + move_dy).unwrap();
                    if (move_x, move_y) == pos {
                        return true;
                    }
                }
            }
        }
    }
    false
}

pub fn is_colour_in_check(colour: PieceColour, pieces: &Vec<Piece>) -> bool {
    for piece in pieces {
        if piece.piece_type == PieceType::King && piece.colour == colour {
            for &(dx, dy) in &piece.squares_occupied {
                let x = piece.x.checked_add_signed(dx).unwrap();
                let y = piece.y.checked_add_signed(dy).unwrap();
                if is_square_defended((x, y), colour.opposite(), pieces) {
                    return true;
                }
            }
        }
    }
    false
}

pub fn is_colour_in_checkmate(colour: PieceColour, pieces: &Vec<Piece>) -> bool {
    for piece in pieces {
        if piece.colour == colour {
            let pieces_without_self = pieces.iter()
                .filter_map(|p| if p != piece { Some(p.clone()) } else { None })
                .collect();

            for (move_dx, move_dy) in piece.valid_moves(&pieces_without_self) {
                let new_x = piece.x.checked_add_signed(move_dx).unwrap();
                let new_y = piece.y.checked_add_signed(move_dy).unwrap();
                let pieces_after_move = pieces.iter().filter_map(|p| {
                    if p == piece {
                        let mut new_piece = p.clone();
                        new_piece.x = new_x;
                        new_piece.y = new_y;
                        Some(new_piece)
                    } else if p.occupies_square((new_x, new_y)) {
                        // assume all necessary checks have already been done and this is a piece
                        // of the opposite colour
                        None
                    } else {
                        Some(p.clone())
                    }
                }).collect();

                if !is_colour_in_check(piece.colour, &pieces_after_move) {
                    return false;
                }
            }

            if piece.piece_type != PieceType::Pawn {
                continue;
            }

            for (move_dx, move_dy) in piece.valid_captures(&pieces_without_self) {
                let new_x = piece.x.checked_add_signed(move_dx).unwrap();
                let new_y = piece.y.checked_add_signed(move_dy).unwrap();
                let pieces_after_move = pieces.iter().filter_map(|p| {
                    if p == piece {
                        let mut new_piece = p.clone();
                        new_piece.x = new_x;
                        new_piece.y = new_y;
                        Some(new_piece)
                    } else if p.occupies_square((new_x, new_y)) {
                        // assume all necessary checks have already been done and this is a piece
                        // of the opposite colour
                        None
                    } else {
                        Some(p.clone())
                    }
                }).collect();
                if !is_colour_in_check(piece.colour, &pieces_after_move) {
                    // make sure there is actually a piece to capture somewhere
                    for &(piece_dx, piece_dy) in &piece.squares_occupied {
                        let new_x = new_x.checked_add_signed(piece_dx).unwrap();
                        let new_y = new_y.checked_add_signed(piece_dy).unwrap();
                        for piece in &pieces_without_self {
                            if piece.colour == piece.colour.opposite() && piece.occupies_square((new_x, new_y)) {
                                return false;
                            }
                        }
                    }
                }
            }
        }
    }
    true
}

impl Piece {
    pub fn occupies_square(&self, square: (u8, u8)) -> bool {
        for &(dx, dy) in &self.squares_occupied {
            let x = self.x.checked_add_signed(dx).expect("x of piece < 0");
            let y = self.y.checked_add_signed(dy).expect("y of piece < 0");
            if (x, y) == square {
                return true;
            }
        }
        false
    }

    pub fn valid_captures(&self, pieces: &Vec<Piece>) -> Vec<(i8, i8)> {
        match self.piece_type {
            PieceType::Pawn => {
                let mut moves = Vec::new();
                if self.colour == PieceColour::White {
                    'move_loop: for move_dx in [-1, 1] {
                        for &(piece_dx, piece_dy) in &self.squares_occupied {
                            let Some(new_x) = self.x.checked_add_signed(piece_dx + move_dx) else {
                                continue 'move_loop;
                            };
                            let Some(new_y) = self.y.checked_add_signed(piece_dy + 1) else {
                                continue 'move_loop;
                            };
                            if new_x > 7 || new_y > 7 { continue 'move_loop; }

                            for piece in pieces {
                                // (new_x, new_y) cannot be occupied by a piece of the same colour
                                if piece.colour == self.colour {
                                    if piece.occupies_square((new_x, new_y)) {
                                        continue 'move_loop;
                                    }
                                }
                            }
                        }
                        moves.push((move_dx, 1));
                    }
                } else {
                    'move_loop: for move_dx in [-1, 1] {
                        for &(piece_dx, piece_dy) in &self.squares_occupied {
                            let Some(new_x) = self.x.checked_add_signed(piece_dx + move_dx) else {
                                continue 'move_loop;
                            };
                            let Some(new_y) = self.y.checked_add_signed(piece_dy - 1) else {
                                continue 'move_loop;
                            };
                            if new_x > 7 || new_y > 7 { continue 'move_loop; }

                            for piece in pieces {
                                // (new_x, new_y) cannot be occupied by a piece of the same colour
                                if piece.colour == self.colour {
                                    if piece.occupies_square((new_x, new_y)) {
                                        continue 'move_loop;
                                    }
                                }
                            }
                        }
                        moves.push((move_dx, -1));
                    }
                }
                moves
            },
            _ => self.valid_moves(pieces),
        }
    }

    pub fn valid_moves(&self, pieces: &Vec<Piece>) -> Vec<(i8, i8)>  {
        let mut moves = Vec::new();
        match self.piece_type {
            PieceType::King => {
                let move_deltas = [
                    (-1, -1), (0, -1), (1, -1),
                    (-1, 0), (1, 0),
                    (-1, 1), (0, 1), (1, 1),
                ];
                'move_loop: for (move_dx, move_dy) in move_deltas {
                    for &(piece_dx, piece_dy) in &self.squares_occupied {
                        // (new_x, new_y) is the space occupied by this portion of the piece
                        // after the move
                        let Some(new_x) = self.x.checked_add_signed(piece_dx + move_dx) else {
                            continue 'move_loop;   // new_x < 0, so invalid move
                        };
                        let Some(new_y) = self.y.checked_add_signed(piece_dy + move_dy) else {
                            continue 'move_loop;   // new_y < 0, so invalid move
                        };
                        if new_x > 7 || new_y > 7 { continue 'move_loop; }

                        for piece in pieces {
                            // (new_x, new_y) cannot be occupied by a piece of the same colour
                            if piece.colour == self.colour && piece.occupies_square((new_x, new_y)) {
                                continue 'move_loop;
                            }
                        }
                    }
                    moves.push((move_dx, move_dy));
                }
            },
            PieceType::Queen => {
                let move_directions = [
                    (-1, -1), (0, -1), (1, -1),
                    (-1, 0), (1, 0),
                    (-1, 1), (0, 1), (1, 1),
                ];
                'move_dir_loop: for (move_dx, move_dy) in move_directions {
                    for move_magnitude in 1..=7 {
                        for &(piece_dx, piece_dy) in &self.squares_occupied {
                            // (new_x, new_y) is the space occupied by this portion of the piece
                            // after the move
                            let Some(new_x) = self.x.checked_add_signed(
                                piece_dx + move_dx * move_magnitude) else {
                                continue 'move_dir_loop;   // new_x < 0, so invalid move
                            };
                            let Some(new_y) = self.y.checked_add_signed(
                                piece_dy + move_dy * move_magnitude) else {
                                continue 'move_dir_loop;   // new_y < 0, so invalid move
                            };
                            if new_x > 7 || new_y > 7 { continue 'move_dir_loop; }

                            for piece in pieces {
                                // (new_x, new_y) cannot be occupied by a piece of the same colour
                                if piece.occupies_square((new_x, new_y)) {
                                    // if the piece is of the opposite colour, that's fine, we can capture it,
                                    // but we can't move past it
                                    if piece.colour != self.colour {
                                        moves.push((move_dx * move_magnitude, move_dy * move_magnitude));
                                    }
                                    continue 'move_dir_loop;
                                }
                            }
                        }
                        moves.push((move_dx * move_magnitude, move_dy * move_magnitude));
                    }
                }
            },
            PieceType::Bishop => {
                let move_directions = [
                    (-1, -1), (1, -1),
                    (-1, 1), (1, 1),
                ];
                'move_dir_loop: for (move_dx, move_dy) in move_directions {
                    for move_magnitude in 1..=7 {
                        for &(piece_dx, piece_dy) in &self.squares_occupied {
                            // (new_x, new_y) is the space occupied by this portion of the piece
                            // after the move
                            let Some(new_x) = self.x.checked_add_signed(
                                piece_dx + move_dx * move_magnitude) else {
                                continue 'move_dir_loop;   // new_x < 0, so invalid move
                            };
                            let Some(new_y) = self.y.checked_add_signed(
                                piece_dy + move_dy * move_magnitude) else {
                                continue 'move_dir_loop;   // new_y < 0, so invalid move
                            };
                            if new_x > 7 || new_y > 7 { continue 'move_dir_loop; }

                            for piece in pieces {
                                // (new_x, new_y) cannot be occupied by a piece of the same colour
                                if piece.occupies_square((new_x, new_y)) {
                                    // if the piece is of the opposite colour, that's fine, we can capture it,
                                    // but we can't move past it
                                    if piece.colour != self.colour {
                                        moves.push((move_dx * move_magnitude, move_dy * move_magnitude));
                                    }
                                    continue 'move_dir_loop;
                                }
                            }
                        }
                        moves.push((move_dx * move_magnitude, move_dy * move_magnitude));
                    }
                }
            },
            PieceType::Knight => {
                let move_deltas = [
                    (-2, -3), (-2, 3),
                    (2, -3), (2, 3),
                    (-3, -2), (-3, 2),
                    (3, -2), (3, 2),
                ];
                'move_loop: for (move_dx, move_dy) in move_deltas {
                    for &(piece_dx, piece_dy) in &self.squares_occupied {
                        // (new_x, new_y) is the space occupied by this portion of the piece
                        // after the move
                        let Some(new_x) = self.x.checked_add_signed(piece_dx + move_dx) else {
                            continue 'move_loop;   // new_x < 0, so invalid move
                        };
                        let Some(new_y) = self.y.checked_add_signed(piece_dy + move_dy) else {
                            continue 'move_loop;   // new_y < 0, so invalid move
                        };
                        if new_x > 7 || new_y > 7 { continue 'move_loop; }

                        for piece in pieces {
                            // (new_x, new_y) cannot be occupied by a piece of the same colour
                            if piece.colour == self.colour {
                                if piece.occupies_square((new_x, new_y)) {
                                    continue 'move_loop;
                                }
                            }
                        }
                    }
                    moves.push((
                        self.x as i8 + move_dx,
                        self.y as i8 + move_dy,
                    ));
                }
            },
            PieceType::Rook => {
                let move_directions = [
                    (-1, 0), (1, 0),
                    (0, -1), (0, 1),
                ];
                'move_dir_loop: for (move_dx, move_dy) in move_directions {
                    for move_magnitude in 1..=7 {
                        for &(piece_dx, piece_dy) in &self.squares_occupied {
                            // (new_x, new_y) is the space occupied by this portion of the piece
                            // after the move
                            let Some(new_x) = self.x.checked_add_signed(
                                piece_dx + move_dx * move_magnitude) else {
                                continue 'move_dir_loop;   // new_x < 0, so invalid move
                            };
                            let Some(new_y) = self.y.checked_add_signed(
                                piece_dy + move_dy * move_magnitude) else {
                                continue 'move_dir_loop;   // new_y < 0, so invalid move
                            };
                            if new_x > 7 || new_y > 7 { continue 'move_dir_loop; }

                            for piece in pieces {
                                // (new_x, new_y) cannot be occupied by a piece of the same colour
                                if piece.occupies_square((new_x, new_y)) {
                                    // if the piece is of the opposite colour, that's fine, we can capture it,
                                    // but we can't move past it
                                    if piece.colour != self.colour {
                                        moves.push((move_dx * move_magnitude, move_dy * move_magnitude));
                                    }
                                    continue 'move_dir_loop;
                                }
                            }
                        }
                        moves.push((
                            self.x as i8 + move_dx * move_magnitude,
                            self.y as i8 + move_dy * move_magnitude,
                        ));
                    }
                }
            },
            PieceType::Pawn => {
                if self.colour == PieceColour::White {
                    // one square forward
                    for &(piece_dx, piece_dy) in &self.squares_occupied {
                        let Some(new_x) = self.x.checked_add_signed(piece_dx) else {
                            return moves; // we can return early here because if we can't move 1 square forward,
                                            // we definitely can't move 2 squares forward
                        };
                        let Some(new_y) = self.y.checked_add_signed(piece_dy + 1) else {
                            return moves;
                        };
                        if new_x > 7 || new_y > 7 { return moves; }

                        for piece in pieces {
                            // (new_x, new_y) cannot be occupied by a piece of any colour
                            if piece.occupies_square((new_x, new_y)) {
                                return moves;
                            }
                        }
                    }
                    moves.push((0, 1));

                    // two squares forward
                    // ignoring squares_occupied now because the pawn can't have moved or captured
                    if self.y == 1 {
                        for piece in pieces {
                            // (new_x, new_y) cannot be occupied by a piece of any colour
                            if piece.occupies_square((self.x, self.y + 2)) {
                                return moves;
                            }
                        }
                        moves.push((0, 2));
                    }
                } else {
                    // one square forward
                    for &(piece_dx, piece_dy) in &self.squares_occupied {
                        let Some(new_x) = self.x.checked_add_signed(piece_dx) else {
                            return moves; // we can return early here because if we can't move 1 square forward,
                                            // we definitely can't move 2 squares forward
                        };
                        let Some(new_y) = self.y.checked_add_signed(piece_dy - 1) else {
                            return moves;
                        };
                        if new_x > 7 || new_y > 7 { return moves; }

                        for piece in pieces {
                            // (new_x, new_y) cannot be occupied by a piece of any colour
                            if piece.occupies_square((new_x, new_y)) {
                                return moves;
                            }
                        }
                    }
                    moves.push((0, -1));

                    // two squares forward
                    // ignoring squares_occupied now because the pawn can't have moved or captured
                    if self.y == 6 {
                        for piece in pieces {
                            // (new_x, new_y) cannot be occupied by a piece of any colour
                            if piece.occupies_square((self.x, self.y - 2)) {
                                return moves;
                            }
                        }
                        moves.push((0, -2));
                    }
                }
            }, 
        }
        moves
    }

    pub fn is_move_valid(&self, new_position: (u8, u8), pieces: &Vec<Piece>) -> bool {
        let pieces_without_self = pieces.iter()
            .filter_map(|p| if p != self { Some(p.clone()) } else { None })
            .collect();

        for (move_dx, move_dy) in self.valid_moves(&pieces_without_self) {
            let new_x = self.x.checked_add_signed(move_dx).unwrap();
            let new_y = self.y.checked_add_signed(move_dy).unwrap();
            if (new_x, new_y) == new_position {
                return true;
            }
        }

        // it bothers me that the next check is in practice only required for non-pawns, so i'll
        // just return false if we're not a pawn here
        if self.piece_type != PieceType::Pawn {
            return false;
        }

        for (move_dx, move_dy) in self.valid_captures(&pieces_without_self) {
            let new_x = self.x.checked_add_signed(move_dx).unwrap();
            let new_y = self.y.checked_add_signed(move_dy).unwrap();
            if (new_x, new_y) == new_position {
                // make sure there is actually a piece to capture somewhere
                for &(piece_dx, piece_dy) in &self.squares_occupied {
                    let new_x = new_x.checked_add_signed(piece_dx).unwrap();
                    let new_y = new_y.checked_add_signed(piece_dy).unwrap();
                    for piece in &pieces_without_self {
                        if piece.colour == self.colour.opposite() && piece.occupies_square((new_x, new_y)) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Like is_move_valid, but also checks if the king would be in check afterwards
    pub fn is_move_playable(&self, new_position: (u8, u8), pieces: &Vec<Piece>) -> bool {
        if !self.is_move_valid(new_position, pieces) {
            return false;
        }

        let pieces_after_move = pieces.iter().filter_map(|piece| {
            if piece == self {
                let mut new_piece = piece.clone();
                new_piece.x = new_position.0;
                new_piece.y = new_position.1;
                Some(new_piece)
            } else if piece.occupies_square(new_position) {
                // assume all necessary checks have already been done and this is a piece
                // of the opposite colour
                None
            } else {
                Some(piece.clone())
            }
        }).collect();

        if is_colour_in_check(self.colour, &pieces_after_move) {
            return false;
        }

        true
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
                    is_symmetric = true;
                    break;
                }
            }
            if !is_symmetric {
                break;
            }
        }
        if is_symmetric {
            self.transform.rotation = Quat::IDENTITY;

            // transform.scale.x = 2 * length from centre in direction of rotation
            // transform.scale.z = 2 * length from centre perp. to direction of rotation
            let mut max_x = self.offset.x;
            let mut max_y = self.offset.z;
            for (x, y) in &self.squares_occupied {
                if (*y as f32) == self.offset.z.floor() && (*x as f32) > max_x {
                    max_x = *x as f32;
                }
                if (*x as f32) == self.offset.x.floor() && (*y as f32) > max_y {
                    max_y = *y as f32;
                }
            }

            self.transform.scale.x = (max_x - self.offset.x + 0.5) * 2.0;
            self.transform.scale.z = (max_y - self.offset.z + 0.5) * 2.0;
            self.transform.scale.y *= 1.414;
        } else {
            self.transform.rotation = Quat::from_rotation_y(-PI/4.0);

            // transform.scale.x = 2 * length from centre in direction of rotation
            // transform.scale.z = 2 * length from centre perp. to direction of rotation
            let mut max_x = self.offset.x;
            let mut max_y = self.offset.z;
            for (x, y) in &self.squares_occupied {
                // project top right corner onto pi/4 diagonal and bottom right corner onto -pi/4 diagonal
                let x_mag = ((*x as f32 + 0.5 - self.offset.x) + (*y as f32 + 0.5 - self.offset.z)) * 1.414;
                let y_mag = ((*x as f32 + 0.5 - self.offset.x) - (*y as f32 - 0.5 - self.offset.z)) * 1.414;
                if x_mag > max_x { max_x = x_mag; }
                if y_mag > max_y { max_y = y_mag; }
            }

            self.transform.scale.x = max_x;//(max_x - self.offset.x + 0.5) * 2.0 * 1.414;
            self.transform.scale.z = max_y;//(max_y - self.offset.z + 0.5) * 2.0 * 1.414;
            self.transform.scale.y *= 1.1;
        }
    }

    pub fn consume_piece(&mut self, x: u8, y: u8) {
        if x as i8 - self.x as i8 == y as i8 - self.y as i8 {
            // diagonal capture
            if x < self.x {
                for (x, y) in self.squares_occupied.clone() {
                    self.squares_occupied.insert((x + 1, y + 1));
                }
            } else {
                for (x, y) in self.squares_occupied.clone() {
                    self.squares_occupied.insert((x - 1, y - 1));
                }
            }
        } else if x as i8 - self.x as i8 == self.y as i8 - y as i8 {
            // diagonal capture (the other way)
            if x < self.x {
                for (x, y) in self.squares_occupied.clone() {
                    self.squares_occupied.insert((x + 1, y - 1));
                }
            } else {
                for (x, y) in self.squares_occupied.clone() {
                    self.squares_occupied.insert((x - 1, y + 1));
                }
            }
        } else if x == self.x {
            // vertical capture
            if y < self.y {
                for (x, y) in self.squares_occupied.clone() {
                    self.squares_occupied.insert((x, y + 1));
                }
            } else if y > self.y {
                for (x, y) in self.squares_occupied.clone() {
                    self.squares_occupied.insert((x, y - 1));
                }
            } else {
                panic!("attempted to capture on own square");
            }
        } else if y == self.y {
            // horizontal capture
            if x < self.x {
                for (x, y) in self.squares_occupied.clone() {
                    self.squares_occupied.insert((x + 1, y));
                }
            } else if x > self.x {
                for (x, y) in self.squares_occupied.clone() {
                    self.squares_occupied.insert((x - 1, y));
                }
            } else {
                panic!("attempted to capture on own square");
            }
        }


        // if there are any gaps they should be filled
        for (x, y) in self.squares_occupied.clone() {
            if
                !self.squares_occupied.contains(&(x + 1, y))
                && self.squares_occupied.contains(&(x + 2, y))
                && self.squares_occupied.contains(&(x + 1, y + 1))
                && self.squares_occupied.contains(&(x + 1, y - 1))
            {
                self.squares_occupied.insert((x + 1, y));
            }
        }

        self.update_transform();
    }
}

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        let direction = piece.transform.translation - transform.translation;
        let scale_diff = piece.transform.scale - transform.scale;
        if direction.length() > 0.01 {
            transform.translation += direction.normalize() * time.delta_seconds() * 4.0 * direction.length().sqrt();
        }
        if scale_diff.length() > 0.01 {
            transform.scale += scale_diff.normalize() * time.delta_seconds() * 4.0 * scale_diff.length().sqrt();
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
            squares_occupied: HashSet::from([(0, 0)]),
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
