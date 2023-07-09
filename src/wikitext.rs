use crate::tokenizer::Token;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Display;

/// The root of a wikitext document.
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Wikitext {
    /// The root of the section tree of the page.
    pub root_section: Section,
}

impl Wikitext {
    /// Print the headlines of the text.
    pub fn print_headlines(&self) {
        self.root_section.print_headlines();
    }

    /// List the headlines of the text.
    pub fn list_headlines(&self) -> Vec<Headline> {
        let mut result = Vec::new();
        self.root_section.list_headlines(&mut result);
        result
    }

    /// List the double brace expressions of the text.
    pub fn list_double_brace_expressions(&self) -> Vec<TextPiece> {
        let mut result = Vec::new();
        self.root_section.list_double_brace_expressions(&mut result);
        result
    }

    /// List the plain parts of the text.
    pub fn list_plain_text(&self) -> Vec<TextPiece> {
        let mut result = Vec::new();
        self.root_section.list_plain_text(&mut result);
        result
    }
}

/// A section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Section {
    /// The headline of the section.
    pub headline: Headline,
    /// The text of the section.
    pub text: Text,
    /// The subsections of the section.
    pub subsections: Vec<Section>,
}

impl Section {
    /// Print the headlines of the text.
    pub fn print_headlines(&self) {
        println!(
            "{0} {1} {0}",
            "=".repeat(self.headline.level.into()),
            self.headline.label
        );
        for subsection in &self.subsections {
            subsection.print_headlines();
        }
    }

    /// List the headlines of the text.
    pub fn list_headlines(&self, result: &mut Vec<Headline>) {
        result.push(self.headline.clone());
        for subsection in &self.subsections {
            subsection.list_headlines(result);
        }
    }

    /// List the double brace expressions of the text.
    pub fn list_double_brace_expressions(&self, result: &mut Vec<TextPiece>) {
        for text_piece in &self.text.pieces {
            if matches!(text_piece, TextPiece::DoubleBraceExpression { .. }) {
                result.push(text_piece.clone());
            }
        }
        for subsection in &self.subsections {
            subsection.list_double_brace_expressions(result);
        }
    }

    /// List the plain parts of the text.
    pub fn list_plain_text(&self, result: &mut Vec<TextPiece>) {
        for text_piece in &self.text.pieces {
            if matches!(text_piece, TextPiece::Text(_)) {
                result.push(text_piece.clone());
            }
        }
        for subsection in &self.subsections {
            subsection.list_plain_text(result);
        }
    }
}

/// A headline of a section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Headline {
    /// The label of the headline.
    pub label: String,
    /// The level of the headline.
    pub level: u8,
}

impl Headline {
    /// Create a new headline with the given label and level.
    pub fn new(label: impl Into<String>, level: u8) -> Self {
        Self {
            label: label.into(),
            level,
        }
    }
}

/// The text content of a section.
#[derive(Debug, Clone, Eq, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Text {
    /// The pieces of the text.
    pub pieces: Vec<TextPiece>,
}

impl Text {
    /// Create a new empty text.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns `true` if this `Text` contains no pieces.
    pub fn is_empty(&self) -> bool {
        self.pieces.is_empty()
    }

    /// Extend the current last text piece with the given string,
    /// or append a new text piece created from the given string if there is no text piece
    /// or the last text piece is not of variant [`Text`](TextPiece::Text) or [`FormattedText`](TextPiece::FormattedText).
    pub fn extend_with_text(&mut self, text: &str) {
        if let Some(TextPiece::Text(last)) = self.pieces.last_mut() {
            last.push_str(text);
        } else if let Some(TextPiece::FormattedText {
            text: formatted_text,
            ..
        }) = self.pieces.last_mut()
        {
            formatted_text.extend_with_text(text)
        } else {
            self.pieces.push(TextPiece::Text(text.to_string()));
        }
    }

    /// Trim whitespace from the beginning and the end of the text.
    pub fn trim_self(&mut self) {
        self.trim_self_start();
        self.trim_self_end();
    }

    /// Trim whitespace from the beginning of the text.
    pub fn trim_self_start(&mut self) {
        let mut offset = 0;
        while offset < self.pieces.len() {
            match &mut self.pieces[offset] {
                TextPiece::Text(text) => {
                    *text = text.trim_start().to_string();
                    if !text.is_empty() {
                        break;
                    }
                }
                TextPiece::DoubleBraceExpression { .. }
                | TextPiece::InternalLink { .. }
                | TextPiece::ListItem { .. } => break,
                TextPiece::FormattedText { text, .. } => {
                    text.trim_self_start();
                    if !text.is_empty() {
                        break;
                    }
                }
            }
            offset += 1;
        }
        self.pieces.drain(..offset);
    }

    /// Trim whitespace from the end of the text.
    pub fn trim_self_end(&mut self) {
        let mut limit = self.pieces.len();
        while limit > 0 {
            match &mut self.pieces[limit - 1] {
                TextPiece::Text(text) => {
                    *text = text.trim_end().to_string();
                    if !text.is_empty() {
                        break;
                    }
                }
                TextPiece::DoubleBraceExpression { .. } | TextPiece::InternalLink { .. } => break,
                TextPiece::FormattedText { text, .. } => {
                    text.trim_self_end();
                    if !text.is_empty() {
                        break;
                    }
                }
                TextPiece::ListItem { text, .. } => {
                    text.trim_self_end();
                    break;
                }
            }
            limit -= 1;
        }
        self.pieces.drain(limit..);
    }
}

/// A piece of text of a section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TextPiece {
    /// A plain string.
    Text(String),
    /// A double brace expression.
    DoubleBraceExpression {
        /// The tag of the expression.
        tag: String,
        /// The attributes of the expression.
        attributes: Vec<Attribute>,
    },
    /// An internal link.
    InternalLink {
        /// The link target.
        target: String,
        /// The link options.
        options: Vec<String>,
        /// The label of the link.
        label: Option<Text>,
    },
    /// A piece of text that is formatted in e.g. bold or italics.
    FormattedText {
        /// The formatting applied to the text.
        formatting: TextFormatting,
        /// The text.
        text: Text,
    },
    /// A list item.
    ListItem {
        /// The prefix deciding the level and numbering of the list.
        list_prefix: String,
        /// The text of the list item.
        text: Text,
    },
}

/// An attribute of e.g. a double brace expression.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Attribute {
    /// The name of the attribute.
    pub name: Option<String>,
    /// The value of the attribute.
    pub value: Text,
}

/// Format of formatted text.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[allow(missing_docs)]
pub enum TextFormatting {
    Italic,
    Bold,
    ItalicBold,
}

impl Display for Text {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        for text_piece in &self.pieces {
            write!(fmt, "{text_piece}")?;
        }
        Ok(())
    }
}

impl Display for TextPiece {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TextPiece::Text(text) => write!(fmt, "{text}"),
            TextPiece::DoubleBraceExpression {
                tag,
                attributes: parameters,
            } => {
                write!(fmt, "{{{{{tag}")?;

                for parameter in parameters {
                    write!(fmt, "|{parameter}")?;
                }

                write!(fmt, "}}}}")
            }
            TextPiece::InternalLink {
                target: url,
                options,
                label,
            } => {
                write!(fmt, "[[{url}")?;
                for option in options {
                    write!(fmt, "|{option}")?;
                }
                if let Some(label) = label {
                    write!(fmt, "|{label}")?;
                }
                write!(fmt, "]]")
            }
            TextPiece::FormattedText {
                formatting: format,
                text,
            } => {
                write!(fmt, "{}", Token::from(*format))?;
                write!(fmt, "{}", text)?;
                write!(fmt, "{}", Token::from(*format))
            }
            TextPiece::ListItem { list_prefix, text } => {
                write!(fmt, "{list_prefix} {text}")
            }
        }
    }
}

impl Display for Attribute {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.name {
            write!(fmt, "{name}=")?;
        }

        write!(fmt, "{}", self.value)
    }
}
