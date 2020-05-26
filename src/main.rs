extern crate termion;

use rim::Config;
use rim::util::*;
use std::env;

fn main() {
    let config = Config::new(env::args()).unwrap();

    let editor = Editor::new(config);
    editor.editor_loop();

}
