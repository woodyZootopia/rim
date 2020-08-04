use crate::screen::*;
use crate::text::*;
use crate::util::*;
use std::fs;
use std::io::{BufRead, Error, Write};
use termion::event::{Event, Key, MouseEvent};
use termion::input::TermRead;
pub struct Editor<R, W>
where
    R: BufRead,
    W: Write,
{
    filepath: String,
    buffer: Buffer,
    io: IO<R, W>,
}

pub struct Buffer {
    screen: ScreenState,
    text: TextState,
}
impl<R, W> Editor<R, W>
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
        let text: TextState = text.lines().map(|x| x.chars().collect()).collect();
        let stdin = reader;
        let mut stdout = writer;
        write!(stdout, "{}", termion::clear::All).unwrap();

        let buffer = Buffer {
            screen: ScreenState {
                terminal_size: termion::terminal_size().unwrap(),
                ..Default::default()
            },
            text,
        };

        Editor {
            filepath: config.filepath,
            buffer,
            io: IO { stdin, stdout },
        }
    }

    pub fn editor_loop(mut self) -> () {
        self.buffer
            .text
            .rewrite_entire_screen(&mut self.io.stdout, 0);
        let mut mode = Mode::Normal;
        for c in self.io.stdin.events() {
            let evt = c.unwrap();
            let mut line_to_rewrite: Option<usize> = None;
            let mut rewrite_all_lines = false;
            let mut error_message: Option<String> = None;
            mode = match mode {
                Mode::Normal => match evt {
                    Event::Key(key) => match key {
                        Key::Char(ch) => match ch {
                            'q' => break,
                            'h' => {
                                self.buffer.screen.move_horiz(&self.buffer.text, -1);
                                Mode::Normal
                            }
                            'j' => {
                                if self.buffer.screen.cursor.y + self.buffer.screen.row_offset + 1
                                    < self.buffer.text.len()
                                {
                                    if let Some(line) =
                                        self.buffer.screen.move_vert(&self.buffer.text, 1)
                                    {
                                        line_to_rewrite = Some(line);
                                        write!(self.io.stdout, "{}", termion::scroll::Up(1))
                                            .unwrap();
                                    }
                                }
                                Mode::Normal
                            }
                            'k' => {
                                if self.buffer.screen.cursor.y + self.buffer.screen.row_offset >= 1
                                {
                                    if let Some(line) =
                                        self.buffer.screen.move_vert(&self.buffer.text, -1)
                                    {
                                        line_to_rewrite = Some(line);
                                        write!(self.io.stdout, "{}", termion::scroll::Down(1))
                                            .unwrap();
                                    };
                                }
                                Mode::Normal
                            }
                            'l' => {
                                self.buffer.screen.move_horiz(&self.buffer.text, 1);
                                Mode::Normal
                            }
                            '0' => {
                                self.buffer.screen.cursor.x = 0;
                                Mode::Normal
                            }
                            '$' => {
                                self.buffer.screen.cursor.x = self.buffer.text
                                    [self.buffer.screen.cursor.y + self.buffer.screen.row_offset]
                                    .len()
                                    - 2;
                                Mode::Normal
                            }
                            'w' => {
                                let mut has_seen_space = false;
                                loop {
                                    if self.buffer.screen.cursor.x + 1
                                        >= self.buffer.text[self.buffer.screen.cursor.y
                                            + self.buffer.screen.row_offset]
                                            .len()
                                    {
                                        break;
                                    }
                                    match self.buffer.text[self.buffer.screen.cursor.y
                                        + self.buffer.screen.row_offset]
                                        [self.buffer.screen.cursor.x]
                                    {
                                        'a'..='z' => {
                                            if has_seen_space {
                                                break;
                                            }
                                            self.buffer.screen.cursor.x += 1;
                                        }
                                        _ => break,
                                    }
                                }
                                Mode::Normal
                            }
                            'x' => {
                                if self.buffer.text
                                    [self.buffer.screen.cursor.y + self.buffer.screen.row_offset]
                                    .len()
                                    >= 1
                                {
                                    self.buffer.text[self.buffer.screen.cursor.y
                                        + self.buffer.screen.row_offset]
                                        .remove(self.buffer.screen.cursor.x);
                                    if self.buffer.screen.cursor.x
                                        >= self.buffer.text[self.buffer.screen.cursor.y
                                            + self.buffer.screen.row_offset]
                                            .len()
                                        && self.buffer.screen.cursor.x > 0
                                    {
                                        self.buffer.screen.cursor.x -= 1;
                                    }
                                }
                                line_to_rewrite = Some(self.buffer.screen.cursor.y);
                                Mode::Normal
                            }
                            'i' => Mode::Insert,
                            'a' => {
                                self.buffer.screen.move_horiz(&self.buffer.text, 1);
                                line_to_rewrite = Some(self.buffer.screen.cursor.y);
                                Mode::Insert
                            }
                            'o' => {
                                self.buffer.text.insert(
                                    self.buffer.screen.cursor.y + self.buffer.screen.row_offset + 1,
                                    Vec::new(),
                                );
                                self.buffer.screen.move_vert(&self.buffer.text, 1);
                                rewrite_all_lines = true;
                                Mode::Insert
                            }
                            'I' => {
                                self.buffer.screen.cursor.x = 0;
                                line_to_rewrite = Some(self.buffer.screen.cursor.y);
                                Mode::Insert
                            }
                            'A' => {
                                self.buffer.screen.cursor.x = self.buffer.text
                                    [self.buffer.screen.cursor.y + self.buffer.screen.row_offset]
                                    .len();
                                line_to_rewrite = Some(self.buffer.screen.cursor.y);
                                Mode::Insert
                            }
                            ':' => Mode::Command(String::new()),
                            _ => Mode::Normal,
                        },
                        Key::Ctrl(ch) => match ch {
                            'l' => {
                                rewrite_all_lines = true;
                                Mode::Normal
                            }
                            _ => Mode::Normal,
                        },
                        _ => Mode::Normal,
                    },
                    Event::Mouse(me) => {
                        if let MouseEvent::Press(_, x, y) = me {
                            self.buffer.screen.cursor = Cursor {
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
                            self.buffer.screen.move_horiz(&self.buffer.text, 0);
                            Mode::Normal
                        }
                        Key::Char('\n') => {
                            let right_of_cursor_text = self.buffer.text
                                [self.buffer.screen.cursor.y + self.buffer.screen.row_offset]
                                .split_off(self.buffer.screen.cursor.x);
                            self.buffer.text.insert(
                                self.buffer.screen.cursor.y + self.buffer.screen.row_offset + 1,
                                right_of_cursor_text,
                            );
                            self.buffer.screen.move_vert(&self.buffer.text, 1);
                            self.buffer.screen.cursor.x = 0;
                            rewrite_all_lines = true;
                            Mode::Insert
                        }
                        Key::Char(ch) => {
                            self.buffer.text
                                [self.buffer.screen.cursor.y + self.buffer.screen.row_offset]
                                .insert(self.buffer.screen.cursor.x, ch);
                            self.buffer.screen.cursor.x += 1;
                            line_to_rewrite = Some(self.buffer.screen.cursor.y);
                            Mode::Insert
                        }
                        Key::Ctrl(ch) => match ch {
                            'u' => {
                                self.buffer.text
                                    [self.buffer.screen.cursor.y + self.buffer.screen.row_offset] =
                                    self.buffer.text[self.buffer.screen.cursor.y
                                        + self.buffer.screen.row_offset]
                                        .split_off(self.buffer.screen.cursor.x);
                                self.buffer.screen.cursor.x = 0;
                                line_to_rewrite = Some(self.buffer.screen.cursor.y);
                                Mode::Insert
                            }
                            'a' => {
                                self.buffer.screen.cursor.x = 0;
                                Mode::Insert
                            }
                            'e' => {
                                self.buffer.screen.cursor.x = self.buffer.text
                                    [self.buffer.screen.cursor.y + self.buffer.screen.row_offset]
                                    .len();
                                Mode::Insert
                            }
                            'h' if self.buffer.screen.cursor.x >= 1 => {
                                self.buffer.text
                                    [self.buffer.screen.cursor.y + self.buffer.screen.row_offset]
                                    .remove(self.buffer.screen.cursor.x - 1);
                                self.buffer.screen.cursor.x -= 1;
                                line_to_rewrite = Some(self.buffer.screen.cursor.y);
                                Mode::Insert
                            }
                            _ => Mode::Insert,
                        },
                        Key::Backspace if self.buffer.screen.cursor.x >= 1 => {
                            self.buffer.text
                                [self.buffer.screen.cursor.y + self.buffer.screen.row_offset]
                                .remove(self.buffer.screen.cursor.x - 1);
                            self.buffer.screen.cursor.x -= 1;
                            line_to_rewrite = Some(self.buffer.screen.cursor.y);
                            Mode::Insert
                        }
                        _ => Mode::Insert,
                    },
                    Event::Mouse(me) => {
                        if let MouseEvent::Press(_, x, y) = me {
                            self.buffer.screen.cursor = Cursor {
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
                                match save_to_file(&self.filepath, &self.buffer.text) {
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
            if rewrite_all_lines {
                self.buffer
                    .text
                    .rewrite_entire_screen(&mut self.io.stdout, self.buffer.screen.row_offset);
            }
            match line_to_rewrite {
                Some(line) => self.buffer.text.rewrite_single_line(
                    &mut self.io.stdout,
                    line,
                    self.buffer.screen.row_offset,
                ),
                None => (),
            }
            match error_message {
                None => print_status(
                    &mut self.io.stdout,
                    &mode,
                    vec![
                        mode.to_string(),
                        (self.buffer.screen.cursor.y + self.buffer.screen.row_offset + 1)
                            .to_string(),
                        (self.buffer.screen.cursor.x + 1).to_string(),
                    ],
                ),
                Some(message) => print_status(&mut self.io.stdout, &mode, vec![message]),
            }
            write!(
                self.io.stdout,
                "{}",
                termion::cursor::Goto(
                    self.buffer.screen.cursor.x as u16 + 1,
                    self.buffer.screen.cursor.y as u16 + 1
                )
            )
            .unwrap();
            self.io.stdout.flush().unwrap();
        }
    }
}
