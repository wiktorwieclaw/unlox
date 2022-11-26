pub struct Cursor<'a> {
    source: &'a str,
    start: usize,
    current: usize,
    line: u32,
}

impl<'a> Cursor<'a> {
    pub fn new(source: &'a str) -> Self {
        Cursor {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn reset_position(&mut self) {
        self.start = self.current
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    pub fn advance(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.current += 1;
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }

    pub fn advance_while(&mut self, pred: impl Fn(char) -> bool) {
        loop {
            match self.peek() {
                Some(c) if pred(c) => self.advance(),
                _ => break,
            };
        }
    }

    pub fn match_current(&mut self, expected: char) -> bool {
        match self.peek() {
            Some(c) if c == expected => {
                self.current += 1;
                true
            }
            _ => false,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.source[self.current..].chars().next()
    }

    pub fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            return None;
        }
        self.source[(self.current + 1)..].chars().next()
    }

    pub fn current_str(&mut self) -> &str {
        &self.source[self.start..self.current]
    }
}
