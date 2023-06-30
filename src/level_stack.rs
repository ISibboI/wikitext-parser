use crate::wikitext::{Headline, Section, TextPiece};

/// Data structure used to parse wikitext sections and headlines at different levels.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LevelStack {
    stack: Vec<Vec<Section>>,
}

impl LevelStack {
    /// Create a new headline for a page with the given headline.
    pub fn new(headline: String) -> Self {
        Self {
            stack: vec![vec![Section {
                headline: Headline {
                    label: headline,
                    level: 1,
                },
                text: Vec::new(),
                subsections: vec![],
            }]],
        }
    }

    fn top_mut(&mut self) -> &mut Vec<Section> {
        self.stack.last_mut().unwrap()
    }

    /// Append a new headline found on the page.
    pub fn append_headline(&mut self, headline: Headline) {
        self.adjust_level(headline.level.into());
        debug_assert!(self.stack.len() > 1);
        self.top_mut().push(Section {
            headline,
            text: Vec::new(),
            subsections: vec![],
        });
    }

    /// Append a text piece found on the page.
    pub fn append_text_piece(&mut self, text_piece: TextPiece) {
        self.top_mut().last_mut().unwrap().text.push(text_piece);
    }

    /// Extend the current last text piece with the given string,
    /// or append a new text piece created from the given string if there is no text piece
    /// or the last text piece is not of variant [`Text`](TextPiece::Text).
    pub fn extend_text_piece(&mut self, text: &str) {
        let current_text = &mut self.top_mut().last_mut().unwrap().text;
        if let Some(TextPiece::Text(last)) = current_text.last_mut() {
            last.push_str(text);
        } else {
            current_text.push(TextPiece::Text(text.to_string()));
        }
    }

    fn adjust_level(&mut self, level: usize) {
        while self.stack.len() > level {
            let mut last = self.stack.pop().unwrap();
            while self.stack.last().unwrap().is_empty() {
                self.stack.pop();
            }
            self.top_mut()
                .last_mut()
                .unwrap()
                .subsections
                .append(&mut last);
        }
        while self.stack.len() < level {
            self.stack.push(Vec::new());
        }
        debug_assert_eq!(self.stack.len(), level);
    }

    /// Collapse the stack down to the root section and return it.
    /// The root section contains the whole section hierarchy added to the stack.
    pub fn into_root_section(mut self) -> Section {
        self.adjust_level(1);
        debug_assert_eq!(self.stack.len(), 1);
        let mut level_1 = self.stack.pop().unwrap();
        debug_assert_eq!(level_1.len(), 1);
        level_1.pop().unwrap()
    }
}
