use crate::text::*;
use std::cmp;
#[derive(Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

#[derive(Default)]
pub struct ScreenState {
    pub cursor: Cursor,
    pub row_offset: usize,
    pub terminal_size: (u16, u16),
}

impl ScreenState {
    /// Move key vertically. After that, make sure key is in valid place.
    /// Returns the line to rewrite.
    pub fn move_vert(&mut self, text: &TextState, movement: i32) -> Option<usize> {
        let mut line_to_rewrite = None;
        if movement > 0 {
            if self.cursor.y as i32 + movement > self.terminal_size.1 as i32 - 2 {
                self.cursor.y -= movement as usize;
                self.row_offset += movement as usize;
                line_to_rewrite = Some(self.cursor.y + movement as usize);
            }
            self.cursor.y += movement as usize;
        }
        if movement < 0 {
            let distance = -movement as usize;
            if self.cursor.y as i32 + movement < 0 {
                if self.row_offset > 0 {
                    self.row_offset -= distance;
                }
                self.cursor.y += distance;
                line_to_rewrite = Some(self.cursor.y - distance);
            }
            self.cursor.y -= -movement as usize;
        }

        if self.cursor.x > text[self.cursor.y + self.row_offset].len() {
            self.cursor.x = cmp::max(text[self.cursor.y + self.row_offset].len(), 1) - 1;
        }
        return line_to_rewrite;
    }
    /// Move key horizontally. After that, make sure key is in valid place.
    pub fn move_horiz(&mut self, text: &TextState, distance: i32) {
        if distance < 0 {
            let distance = -distance as usize;
            self.cursor.x -= cmp::min(self.cursor.x, distance);
        } else {
            let distance = distance as usize;
            self.cursor.x = cmp::min(
                self.cursor.x + distance,
                cmp::max(text[self.cursor.y + self.row_offset].len(), 1),
            );
        }
    }
}

