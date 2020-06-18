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
    use std::cmp;
    use std::fs;
    use std::io::{stdin, stdout, Stdin, Stdout, Write};
    use termion;
    use termion::event::{Event, Key, MouseEvent};
    use termion::input::{MouseTerminal, TermRead};
    use termion::raw::IntoRawMode;
    use termion::screen::AlternateScreen;

    fn debug_print(stdout: &mut Box<dyn Write>, mode: &Mode, args: Vec<String>) {
        write!(
            stdout,
            "{}{}{}",
            termion::cursor::Goto(0, termion::terminal_size().unwrap().1),
            match mode {
                Mode::Normal => termion::color::Bg(termion::color::Rgb(145, 172, 209)),
                Mode::Insert => termion::color::Bg(termion::color::Rgb(192, 202, 142)),
                Mode::Command => termion::color::Bg(termion::color::Rgb(233, 144, 144)),
                _ => termion::color::Bg(termion::color::Rgb(145, 172, 209)),
            },
            termion::color::Fg(termion::color::Black),
        )
        .unwrap();
        for item in args {
            write!(stdout, "{}, ", item).unwrap();
        }
        write!(
            stdout,
            "{}{}",
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset),
        )
        .unwrap();
    }

    struct Cursor {
        x: usize,
        y: usize,
    }

    type Text = Vec<Vec<char>>;

    trait UpdateScreen {
        fn rewrite_entire_screen(&self, stdout: &mut Box<dyn Write>, row_offset: usize) -> ();
        fn rewrite_single_line(
            &self,
            stdout: &mut Box<dyn Write>,
            line_to_rewrite: usize,
            row_offset: usize,
        ) -> ();
    }

    impl UpdateScreen for Text {
        fn rewrite_entire_screen(&self, stdout: &mut Box<dyn Write>, row_offset: usize) {
            write!(stdout, "{}", termion::clear::All).unwrap();
            for (i, column) in self[row_offset
                ..cmp::min(
                    termion::terminal_size().unwrap().1 as usize + row_offset - 1,
                    self.len(),
                )]
                .iter()
                .enumerate()
            {
                write!(
                    stdout,
                    "{}{}",
                    termion::cursor::Goto(0, i as u16 + 1),
                    column.iter().collect::<String>()
                )
                .unwrap();
            }
            write!(stdout, "{}", termion::cursor::Goto(1, 1)).unwrap();
            stdout.flush().unwrap();
        }
        fn rewrite_single_line(
            &self,
            stdout: &mut Box<dyn Write>,
            line_to_rewrite: usize,
            row_offset: usize,
        ) {
            let column = self.iter().collect::<Vec<&Vec<char>>>()[line_to_rewrite + row_offset];
            write!(
                stdout,
                "{}{}",
                termion::cursor::Goto(0, line_to_rewrite as u16 + 1),
                termion::clear::CurrentLine
            )
            .unwrap();
            write!(
                stdout,
                "{}{}",
                termion::cursor::Goto(0, line_to_rewrite as u16 + 1),
                column.iter().collect::<String>()
            )
            .unwrap();
            return;
        }
    }

    pub struct EditorState {
        screen: ScreenState,
        io: IO,
        text: Text,
    }

    pub struct ScreenState {
        cursor: Cursor,
        row_offset: usize,
    }

    impl ScreenState {
        pub fn adjust_screen_offset(&mut self) -> bool {
            false
        }
    }

    pub struct IO {
        stdin: Stdin,
        stdout: Box<dyn Write>,
    }

    #[derive(Debug)]
    enum Mode {
        Normal,
        Insert,
        Command,
    }

    impl std::fmt::Display for Mode {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl EditorState {
        pub fn new(config: Config) -> EditorState {
            let text = fs::read_to_string(config.filename).unwrap();
            let text: Vec<Vec<char>> = text.lines().map(|x| x.chars().collect()).collect();
            let stdin = stdin();
            let stdout =
                AlternateScreen::from(MouseTerminal::from(stdout().into_raw_mode().unwrap()));
            let mut stdout = Box::from(stdout);

            write!(stdout, "{}", termion::clear::All).unwrap();

            EditorState {
                text,
                screen: ScreenState {
                    cursor: Cursor { x: 0, y: 0 },
                    row_offset: 0,
                },
                io: IO { stdin, stdout },
            }
        }

        pub fn editor_loop(mut self) -> () {
            self.text.rewrite_entire_screen(&mut self.io.stdout, 0);
            let mut mode = Mode::Normal;
            for c in self.io.stdin.events() {
                let evt = c.unwrap();
                let mut line_to_rewrite: Option<usize> = None;
                let mut flag_rewrite_all = false;
                match mode {
                    Mode::Normal => match evt {
                        Event::Key(key) => match key {
                            Key::Char(ch) => match ch {
                                'q' => break,
                                'h' => {
                                    if self.screen.cursor.x >= 1 {
                                        self.screen.cursor.x -= 1;
                                    }
                                }
                                'j' => {
                                    if self.screen.cursor.y + self.screen.row_offset + 1
                                        < self.text.len()
                                    {
                                        self.screen.cursor.y += 1;
                                        if self.screen.cursor.x
                                            > self.text
                                                [self.screen.cursor.y + self.screen.row_offset]
                                                .len()
                                        {
                                            self.screen.cursor.x = cmp::max(
                                                self.text
                                                    [self.screen.cursor.y + self.screen.row_offset]
                                                    .len()
                                                    as i32
                                                    - 1,
                                                0,
                                            )
                                                as usize;
                                        }
                                        if self.screen.cursor.y + 1
                                            > termion::terminal_size().unwrap().1 as usize - 1
                                        {
                                            self.screen.cursor.y -= 1;
                                            self.screen.row_offset += 1;
                                            flag_rewrite_all = true;
                                        }
                                    }
                                }
                                'k' => {
                                    if self.screen.cursor.y + self.screen.row_offset >= 1 {
                                        if self.screen.cursor.y == 0 {
                                            self.screen.row_offset -= 1;
                                            flag_rewrite_all = true;
                                        } else {
                                            self.screen.cursor.y -= 1;
                                        }
                                        if self.screen.cursor.x
                                            >= self.text
                                                [self.screen.cursor.y + self.screen.row_offset]
                                                .len()
                                        {
                                            self.screen.cursor.x = cmp::max(
                                                self.text
                                                    [self.screen.cursor.y + self.screen.row_offset]
                                                    .len()
                                                    as i32
                                                    - 1,
                                                0,
                                            )
                                                as usize;
                                        }
                                    }
                                }
                                'l' => {
                                    if self.screen.cursor.y + self.screen.row_offset
                                        < self.text.len()
                                        && self.screen.cursor.x + 1
                                            < self.text
                                                [self.screen.cursor.y + self.screen.row_offset]
                                                .len()
                                    {
                                        self.screen.cursor.x += 1;
                                    }
                                }
                                'w' => {
                                    let mut has_seen_space = false;
                                    loop {
                                        if self.screen.cursor.x + 1
                                            >= self.text
                                                [self.screen.cursor.y + self.screen.row_offset]
                                                .len()
                                        {
                                            break;
                                        }
                                        match self.text
                                            [self.screen.cursor.y + self.screen.row_offset]
                                            [self.screen.cursor.x]
                                        {
                                            'a'..='z' => {
                                                if has_seen_space {
                                                    break;
                                                }
                                                self.screen.cursor.x += 1;
                                            }
                                            ' ' => {
                                                has_seen_space = true;
                                                self.screen.cursor.x += 1;
                                            }
                                            _ => break,
                                        }
                                    }
                                }
                                'x' => {
                                    if self.text[self.screen.cursor.y + self.screen.row_offset]
                                        .len()
                                        >= 1
                                    {
                                        self.text[self.screen.cursor.y + self.screen.row_offset]
                                            .remove(self.screen.cursor.x);
                                        if self.screen.cursor.x
                                            >= self.text
                                                [self.screen.cursor.y + self.screen.row_offset]
                                                .len()
                                            && self.screen.cursor.x > 0
                                        {
                                            self.screen.cursor.x -= 1;
                                        }
                                    }
                                    line_to_rewrite = Some(self.screen.cursor.y);
                                }
                                'i' => {
                                    mode = Mode::Insert;
                                }
                                'a' => {
                                    self.screen.cursor.x += 1;
                                    mode = Mode::Insert;
                                    line_to_rewrite = Some(self.screen.cursor.y);
                                }
                                'I' => {
                                    self.screen.cursor.x = 0;
                                    mode = Mode::Insert;
                                    line_to_rewrite = Some(self.screen.cursor.y);
                                }
                                'A' => {
                                    self.screen.cursor.x = self.text
                                        [self.screen.cursor.y + self.screen.row_offset]
                                        .len();
                                    mode = Mode::Insert;
                                    line_to_rewrite = Some(self.screen.cursor.y);
                                }
                                ':' => {
                                    mode = Mode::Command;
                                }
                                _ => (),
                            },
                            Key::Ctrl(ch) => match ch {
                                'l' => flag_rewrite_all = true,
                                _ => (),
                            },
                            _ => (),
                        },
                        Event::Mouse(me) => match me {
                            MouseEvent::Press(_, x, y) => {
                                self.screen.cursor = Cursor {
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
                            self.text[self.screen.cursor.y + self.screen.row_offset]
                                .insert(self.screen.cursor.x, ch);
                            self.screen.cursor.x += 1;
                            line_to_rewrite = Some(self.screen.cursor.y);
                        }
                        Event::Key(Key::Backspace) if self.screen.cursor.x >= 1 => {
                            self.text[self.screen.cursor.y + self.screen.row_offset]
                                .remove(self.screen.cursor.x - 1);
                            self.screen.cursor.x -= 1;
                            line_to_rewrite = Some(self.screen.cursor.y);
                        }
                        _ => (),
                    },
                    Mode::Command => match evt {
                        Event::Key(Key::Esc) => {
                            mode = Mode::Normal;
                        }
                        _ => (),
                    },
                }
                if flag_rewrite_all {
                    self.text
                        .rewrite_entire_screen(&mut self.io.stdout, self.screen.row_offset);
                }
                match line_to_rewrite {
                    Some(line) => self.text.rewrite_single_line(
                        &mut self.io.stdout,
                        line,
                        self.screen.row_offset,
                    ),
                    None => (),
                }
                debug_print(
                    &mut self.io.stdout,
                    &mode,
                    vec![
                        mode.to_string().to_uppercase(),
                        (self.screen.cursor.y + self.screen.row_offset + 1).to_string(),
                        (self.screen.cursor.x + 1).to_string(),
                    ],
                );
                write!(
                    self.io.stdout,
                    "{}",
                    termion::cursor::Goto(
                        self.screen.cursor.x as u16 + 1,
                        self.screen.cursor.y as u16 + 1
                    )
                )
                .unwrap();
                self.io.stdout.flush().unwrap();
            }
        }
    }
}
