use crate::error::{Error, Result};
use crate::level_stack::LevelStack;
use crate::tokenizer::{MultipeekTokenizer, Token, Tokenizer};
use crate::wikitext::{Headline, TextPiece, Wikitext};

/// Parse textual wikitext into a semantic representation.
pub fn parse_wikitext(wikitext: &str, headline: String) -> Result<Wikitext> {
    let mut level_stack = LevelStack::new(headline);
    let mut tokenizer = MultipeekTokenizer::new(Tokenizer::new(wikitext));

    loop {
        match tokenizer.next() {
            Token::Text(text) => level_stack.append_text_piece(TextPiece::Text(text.to_string())),
            Token::MultiEquals(count) => {
                if let Some(headline) = parse_potential_headline(&mut tokenizer, count) {
                    if headline.level == 1 {
                        return Err(Error::SecondRootSection {
                            label: headline.label,
                        });
                    }
                    level_stack.append_headline(headline);
                } else {
                    level_stack
                        .append_text_piece(TextPiece::Text(Token::MultiEquals(count).to_string()));
                }
            }
            Token::DoubleOpenBrace => {
                level_stack.append_text_piece(parse_double_brace_expression(&mut tokenizer)?)
            }
            Token::DoubleCloseBrace => return Err(Error::UnmatchedDoubleCloseBrace),
            Token::Eof => break,
            ignored_token => {
                level_stack.extend_text_piece(ignored_token.to_str());
            }
        }
    }

    Ok(Wikitext {
        root_section: level_stack.into_root_section(),
    })
}

fn parse_potential_headline(tokenizer: &mut MultipeekTokenizer, level: u8) -> Option<Headline> {
    tokenizer.peek(1);
    if let (Some(Token::Text(text)), Some(Token::MultiEquals(second_level))) =
        (tokenizer.repeek(0), tokenizer.repeek(1))
    {
        if level == *second_level && !text.contains('\n') {
            let label = text.trim().to_string();
            tokenizer.next();
            tokenizer.next();
            Some(Headline { label, level })
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_double_brace_expression(tokenizer: &mut MultipeekTokenizer) -> Result<TextPiece> {
    let mut result = Vec::new();

    loop {
        match tokenizer.next() {
            token @ (Token::Text(_) | Token::MultiEquals(_)) => {
                if let Some(TextPiece::Text(text)) = result.last_mut() {
                    text.push_str(&token.to_string());
                } else {
                    result.push(TextPiece::Text(token.to_string()));
                }
            }
            Token::DoubleOpenBrace => result.push(parse_double_brace_expression(tokenizer)?),
            Token::DoubleCloseBrace => break,
            Token::VerticalBar => result.push(TextPiece::Text(String::new())),
            Token::Eof => return Err(Error::UnmatchedDoubleOpenBrace),
        }
    }

    Ok(TextPiece::DoubleBraceExpression(result))
}
