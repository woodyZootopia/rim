use std::cmp;
use std::fs;
use std::io::{BufRead, Error, Write};
use termion;
use termion::event::{Event, Key, MouseEvent};
use termion::input::TermRead;

pub struct Config {
    pub filepath: String,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();
        let filepath = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a filename"),
        };
        Ok(Config { filepath })
    }
}

fn debug_print<W>(mut stdout: W, mode: &Mode, args: Vec<String>)
where
    W: Write,
{
    write!(
        stdout,
        "{}{}{}{}",
        termion::cursor::Goto(0, termion::terminal_size().unwrap().1),
        termion::clear::CurrentLine,
        match mode {
            Mode::Normal => termion::color::Bg(termion::color::Rgb(145, 172, 209)),
            Mode::Insert => termion::color::Bg(termion::color::Rgb(192, 202, 142)),
            Mode::Command(_) => termion::color::Bg(termion::color::Rgb(233, 144, 144)),
        },
        termion::color::Fg(termion::color::Black),
    )
    .unwrap();
    write!(stdout, "{}", args.join(", ")).unwrap();
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

trait UpdateScreen<W>
where
    W: Write,
{
    fn rewrite_entire_screen(&self, stdout: W, row_offset: usize) -> ();
    fn rewrite_single_line(&self, stdout: W, line_to_rewrite: usize, row_offset: usize) -> ();
}

impl<W> UpdateScreen<W> for Text
where
    W: Write,
{
    fn rewrite_entire_screen(&self, mut stdout: W, row_offset: usize) {
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
    fn rewrite_single_line(&self, mut stdout: W, line_to_rewrite: usize, row_offset: usize) {
        let column = self.iter().collect::<Vec<&Vec<char>>>()[line_to_rewrite + row_offset];
        write!(
            stdout,
            "{}{}",
            termion::cursor::Goto(0, line_to_rewrite as u16 + 1),
            termion::clear::CurrentLine
        )
        .unwrap();
        write!(stdout, "{}", column.iter().collect::<String>()).unwrap();
        return;
    }
}

pub struct ScreenState {
    cursor: Cursor,
    row_offset: usize,
}

impl ScreenState {
    /// Move key vertically. After that, make sure key is in valid place.
    /// Returns the line to rewrite.
    fn move_vert(&mut self, text: &Text, movement: i32) -> Option<usize> {
        let mut line_to_rewrite = None;
        if movement > 0 {
            if self.cursor.y as i32 + movement > termion::terminal_size().unwrap().1 as i32 - 2 {
                self.cursor.y -= movement as usize;
                self.row_offset += movement as usize;
                line_to_rewrite = Some(self.cursor.y + movement as usize);
            }
            self.cursor.y += movement as usize;
        }
        if movement < 0 {
            let distance = -movement as usize;
            if self.cursor.y as i32 + movement < 0 {
                if self.row_offset > 0 {
                    self.row_offset -= distance;
                }
                self.cursor.y += distance;
                line_to_rewrite = Some(self.cursor.y - distance);
            }
            self.cursor.y -= -movement as usize;
        }

        if self.cursor.x > text[self.cursor.y + self.row_offset].len() {
            self.cursor.x = cmp::max(text[self.cursor.y + self.row_offset].len(), 1) - 1;
        }
        return line_to_rewrite;
    }
    /// Move key horizontally. After that, make sure key is in valid place.
    pub fn move_horiz(&mut self, text: &Text, distance: i32){
        if distance < 0 {
            let distance = -distance as usize;
            self.cursor.x -= cmp::min(self.cursor.x, distance);
        } else {
            let distance = distance as usize;
            self.cursor.x = cmp::min(
                self.cursor.x + distance,
                text[self.cursor.y + self.row_offset].len() - 1,
            );
        }
    }
}

pub struct IO<R, W>
where
    R: BufRead,
    W: Write,
{
    stdin: R,
    stdout: W,
}

enum Mode {
    Normal,
    Insert,
    Command(String),
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Command(command) => write!(f, "COMMAND:{} ", command),
        }
    }
}

pub struct EditorState<R, W>
where
    R: BufRead,
    W: Write,
{
    filepath: String,
    screen: ScreenState,
    io: IO<R, W>,
    text: Text,
}

impl<R, W> EditorState<R, W>
where
    R: BufRead,
    W: Write,
{
    pub fn new(reader: R, writer: W, config: Config) -> Self
    where
        R: BufRead,
        W: Write,
    {
        let text = fs::read_to_string(&config.filepath).unwrap();
        let text: Vec<Vec<char>> = text.lines().map(|x| x.chars().collect()).collect();
        let stdin = reader;
        let mut stdout = writer;

        write!(stdout, "{}", termion::clear::All).unwrap();

        EditorState {
            filepath: config.filepath,
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
            let mut error_message: Option<String> = None;
            mode = match mode {
                Mode::Normal => match evt {
                    Event::Key(key) => match key {
                        Key::Char(ch) => match ch {
                            'q' => break,
                            'h' => {
                                self.screen.move_horiz(&self.text, -1);
                                Mode::Normal
                            }
                            'j' => {
                                if self.screen.cursor.y + self.screen.row_offset + 1
                                    < self.text.len()
                                {
                                    if let Some(line) = self.screen.move_vert(&self.text, 1) {
                                        line_to_rewrite = Some(line);
                                        write!(self.io.stdout, "{}", termion::scroll::Up(1))
                                            .unwrap();
                                    }
                                }
                                Mode::Normal
                            }
                            'k' => {
                                if self.screen.cursor.y + self.screen.row_offset >= 1 {
                                    if let Some(line) = self.screen.move_vert(&self.text, -1) {
                                        line_to_rewrite = Some(line);
                                        write!(self.io.stdout, "{}", termion::scroll::Down(1))
                                            .unwrap();
                                    };
                                }
                                Mode::Normal
                            }
                            'l' => {
                                self.screen.move_horiz(&self.text, 1);
                                Mode::Normal
                            }
                            '0' => {
                                self.screen.cursor.x = 0;
                                Mode::Normal
                            }
                            '$' => {
                                self.screen.cursor.x =
                                    self.text[self.screen.cursor.y + self.screen.row_offset].len()
                                        - 1;
                                Mode::Normal
                            }
                            'w' => {
                                let mut has_seen_space = false;
                                loop {
                                    if self.screen.cursor.x + 1
                                        >= self.text[self.screen.cursor.y + self.screen.row_offset]
                                            .len()
                                    {
                                        break;
                                    }
                                    match self.text[self.screen.cursor.y + self.screen.row_offset]
                                        [self.screen.cursor.x]
                                    {
                                        'a'..='z' => {
                                            if has_seen_space {
                                                break;
                                            }
                                            self.screen.cursor.x += 1;
                                        }
                                        _ => break,
                                    }
                                }
                                Mode::Normal
                            }
                            'x' => {
                                if self.text[self.screen.cursor.y + self.screen.row_offset].len()
                                    >= 1
                                {
                                    self.text[self.screen.cursor.y + self.screen.row_offset]
                                        .remove(self.screen.cursor.x);
                                    if self.screen.cursor.x
                                        >= self.text[self.screen.cursor.y + self.screen.row_offset]
                                            .len()
                                        && self.screen.cursor.x > 0
                                    {
                                        self.screen.cursor.x -= 1;
                                    }
                                }
                                line_to_rewrite = Some(self.screen.cursor.y);
                                Mode::Normal
                            }
                            'i' => Mode::Insert,
                            'a' => {
                                self.screen.cursor.x += 1;
                                line_to_rewrite = Some(self.screen.cursor.y);
                                Mode::Insert
                            }
                            'o' => {
                                self.text.insert(
                                    self.screen.cursor.y + self.screen.row_offset + 1,
                                    Vec::new(),
                                );
                                self.screen.move_vert(&self.text, 1);
                                flag_rewrite_all = true;
                                Mode::Insert
                            }
                            'I' => {
                                self.screen.cursor.x = 0;
                                line_to_rewrite = Some(self.screen.cursor.y);
                                Mode::Insert
                            }
                            'A' => {
                                self.screen.cursor.x =
                                    self.text[self.screen.cursor.y + self.screen.row_offset].len();
                                line_to_rewrite = Some(self.screen.cursor.y);
                                Mode::Insert
                            }
                            ':' => Mode::Command(String::new()),
                            _ => Mode::Normal,
                        },
                        Key::Ctrl(ch) => match ch {
                            'l' => {
                                flag_rewrite_all = true;
                                Mode::Normal
                            }
                            _ => Mode::Normal,
                        },
                        _ => Mode::Normal,
                    },
                    Event::Mouse(me) => {
                        if let MouseEvent::Press(_, x, y) = me {
                            self.screen.cursor = Cursor {
                                x: x as usize - 1,
                                y: y as usize - 1,
                            };
                        };
                        Mode::Normal
                    }
                    _ => Mode::Normal,
                },
                Mode::Insert => match evt {
                    Event::Key(key) => match key {
                        Key::Esc => {
                            self.screen.move_horiz(&self.text, 0);
                            Mode::Normal
                        }
                        Key::Char('\n') => {
                            let right_of_cursor_text = self.text
                                [self.screen.cursor.y + self.screen.row_offset]
                                .split_off(self.screen.cursor.x);
                            self.text.insert(
                                self.screen.cursor.y + self.screen.row_offset + 1,
                                right_of_cursor_text,
                            );
                            self.screen.move_vert(&self.text, 1);
                            self.screen.cursor.x = 0;
                            flag_rewrite_all = true;
                            Mode::Insert
                        }
                        Key::Char(ch) => {
                            self.text[self.screen.cursor.y + self.screen.row_offset]
                                .insert(self.screen.cursor.x, ch);
                            self.screen.cursor.x += 1;
                            line_to_rewrite = Some(self.screen.cursor.y);
                            Mode::Insert
                        }
                        Key::Ctrl(ch) => match ch {
                            'u' => {
                                self.text[self.screen.cursor.y + self.screen.row_offset] = self
                                    .text[self.screen.cursor.y + self.screen.row_offset]
                                    .split_off(self.screen.cursor.x);
                                self.screen.cursor.x = 0;
                                line_to_rewrite = Some(self.screen.cursor.y);
                                Mode::Insert
                            }
                            'a' => {
                                self.screen.cursor.x = 0;
                                Mode::Insert
                            }
                            'e' => {
                                self.screen.cursor.x =
                                    self.text[self.screen.cursor.y + self.screen.row_offset].len();
                                Mode::Insert
                            }
                            'h' if self.screen.cursor.x >= 1 => {
                                self.text[self.screen.cursor.y + self.screen.row_offset]
                                    .remove(self.screen.cursor.x - 1);
                                self.screen.cursor.x -= 1;
                                line_to_rewrite = Some(self.screen.cursor.y);
                                Mode::Insert
                            }
                            _ => Mode::Insert,
                        },
                        Key::Backspace if self.screen.cursor.x >= 1 => {
                            self.text[self.screen.cursor.y + self.screen.row_offset]
                                .remove(self.screen.cursor.x - 1);
                            self.screen.cursor.x -= 1;
                            line_to_rewrite = Some(self.screen.cursor.y);
                            Mode::Insert
                        }
                        _ => Mode::Insert,
                    },
                    Event::Mouse(me) => {
                        if let MouseEvent::Press(_, x, y) = me {
                            self.screen.cursor = Cursor {
                                x: x as usize - 1,
                                y: y as usize - 1,
                            };
                        }
                        Mode::Insert
                    }
                    _ => Mode::Insert,
                },
                Mode::Command(mut command_buffer) => match evt {
                    Event::Key(key) => match key {
                        Key::Esc => Mode::Normal,
                        Key::Char('\n') => match command_buffer.as_str() {
                            "q" => break,
                            "w" => {
                                match save_to_file(&self.filepath, &self.text) {
                                    Ok(_) => error_message = Some("Save complete".to_string()),
                                    Err(why) => {
                                        error_message = Some(
                                            ["Save failed! reason", why.to_string().as_str()]
                                                .join(": "),
                                        )
                                    }
                                }
                                Mode::Normal
                            }
                            "" => Mode::Normal,
                            _ => {
                                error_message = Some("Sorry, that command is not implemented for now! Go back to Normal mode with C-c.".to_string());
                                Mode::Command(String::new())
                            }
                        },
                        Key::Char(key) => {
                            command_buffer.push(key);
                            Mode::Command(command_buffer)
                        }
                        Key::Backspace => {
                            if command_buffer.len() > 0 {
                                command_buffer.pop();
                                Mode::Command(command_buffer)
                            } else {
                                Mode::Normal
                            }
                        }
                        Key::Ctrl(key) => match key {
                            'c' => Mode::Normal,
                            _ => Mode::Command(command_buffer),
                        },
                        _ => Mode::Command(command_buffer),
                    },
                    _ => Mode::Command(command_buffer),
                },
            };
            if flag_rewrite_all {
                self.text
                    .rewrite_entire_screen(&mut self.io.stdout, self.screen.row_offset);
            }
            match line_to_rewrite {
                Some(line) => {
                    self.text
                        .rewrite_single_line(&mut self.io.stdout, line, self.screen.row_offset)
                }
                None => (),
            }
            match error_message {
                None => debug_print(
                    &mut self.io.stdout,
                    &mode,
                    vec![
                        mode.to_string(),
                        (self.screen.cursor.y + self.screen.row_offset + 1).to_string(),
                        (self.screen.cursor.x + 1).to_string(),
                    ],
                ),
                Some(message) => debug_print(&mut self.io.stdout, &mode, vec![message]),
            }
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
fn save_to_file(filepath: &String, contents: &Text) -> std::result::Result<(), Error> {
    let contents = contents
        .iter()
        .map(|x| x.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");
    std::fs::write(filepath, contents)?;
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{stdin, stdout};
    use termion::input::MouseTerminal;
    use termion::raw::IntoRawMode;
    use termion::screen::AlternateScreen;
    #[test]
    fn test_basic_cursor_move() {
        let config = Config {
            filepath: "./test.txt".to_string(),
        };
        let stdio = stdin();
        let input = stdio.lock();
        let output = stdout();
        let editor = EditorState::new(input, output, config);
    }
}
