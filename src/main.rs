extern crate termion;

use rim::Config;
use rim::EditorState;
use std::env;

fn main() {
    let config = Config::new(env::args()).unwrap();

    let editor = EditorState::new(config);
    editor.editor_loop();

}
