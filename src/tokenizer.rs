use crate::error::ParserErrorKind;
use lazy_static::lazy_static;
use regex::Regex;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fmt;
use std::fmt::Display;

static NOWIKI_OPEN: &str = "<nowiki>";
static NOWIKI_CLOSE: &str = "</nowiki>";

lazy_static! {
    static ref TEXT_REGEX: Regex = Regex::new(&format!(
        "(\\{{\\{{|\\}}\\}}|\\[\\[|\\]\\]|=|\\||'|\n|:|;|\\*|#|{NOWIKI_OPEN}|{NOWIKI_CLOSE})"
    ))
    .unwrap();
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token<'a> {
    Text(Cow<'a, str>),
    Equals,
    DoubleOpenBrace,
    DoubleCloseBrace,
    DoubleOpenBracket,
    DoubleCloseBracket,
    NoWikiOpen,
    NoWikiClose,
    VerticalBar,
    Apostrophe,
    Colon,
    Semicolon,
    Star,
    Sharp,
    Newline,
    Eof,
}

/// A position in a text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextPosition {
    /// One-based line number.
    pub line: usize,
    /// One-based column number.
    pub column: usize,
}

impl TextPosition {
    /// Create a new text position at the given `line` and `column`.
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
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

    /// Returns `true` if the tokenizer has not yet been advanced.
    pub fn is_at_start(&self) -> bool {
        self.position == Default::default()
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
        } else if input.starts_with("[[") {
            self.input.advance_until(2);
            Token::DoubleOpenBracket
        } else if input.starts_with("]]") {
            self.input.advance_until(2);
            Token::DoubleCloseBracket
        } else if input.starts_with(NOWIKI_OPEN) {
            self.input.advance_until(NOWIKI_OPEN.len());
            Token::NoWikiOpen
        } else if input.starts_with(NOWIKI_CLOSE) {
            self.input.advance_until(NOWIKI_CLOSE.len());
            Token::NoWikiClose
        } else if input.starts_with('=') {
            self.input.advance_one();
            Token::Equals
        } else if input.starts_with('|') {
            self.input.advance_one();
            Token::VerticalBar
        } else if input.starts_with('\'') {
            self.input.advance_one();
            Token::Apostrophe
        } else if input.starts_with("\r\n") {
            self.input.advance_until(2);
            Token::Newline
        } else if input.starts_with('\n') {
            self.input.advance_one();
            Token::Newline
        } else if input.starts_with(':') {
            self.input.advance_one();
            Token::Colon
        } else if input.starts_with(';') {
            self.input.advance_one();
            Token::Semicolon
        } else if input.starts_with('*') {
            self.input.advance_one();
            Token::Star
        } else if input.starts_with('#') {
            self.input.advance_one();
            Token::Sharp
        } else if let Some(regex_match) = TEXT_REGEX.find(input) {
            let result = Token::Text(input[..regex_match.start()].into());
            self.input.advance_until(regex_match.start());
            result
        } else {
            let result = Token::Text(self.input.remaining_input().into());
            self.input.advance_until(input.len());
            result
        }
    }

    /// Returns `true` if the tokenizer has not yet been advanced.
    #[allow(unused)]
    pub fn is_at_start(&self) -> bool {
        self.input.is_at_start()
    }
}

pub struct MultipeekTokenizer<'tokenizer> {
    tokenizer: Tokenizer<'tokenizer>,
    peek: VecDeque<(Token<'tokenizer>, TextPosition)>,
    next_was_called: bool,
}

impl<'tokenizer> MultipeekTokenizer<'tokenizer> {
    pub fn new(tokenizer: Tokenizer<'tokenizer>) -> Self {
        Self {
            tokenizer,
            peek: VecDeque::new(),
            next_was_called: false,
        }
    }

    pub fn next<'token>(&mut self) -> (Token<'token>, TextPosition)
    where
        'tokenizer: 'token,
    {
        self.next_was_called = true;
        if let Some((token, text_position)) = self.peek.pop_front() {
            (token, text_position)
        } else {
            let text_position = self.tokenizer.input.position;
            (self.tokenizer.next(), text_position)
        }
    }

    pub fn peek(&mut self, distance: usize) -> &(Token, TextPosition) {
        while self.peek.len() < distance + 1 {
            let text_position = self.tokenizer.input.position;
            self.peek.push_back((self.tokenizer.next(), text_position));
        }
        &self.peek[distance]
    }

    /// Peeks a position inside the current peek buffer.
    /// If the position and no position after it was not yet peeked, returns `None`.
    /// This is useful because it does not require a mutable reference to self.
    pub fn repeek(&self, distance: usize) -> Option<&(Token, TextPosition)> {
        self.peek.get(distance)
    }

    pub fn expect(&mut self, token: &Token) -> crate::error::Result<()> {
        let (next, text_position) = self.next();
        if &next == token {
            Ok(())
        } else {
            Err(ParserErrorKind::UnexpectedToken {
                expected: token.to_string(),
                actual: next.to_string(),
            }
            .into_parser_error(text_position))
        }
    }

    /// Returns `true` if the tokenizer has not yet been advanced.
    #[allow(unused)]
    pub fn is_at_start(&self) -> bool {
        !self.next_was_called
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
            Token::Equals => "=",
            Token::DoubleOpenBrace => "{{",
            Token::DoubleCloseBrace => "}}",
            Token::DoubleOpenBracket => "[[",
            Token::DoubleCloseBracket => "]]",
            Token::NoWikiOpen => NOWIKI_OPEN,
            Token::NoWikiClose => NOWIKI_CLOSE,
            Token::VerticalBar => "|",
            Token::Apostrophe => "'",
            Token::Newline => "\n",
            Token::Colon => ":",
            Token::Semicolon => ";",
            Token::Star => "*",
            Token::Sharp => "#",
            Token::Eof => "<EOF>",
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
                Token::Equals,
                Token::Equals,
                Token::Text("a".into()),
                Token::Equals,
                Token::Text("  v".into()),
                Token::DoubleCloseBrace,
                Token::Text(" ".into()),
                Token::DoubleCloseBrace,
                Token::Text(" } edf } } [ {".into()),
                Token::Eof,
            ]
        );
    }
}
