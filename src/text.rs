use std::cmp;
use std::io::{BufRead, Error, Write};
pub type TextState = Vec<Vec<char>>;

pub trait UpdateScreen<W>
where
    W: Write,
{
    fn rewrite_entire_screen(&self, stdout: W, row_offset: usize) -> ();
    fn rewrite_single_line(&self, stdout: W, line_to_rewrite: usize, row_offset: usize) -> ();
}

impl<W> UpdateScreen<W> for TextState
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
