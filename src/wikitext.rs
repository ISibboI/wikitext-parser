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
}

/// A section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub struct Section {
    /// The headline of the section.
    pub headline: Headline,
    /// The text of the section.
    pub text: Vec<TextPiece>,
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
        for text_piece in &self.text {
            if matches!(text_piece, TextPiece::DoubleBraceExpression(_)) {
                result.push(text_piece.clone());
            }
        }
        for subsection in &self.subsections {
            subsection.list_double_brace_expressions(result);
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

/// A piece of text of a section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub enum TextPiece {
    /// A plain string.
    Text(String),
    /// A double brace expression.
    DoubleBraceExpression(Vec<TextPiece>),
}

impl ToString for TextPiece {
    fn to_string(&self) -> String {
        match self {
            TextPiece::Text(text) => text.clone(),
            TextPiece::DoubleBraceExpression(text_pieces) => {
                format!(
                    "{{{{{}}}}}",
                    text_pieces
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("|")
                )
            }
        }
    }
}
