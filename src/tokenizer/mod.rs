use crate::MAX_SECTION_DEPTH;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::VecDeque;

lazy_static! {
    static ref TEXT_REGEX: Regex = Regex::new(r"(\{\{|\}\}|=)").unwrap();
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Token<'a> {
    Text(&'a str),
    MultiEquals(u8),
    DoubleOpenBrace,
    DoubleCloseBrace,
    Eof,
}

pub struct Tokenizer<'input> {
    input: &'input str,
}

impl<'input> Tokenizer<'input> {
    pub fn new<'input_argument: 'input>(input: &'input_argument str) -> Self {
        Self { input }
    }

    #[allow(unused)]
    pub fn tokenize_all(&mut self) -> Vec<Token<'input>> {
        let mut tokens = Vec::new();
        while tokens.last() != Some(&Token::Eof) {
            tokens.push(self.next());
        }
        tokens
    }

    pub fn next<'token>(&mut self) -> Token<'token>
    where
        'input: 'token,
    {
        if self.input.is_empty() {
            Token::Eof
        } else if self.input.starts_with(r"{{") {
            self.input = &self.input[2..];
            Token::DoubleOpenBrace
        } else if self.input.starts_with(r"}}") {
            self.input = &self.input[2..];
            Token::DoubleCloseBrace
        } else if self.input.starts_with('=') {
            let mut length = 1u8;
            self.input = &self.input[1..];
            while self.input.starts_with('=') && usize::from(length) < MAX_SECTION_DEPTH {
                length += 1;
                self.input = &self.input[1..];
            }
            Token::MultiEquals(length)
        } else if let Some(regex_match) = TEXT_REGEX.find(self.input) {
            let result = Token::Text(&self.input[..regex_match.start()]);
            self.input = &self.input[regex_match.start()..];
            result
        } else {
            let result = Token::Text(self.input);
            self.input = &self.input[self.input.len()..];
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
}

impl<'token> ToString for Token<'token> {
    fn to_string(&self) -> String {
        match self {
            Token::Text(text) => text.to_string(),
            Token::MultiEquals(amount) => "=".repeat(usize::from(*amount)),
            Token::DoubleOpenBrace => r"{{".to_string(),
            Token::DoubleCloseBrace => r"}}".to_string(),
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
