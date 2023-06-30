use crate::tokenizer::TextPosition;

pub type Result<T> = std::result::Result<T, ParserError>;

#[derive(Debug)]
pub struct ParserError {
    pub kind: ParserErrorKind,
    pub position: TextPosition,
}

#[derive(Debug)]
pub enum ParserErrorKind {
    /// Found a second root section, but only one is allowed.
    SecondRootSection {
        /// The label of the second root section.
        label: String,
    },

    /// Found a section at a level that is deeper than supported.
    SectionLevelTooDeep {
        /// The too deep level.
        level: usize,
    },

    /// Found a double close brace that does not match any opened one.
    UnmatchedDoubleCloseBrace,

    /// Found a double open brace that does not match any closed one.
    UnmatchedDoubleOpenBrace,

    /// A tag contains a token that does not belong there.
    UnexpectedTokenInTag { token: String },

    /// A parameter contains a token that does not belong there.
    UnexpectedTokenInParameter { token: String },

    /// A parameter contains a token that does not belong there.
    UnexpectedToken { expected: String, actual: String },
}

impl ParserError {
    pub fn new(kind: ParserErrorKind, position: TextPosition) -> Self {
        Self { kind, position }
    }
}

impl ParserErrorKind {
    pub fn into_parser_error(self, position: TextPosition) -> ParserError {
        ParserError {
            kind: self,
            position,
        }
    }
}
