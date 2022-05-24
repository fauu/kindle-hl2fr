use std::io::{BufRead, BufReader, Read, Result};

pub struct LineReader<R> {
    pub buf: String,
    pub current_line: i32,
    reader: BufReader<R>,
}

impl<R: Read> LineReader<R> {
    pub fn new(buf_reader: BufReader<R>) -> LineReader<R> {
        LineReader {
            reader: buf_reader,
            buf: String::new(),
            current_line: 0,
        }
    }

    pub fn read_line_to_buf(&mut self) -> Result<usize> {
        let result = self.reader.read_line(&mut self.buf);
        if result.is_ok() {
            self.current_line += 1;
        }
        result
    }
}
