use lazy_static::lazy_static;
use regex::Regex;

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
        } else if self.input.starts_with("{{") {
            self.input = &self.input[2..];
            Token::DoubleOpenBrace
        } else if self.input.starts_with("}}") {
            self.input = &self.input[2..];
            Token::DoubleCloseBrace
        } else if self.input.starts_with("=") {
            let mut length = 1;
            self.input = &self.input[1..];
            while self.input.starts_with("=") {
                length += 1;
                self.input = &self.input[1..];
            }
            Token::MultiEquals(length)
        } else {
            if let Some(regex_match) = TEXT_REGEX.find(self.input) {
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
