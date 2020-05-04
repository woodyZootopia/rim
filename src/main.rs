extern crate termion;

use std::io::{stdin, stdout, Write};
// use termion::color;
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;

// use std::env;
use std::fs;

fn main() {
    let text = fs::read_to_string("test.txt").unwrap();
    let texts: Vec<&str> = text.lines().collect();
    let stdin = stdin();
    let mut stdout = AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode().unwrap()));
    write!(stdout,"{}",termion::clear::All).unwrap();
    for (i, text) in texts.iter().enumerate() {
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(1, i as u16 +1),
            text
        ).unwrap();
    stdout.flush().unwrap();
    }

    for c in stdin.events() {
        let evt = c.unwrap();
        match evt {
            Event::Key(kc) => match kc {
                Key::Char('q') => break,
                // Key::Char('j') => write!(stdout, "{}", termion::cursor::Goto()).unwrap(),
                _ => (),
            },
            Event::Mouse(me) => match me {
                MouseEvent::Press(_, x, y) => {
                    write!(stdout, "{}x", termion::cursor::Goto(x, y)).unwrap();
                }
                _ => (),
            },
            _ => (),
        }
        stdout.flush().unwrap();
    }
}
