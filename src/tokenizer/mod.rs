use crate::error::ParserErrorKind;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Display;

lazy_static! {
    static ref TEXT_REGEX: Regex = Regex::new(r"(\{\{|\}\}|=|\|)").unwrap();
}

pub const MAX_SECTION_DEPTH: usize = 6;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token<'a> {
    Text(&'a str),
    MultiEquals(u8),
    DoubleOpenBrace,
    DoubleCloseBrace,
    VerticalBar,
    Eof,
}

/// A position in a text.
#[derive(Clone, Copy, Debug)]
pub struct TextPosition {
    /// One-based line number.
    pub line: usize,
    /// One-based column number.
    pub column: usize,
}

impl Default for TextPosition {
    fn default() -> Self {
        Self { line: 1, column: 1 }
    }
}

#[derive(Clone, Debug)]
pub struct PositionAwareStrIterator<'input> {
    input: &'input str,
    position: TextPosition,
}

impl<'input> PositionAwareStrIterator<'input> {
    pub fn new<'input_argument: 'input>(input: &'input_argument str) -> Self {
        Self {
            input,
            position: Default::default(),
        }
    }

    pub fn remaining_input(&self) -> &'input str {
        self.input
    }

    pub fn advance_until(&mut self, limit: usize) {
        let mut cumulative_advancement = 0;
        while cumulative_advancement < limit {
            cumulative_advancement += self.advance_one();
        }
        assert_eq!(cumulative_advancement, limit);
    }

    pub fn advance_one(&mut self) -> usize {
        assert!(!self.input.is_empty());
        if self.input.starts_with('\n') {
            self.position.line += 1;
            self.position.column = 1;
        } else {
            self.position.column += 1;
        }

        if let Some((offset, _)) = self.input.char_indices().nth(1) {
            self.input = &self.input[offset..];
            offset
        } else {
            let offset = self.input.len();
            self.input = &self.input[offset..];
            offset
        }
    }
}

pub struct Tokenizer<'input> {
    input: PositionAwareStrIterator<'input>,
}

impl<'input> Tokenizer<'input> {
    pub fn new<'input_argument: 'input>(input: &'input_argument str) -> Self {
        Self {
            input: PositionAwareStrIterator::new(input),
        }
    }

    #[allow(unused)]
    pub fn tokenize_all(&mut self) -> Vec<Token<'input>> {
        let mut tokens = Vec::new();
        while tokens.last() != Some(&Token::Eof) {
            tokens.push(self.next());
        }
        tokens
    }

    pub fn next<'token, 'this>(&'this mut self) -> Token<'token>
    where
        'input: 'token + 'this,
    {
        let input = self.input.remaining_input();
        if input.is_empty() {
            Token::Eof
        } else if input.starts_with(r"{{") {
            self.input.advance_until(2);
            Token::DoubleOpenBrace
        } else if input.starts_with(r"}}") {
            self.input.advance_until(2);
            Token::DoubleCloseBrace
        } else if input.starts_with('=') {
            let mut length = 1u8;
            self.input.advance_one();
            while self.input.remaining_input().starts_with('=')
                && usize::from(length) < MAX_SECTION_DEPTH
            {
                length += 1;
                self.input.advance_one();
            }
            Token::MultiEquals(length)
        } else if input.starts_with('|') {
            self.input.advance_one();
            Token::VerticalBar
        } else if let Some(regex_match) = TEXT_REGEX.find(input) {
            let result = Token::Text(&input[..regex_match.start()]);
            self.input.advance_until(regex_match.start());
            result
        } else {
            let result = Token::Text(self.input.remaining_input());
            self.input.advance_until(input.len());
            result
        }
    }
}

pub struct MultipeekTokenizer<'tokenizer> {
    tokenizer: Tokenizer<'tokenizer>,
    peek: VecDeque<Token<'tokenizer>>,
}

impl<'tokenizer> MultipeekTokenizer<'tokenizer> {
    pub fn new(tokenizer: Tokenizer<'tokenizer>) -> Self {
        Self {
            tokenizer,
            peek: VecDeque::new(),
        }
    }

    pub fn next<'token>(&mut self) -> Token<'token>
    where
        'tokenizer: 'token,
    {
        if let Some(token) = self.peek.pop_front() {
            token
        } else {
            self.tokenizer.next()
        }
    }

    pub fn peek(&mut self, distance: usize) -> &Token {
        while self.peek.len() < distance + 1 {
            self.peek.push_back(self.tokenizer.next());
        }
        &self.peek[distance]
    }

    pub fn repeek(&self, distance: usize) -> Option<&Token> {
        self.peek.get(distance)
    }

    pub fn text_position(&self) -> TextPosition {
        self.tokenizer.input.position
    }

    pub fn expect(&mut self, token: &Token) -> crate::error::Result<()> {
        let next = self.next();
        if &next == token {
            Ok(())
        } else {
            Err(ParserErrorKind::UnexpectedToken {
                expected: token.to_string(),
                actual: next.to_string(),
            }
            .into_parser_error(self.text_position()))
        }
    }
}

impl<'token> Display for Token<'token> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(fmt, "{}", self.to_str())
    }
}

impl Token<'_> {
    pub fn to_str(&self) -> &str {
        match self {
            Token::Text(text) => text,
            Token::MultiEquals(amount) => {
                let buffer = "======";
                assert_eq!(buffer.len(), MAX_SECTION_DEPTH);
                &buffer[..usize::from(*amount)]
            }
            Token::DoubleOpenBrace => r"{{",
            Token::DoubleCloseBrace => r"}}",
            Token::VerticalBar => "|",
            Token::Eof => unreachable!("EOF has no string representation"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokenizer::{Token, Tokenizer};

    #[test]
    fn simple() {
        let input = "{{==a=  v}} }} } edf } } [ {";
        let mut tokenizer = Tokenizer::new(input);
        let tokens = tokenizer.tokenize_all();
        assert_eq!(
            tokens.as_slice(),
            [
                Token::DoubleOpenBrace,
                Token::MultiEquals(2),
                Token::Text("a"),
                Token::MultiEquals(1),
                Token::Text("  v"),
                Token::DoubleCloseBrace,
                Token::Text(" "),
                Token::DoubleCloseBrace,
                Token::Text(" } edf } } [ {"),
                Token::Eof,
            ]
        );
    }
}
