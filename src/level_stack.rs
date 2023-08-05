use crate::wikitext::{Headline, Line, Section};

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
                paragraphs: Default::default(),
                subsections: Default::default(),
            }]],
        }
    }

    fn top_mut(&mut self) -> &mut Vec<Section> {
        self.stack.last_mut().unwrap()
    }

    /// Append a new headline found on the page.
    pub fn append_headline(&mut self, headline: Headline) {
        self.adjust_level(headline.level.into());
        debug_assert!(!self.stack.is_empty());
        self.top_mut().push(Section {
            headline,
            paragraphs: Default::default(),
            subsections: Default::default(),
        });
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
    /// 
    /// If there is more than one root section, then only the first is returned.
    pub fn into_root_section(mut self) -> Section {
        self.adjust_level(1);
        debug_assert_eq!(self.stack.len(), 1);
        let mut level_1 = self.stack.pop().unwrap();
        debug_assert!(!level_1.is_empty());
        level_1.remove(0)
    }

    /// Starts a new paragraph if the current last is not empty.
    pub fn new_paragraph(&mut self) {
        let current_section = self.top_mut().last_mut().unwrap();
        if current_section
            .paragraphs
            .last()
            .map(|paragraph| !paragraph.lines.is_empty())
            .unwrap_or(true)
        {
            current_section.paragraphs.push(Default::default());
        }
    }

    /// Appends a line to the current last paragraph.
    /// Drops the line if it is empty.
    pub fn append_line(&mut self, line: Line) {
        if line.is_empty() {
            return;
        }
        let current_section = self.top_mut().last_mut().unwrap();
        if current_section.paragraphs.is_empty() {
            current_section.paragraphs.push(Default::default());
        }
        current_section
            .paragraphs
            .last_mut()
            .unwrap()
            .lines
            .push(line);
    }
}
