use crate::core::syntax::CharCodes;

/// Position information in the byte stream.
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

/// A cursor over a byte slice for PDF parsing.
pub struct ByteStream<'a> {
    bytes: &'a [u8],
    idx: usize,
    line: usize,
    column: usize,
}

impl<'a> ByteStream<'a> {
    pub fn of(bytes: &'a [u8]) -> Self {
        ByteStream {
            bytes,
            idx: 0,
            line: 0,
            column: 0,
        }
    }

    pub fn move_to(&mut self, offset: usize) {
        self.idx = offset;
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<u8> {
        if self.idx >= self.bytes.len() {
            return None;
        }
        let byte = self.bytes[self.idx];
        self.idx += 1;
        if byte == CharCodes::Newline {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
        Some(byte)
    }

    pub fn peek(&self) -> Option<u8> {
        self.bytes.get(self.idx).copied()
    }

    pub fn peek_ahead(&self, steps: usize) -> Option<u8> {
        self.bytes.get(self.idx + steps).copied()
    }

    pub fn done(&self) -> bool {
        self.idx >= self.bytes.len()
    }

    pub fn offset(&self) -> usize {
        self.idx
    }

    pub fn slice(&self, start: usize, end: usize) -> &'a [u8] {
        &self.bytes[start..end.min(self.bytes.len())]
    }

    pub fn position(&self) -> Position {
        Position {
            line: self.line,
            column: self.column,
            offset: self.idx,
        }
    }

    pub fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.idx..]
    }
}
