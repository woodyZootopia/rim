extern crate termion;

mod lib;
use lib::Config;
use lib::EditorState;
use std::env;
use std::io::{stdin, stdout};
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

fn main() {
    let config = Config::new(env::args()).unwrap();
    let stdin = stdin();
    let stdin = stdin.lock();
    let stdout = AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode().unwrap()));
    // let mut stdout = Box::from(stdout);
    let editor = EditorState::new(stdin, stdout, config);
    editor.editor_loop();
}
