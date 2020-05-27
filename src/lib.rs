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

    type Text = Vec<Vec<char>>;

    trait UpdateScreen {
        fn rewrite_entire_screen(&self, stdout: &mut Box<dyn Write>) -> ();
        fn rewrite_single_line(&self, stdout: &mut Box<dyn Write>, line_to_rewrite: usize) -> ();
    }

    impl UpdateScreen for Text {
        fn rewrite_entire_screen(&self, stdout: &mut Box<dyn Write>) {
            write!(stdout, "{}", termion::clear::All).unwrap();
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
            write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
            stdout.flush().unwrap();
        }
        fn rewrite_single_line(&self, stdout: &mut Box<dyn Write>, line_to_rewrite: usize) {
            let column = self.iter().collect::<Vec<&Vec<char>>>()[line_to_rewrite];
            write!(
                stdout,
                "{}{}",
                termion::cursor::Goto(0, line_to_rewrite as u16 + 1),
                termion::clear::CurrentLine
            )
            .unwrap();
            for (j, ch) in column.iter().enumerate() {
                write!(
                    stdout,
                    "{}{}",
                    termion::cursor::Goto(j as u16 + 1, line_to_rewrite as u16 + 1),
                    ch
                )
                .unwrap();
            }
            return;
        }
    }

    pub struct Editor {
        cursor: Cursor,
        stdin: Stdin,
        stdout: Box<dyn Write>,
        text: Text,
    }

    #[derive(Debug)]
    enum Mode {
        Normal,
        Insert,
        Ex,
    }

    impl Editor {
        pub fn new(config: Config) -> Editor {
            let text = fs::read_to_string(config.filename).unwrap();
            let text: Vec<Vec<char>> = text.lines().map(|x| x.chars().collect()).collect();
            let stdin = stdin();
            let stdout =
                AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode().unwrap()));
            let mut stdout = Box::from(stdout);

            write!(stdout, "{}", termion::clear::All).unwrap();

            Editor {
                cursor: Cursor { x: 0, y: 0 },
                stdin,
                stdout,
                text,
            }
        }

        pub fn editor_loop(mut self) -> () {
            self.text.rewrite_entire_screen(&mut self.stdout);
            let mut mode = Mode::Normal;
            for c in self.stdin.events() {
                let evt = c.unwrap();
                let mut line_to_rewrite: Option<usize> = None;
                let mut flag_rewrite_all = false;
                match mode {
                    Mode::Normal => match evt {
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
                                    if self.cursor.x > self.text[self.cursor.y].len() {
                                        self.cursor.x = self.text[self.cursor.y].len();
                                    }
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
                            Key::Char('w') => {
                                let mut has_seen_space = false;
                                loop {
                                    if self.cursor.x + 1 >= self.text[self.cursor.y].len() {
                                        break;
                                    }
                                    match self.text[self.cursor.y][self.cursor.x] {
                                        'a'..='z' => {
                                            if has_seen_space {
                                                break;
                                            }
                                            self.cursor.x += 1;
                                        }
                                        ' ' => {
                                            has_seen_space = true;
                                            self.cursor.x += 1;
                                        }
                                        _ => break,
                                    }
                                }
                            }
                            Key::Char('x') => {
                                if self.text[self.cursor.y].len() >= 1 {
                                    self.text[self.cursor.y].remove(self.cursor.x);
                                    if self.cursor.x >= self.text[self.cursor.y].len()
                                        && self.cursor.x > 0
                                    {
                                        self.cursor.x -= 1;
                                    }
                                }
                                line_to_rewrite = Some(self.cursor.y);
                            }
                            Key::Char('i') => {
                                mode = Mode::Insert;
                            }
                            Key::Char('a') => {
                                self.cursor.x += 1;
                                mode = Mode::Insert;
                                line_to_rewrite = Some(self.cursor.y);
                            }
                            Key::Char('I') => {
                                self.cursor.x = 0;
                                mode = Mode::Insert;
                                line_to_rewrite = Some(self.cursor.y);
                            }
                            Key::Char('A') => {
                                self.cursor.x = self.text[self.cursor.y].len();
                                mode = Mode::Insert;
                                line_to_rewrite = Some(self.cursor.y);
                            }
                            Key::Char(':') => {
                                mode = Mode::Ex;
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
                    },
                    Mode::Insert => match evt {
                        Event::Key(Key::Esc) => {
                            mode = Mode::Normal;
                        }
                        Event::Key(Key::Char(ch)) => {
                            self.text[self.cursor.y].insert(self.cursor.x, ch);
                            self.cursor.x += 1;
                            line_to_rewrite = Some(self.cursor.y);
                        }
                        Event::Key(Key::Backspace) if self.cursor.x >= 1 => {
                            self.text[self.cursor.y].remove(self.cursor.x - 1);
                            self.cursor.x -= 1;
                            line_to_rewrite = Some(self.cursor.y);
                        }
                        _ => (),
                    },
                    Mode::Ex => match evt {
                        Event::Key(Key::Esc) => {
                            mode = Mode::Normal;
                        }
                        _ => (),
                    },
                }
                if flag_rewrite_all {
                    self.text.rewrite_entire_screen(&mut self.stdout);
                }
                match line_to_rewrite {
                    Some(line) => self.text.rewrite_single_line(&mut self.stdout, line),
                    None => (),
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
