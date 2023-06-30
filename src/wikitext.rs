use std::fmt;
use std::fmt::Display;

/// The root of a wikitext document.
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
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
#[cfg_attr(serde, derive(Serialize, Deserialize))]
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
#[cfg_attr(serde, derive(Serialize, Deserialize))]
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
pub struct Text {
    pub pieces: Vec<TextPiece>,
}

impl Text {
    pub fn new() -> Self {
        Default::default()
    }

    /// Extend the current last text piece with the given string,
    /// or append a new text piece created from the given string if there is no text piece
    /// or the last text piece is not of variant [`Text`](TextPiece::Text).
    pub fn extend_with_text(&mut self, text: &str) {
        if let Some(TextPiece::Text(last)) = self.pieces.last_mut() {
            last.push_str(text);
        } else {
            self.pieces.push(TextPiece::Text(text.to_string()));
        }
    }
}

/// A piece of text of a section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub enum TextPiece {
    /// A plain string.
    Text(String),
    /// A double brace expression.
    DoubleBraceExpression {
        tag: String,
        attributes: Vec<Attribute>,
    },
}

/// An attribute of e.g. a double brace expression.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub struct Attribute {
    pub name: Option<String>,
    pub value: Text,
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

                write!(fmt, "}}")
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
