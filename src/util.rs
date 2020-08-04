use crate::text::*;
use std::io::{BufRead, Error, Write};
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

pub fn print_status<W>(mut stdout: W, mode: &Mode, args: Vec<String>)
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

pub struct IO<R, W>
where
    R: BufRead,
    W: Write,
{
    pub stdin: R,
    pub stdout: W,
}

pub enum Mode {
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

pub fn save_to_file(filepath: &String, contents: &TextState) -> std::result::Result<(), Error> {
    let contents = contents
        .iter()
        .map(|x| x.iter().collect::<String>())
        .collect::<Vec<String>>()
        .join("\n");
    std::fs::write(filepath, contents)?;
    return Ok(());
}

