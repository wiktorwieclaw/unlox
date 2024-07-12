/// Advancing text selection.
pub struct Selection<'a> {
    source: &'a str,
    start: usize,
    end: usize,
    line: u32,
}

impl<'a> Selection<'a> {
    /// Creates an empty selection at the beginning of `source` text.
    pub fn new(source: &'a str) -> Self {
        Selection {
            source,
            start: 0,
            end: 0,
            line: 1,
        }
    }

    /// Advances the end of the selection by one character.
    pub fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.end += 1;
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }

    /// Advances the end of the selection if the next character matches the `expected` character.
    pub fn advance_match(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(c) if c == expected => {
                self.end += 1;
                true
            }
            _ => false,
        }
    }

    /// Continuously advances the end of the selection while the `pred` predicate is satisfied.
    pub fn advance_while(&mut self, pred: impl Fn(char) -> bool) {
        loop {
            match self.peek() {
                Some(c) if pred(c) => self.advance(),
                _ => break,
            };
        }
    }

    /// Peek at the next character without advancing the selection.
    pub fn peek(&self) -> Option<char> {
        self.source[self.end..].chars().next()
    }

    /// Peek at the character after the next character without advancing the selection.
    pub fn peek_second(&self) -> Option<char> {
        self.source.get((self.end + 1)..)?.chars().next()
    }

    /// Clears the selection by moving it's beginning to it's end.
    pub fn clear(&mut self) {
        self.start = self.end
    }

    /// Returns a reference to the selected string.
    pub fn str(&mut self) -> &str {
        &self.source[self.start..self.end]
    }

    /// Returns the line number of the selection's end position.
    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn eof(&self) -> bool {
        self.end >= self.source.len()
    }
}
