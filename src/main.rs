extern crate termion;

use rim::Config;
use rim::EditorState;
use std::env;
use std::io::{stdin, stdout};
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

fn main() {
    let config = Config::new(env::args()).unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();
    let mut stdout = AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode().unwrap()));
    // let mut stdout = Box::from(stdout);
    let editor = EditorState::new(stdin, stdout, config);
    editor.editor_loop();
}
