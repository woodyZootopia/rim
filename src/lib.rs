pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();
        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a filename"),
        };
        Ok(Config { filename })
    }
}
pub mod util {
    use super::Config;
    use std::fs;
    use std::io::{stdin, stdout, Stdin, Stdout, Write};
    // use termion::color;
    use termion::event::{Event, Key, MouseEvent};
    use termion::input::{MouseTerminal, TermRead};
    use termion::raw::IntoRawMode;
    use termion::screen::AlternateScreen;

    struct Cursor {
        x: usize,
        y: usize,
    }
    enum State {
        Normal,
        Insert,
    }
    pub struct Buffer {
        cursor: Cursor,
        state: State,
        stdin: Stdin,
        stdout: Box<dyn Write>,
        text: Vec<Vec<char>>,
    }

    impl Buffer {
        pub fn new(config: Config) -> Buffer {
            let text = fs::read_to_string(config.filename).unwrap();
            let text: Vec<Vec<char>> = text.lines().map(|x| x.chars().collect()).collect();
            let stdin = stdin();
            let mut stdout =
                AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode().unwrap()));
            write!(stdout, "{}", termion::clear::All).unwrap();
            for (i, column) in text.iter().enumerate() {
                for (j, ch) in column.iter().enumerate() {
                    write!(
                        stdout,
                        "{}{}",
                        termion::cursor::Goto(j as u16 + 1, i as u16 + 1),
                        ch
                    )
                    .unwrap();
                }
            }
            write!(stdout, "{}", termion::cursor::Goto(1, 1));
            stdout.flush().unwrap();
            Buffer {
                cursor: Cursor { x: 0, y: 0 },
                state: State::Normal,
                stdin,
                stdout: Box::new(stdout),
                text,
            }
        }

        pub fn buffer_loop(mut self) -> () {
            for c in self.stdin.events() {
                let evt = c.unwrap();
                match evt {
                    Event::Key(kc) => match kc {
                        Key::Char('q') => break,
                        Key::Char('h') => {
                            if self.cursor.x >= 1 {
                                self.cursor.x -= 1;
                            }
                        }
                        Key::Char('j') => {
                            if self.cursor.y + 1 < self.text.len() {
                                self.cursor.y += 1;
                            }
                        }
                        Key::Char('k') => {
                            if self.cursor.y >= 1 {
                                self.cursor.y -= 1;
                            }
                        }
                        Key::Char('l') => {
                            if self.cursor.x + 1 < self.text[self.cursor.y].len() {
                                self.cursor.x += 1;
                            }
                        }
                        _ => (),
                    },
                    Event::Mouse(me) => match me {
                        MouseEvent::Press(_, x, y) => {
                            write!(self.stdout, "{}x", termion::cursor::Goto(x, y)).unwrap();
                        }
                        _ => (),
                    },
                    _ => (),
                }
                write!(
                    self.stdout,
                    "{}",
                    termion::cursor::Goto(self.cursor.x as u16 + 1, self.cursor.y as u16 + 1)
                )
                .unwrap();
                self.stdout.flush().unwrap();
            }
        }
    }
}
