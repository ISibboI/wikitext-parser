//! Parse wikitext into a semantic representation.

#![warn(missing_docs)]

use crate::error::Error;
use error::Result;
#[cfg(serde)]
use serde::{Deserialize, Serialize};

mod error;
#[cfg(test)]
mod tests;
mod tokenizer;

const MAX_SECTION_DEPTH: usize = 6;

/// The root of a wikitext document.
#[derive(Debug, Eq, PartialEq, Clone)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub struct Wikitext {
    /// The root of the section tree of the page.
    pub root_section: Section,
}

/// A section of wikitext.
#[derive(Debug, Clone, Eq, PartialEq)]
#[cfg_attr(serde, derive(Serialize, Deserialize))]
pub struct Section {
    /// The headline of the section.
    pub headline: Headline,
    /// The text of the section.
    pub text: String,
    /// The subsections of the section.
    pub subsections: Vec<Section>,
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

/// Parse textual wikitext into a semantic representation.
pub fn parse_wikitext(wikitext: &str, headline: String) -> Result<Wikitext> {
    let mut level_stack = Vec::new();
    let (mut text, mut section_text, mut next_headline) = find_next_headline(wikitext)?;
    level_stack.push(vec![Section {
        headline: Headline {
            label: headline,
            level: 1,
        },
        text: section_text,
        subsections: vec![],
    }]);

    while let Some(current_headline) = next_headline {
        if current_headline.level == 1 {
            return Err(Error::SecondRootSection {
                label: current_headline.label,
            });
        }

        (text, section_text, next_headline) = find_next_headline(text)?;
        while usize::from(current_headline.level) < level_stack.len() {
            let mut last = level_stack.pop().unwrap();
            while level_stack.last().unwrap().is_empty() {
                level_stack.pop();
            }
            level_stack
                .last_mut()
                .unwrap()
                .last_mut()
                .unwrap()
                .subsections
                .append(&mut last);
        }
        while level_stack.len() < usize::from(current_headline.level) {
            level_stack.push(Vec::new());
        }
        debug_assert!(level_stack.len() > 1);
        level_stack.last_mut().unwrap().push(Section {
            headline: current_headline,
            text: section_text,
            subsections: vec![],
        });
    }

    while level_stack.len() > 1 {
        let mut last = level_stack.pop().unwrap();
        while level_stack.last().unwrap().is_empty() {
            level_stack.pop();
        }
        level_stack
            .last_mut()
            .unwrap()
            .last_mut()
            .unwrap()
            .subsections
            .append(&mut last);
    }

    let root_section = level_stack.pop().unwrap().pop().unwrap();
    debug_assert!(level_stack.is_empty());
    Ok(Wikitext { root_section })
}

fn find_next_headline(text: &str) -> Result<(&str, String, Option<Headline>)> {
    let mut previous_text_limit = 0;
    loop {
        if let Some(location) = text[previous_text_limit..].find('=') {
            previous_text_limit += location;
        } else {
            return Ok((&text[text.len()..], text.to_string(), None));
        }

        let level = text[previous_text_limit..]
            .chars()
            .take_while(|&c| c == '=')
            .count();
        if level > MAX_SECTION_DEPTH {
            previous_text_limit += level;
            continue;
        }
        let headline_marker = &text[previous_text_limit..previous_text_limit + level];
        let headline_candidate = &text[previous_text_limit + level..];

        if let Some(location) = headline_candidate.find(headline_marker) {
            let headline_candidate = &headline_candidate[..location];
            if headline_candidate.contains('\n') {
                previous_text_limit += level;
                continue;
            }

            let label = headline_candidate.trim().to_string();
            let level_u8 = u8::try_from(level).unwrap();
            let headline = Headline {
                label,
                level: level_u8,
            };
            let next_text_offset = previous_text_limit + level + location + level;
            println!("Found headline {headline:?}");
            return Ok((
                &text[next_text_offset..],
                text[..previous_text_limit].to_string(),
                Some(headline),
            ));
        } else {
            previous_text_limit += level;
        }
    }
}
