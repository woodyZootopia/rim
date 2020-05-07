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

    type Text = Vec<Vec<char>>;

    trait UpdateScreen {
        fn rewrite_entire_screen(&self, stdout: &mut Box<dyn Write>) -> ();
    }

    impl UpdateScreen for Text {
        fn rewrite_entire_screen(&self, stdout: &mut Box<dyn Write>) {
            write!(stdout, "{}", termion::cursor::Goto(1, 1));
            stdout.flush().unwrap();
            for (i, column) in self.iter().enumerate() {
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
            stdout.flush();
        }
    }

    pub struct Buffer {
        cursor: Cursor,
        state: State,
        stdin: Stdin,
        stdout: Box<dyn Write>,
        text: Text,
    }

    impl Buffer {
        pub fn new(config: Config) -> Buffer {
            let text = fs::read_to_string(config.filename).unwrap();
            let text: Vec<Vec<char>> = text.lines().map(|x| x.chars().collect()).collect();
            let stdin = stdin();
            let stdout =
                AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode().unwrap()));
            let mut stdout = Box::from(stdout);

            write!(stdout, "{}", termion::clear::All).unwrap();

            Buffer {
                cursor: Cursor { x: 0, y: 0 },
                state: State::Normal,
                stdin,
                stdout,
                text,
            }
        }

        pub fn buffer_loop(mut self) -> () {
            self.text.rewrite_entire_screen(&mut self.stdout);
            for c in self.stdin.events() {
                let evt = c.unwrap();
                let mut flag_rewrite = false;
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
                                if self.cursor.x > self.text[self.cursor.y].len() {
                                    self.cursor.x = self.text[self.cursor.y].len();
                                }
                            }
                        }
                        Key::Char('l') => {
                            if self.cursor.y < self.text.len()
                                && self.cursor.x + 1 < self.text[self.cursor.y].len()
                            {
                                self.cursor.x += 1;
                            }
                        }
                        Key::Char('x') => {
                            self.text[self.cursor.y].remove(self.cursor.x);
                            flag_rewrite = true;
                        }
                        _ => (),
                    },
                    Event::Mouse(me) => match me {
                        MouseEvent::Press(_, x, y) => {
                            self.cursor = Cursor {
                                x: x as usize - 1,
                                y: y as usize - 1,
                            };
                        }
                        _ => (),
                    },
                    _ => (),
                }
                if flag_rewrite {
                    self.text.rewrite_entire_screen(&mut self.stdout);
                }
                write!(
                    self.stdout,
                    "{}",
                    termion::cursor::Goto(self.cursor.x as u16 + 1, self.cursor.y as u16 + 1)
                )
                .unwrap();
                self.stdout.flush();
            }
        }
    }
}
