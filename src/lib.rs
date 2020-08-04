pub mod editor;
pub mod screen;
pub mod text;
pub mod util;
use std::fs;
use std::io::{BufRead, Error, Write};
use termion;
use termion::event::{Event, Key, MouseEvent};
use termion::input::TermRead;

#[cfg(test)]
mod tests {
    use super::*;
    // use std::io::{stdin, stdout, Stdout};
    // use termion::input::MouseTerminal;
    // use termion::raw::IntoRawMode;
    // use termion::screen::AlternateScreen;
    #[test]
    fn test_basic_cursor_move() {}
    // fn test_basic_cursor_move() {
    //     let config = Config {
    //         filepath: "./test.txt".to_string(),
    //     };
    //     let stdio = stdin();
    //     let input = stdio.lock();
    //     // let output = stdout();
    //     let mut output =
    //         AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode().unwrap()));
    //     let editor = EditorState::new("w".into(), output, config);
    // }
}
