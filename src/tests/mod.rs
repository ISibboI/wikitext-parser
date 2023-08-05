use crate::wikitext::{Headline, Line, Paragraph};
use crate::{
    parse_wikitext, ParserErrorKind, Section, Text, TextFormatting, TextPiece, TextPosition,
    Wikitext,
};

mod full_pages;

#[test]
fn test_wiktionary_free_substrings() {
    let input_json_strings = [
        r#""[[File:Free Beer.jpg|thumb|A sign advertising '''free''' beer (obtainable without payment). It is a joke: every day the sign is read, the free beer will be available \"tomorrow\".]]""#,
        r#""{{a}}ȝ""#,
        r#""[[s:Twelve O'Clock|Twelve O'Clock]]""#,
    ];
    for input_json_string in input_json_strings {
        let input: String = serde_json::from_str(input_json_string).unwrap();
        println!("{input:?}");
        let mut errors = Vec::new();
        parse_wikitext(
            &input,
            "free".to_string(),
            &mut Box::new(|error| errors.push(error)),
        );
        assert!(errors.is_empty());
    }
}

#[test]
fn test_wiktionary_nested_formatting() {
    let input = r"{{quote-book|en|year=1988|author=Andrew Radford|title=Transformational grammar: a first course|location=Cambridge, UK|publisher=Cambridge University Press|page=339|chapter=7|passage=But what other kind(s) of syntactic information should be included in Lexical Entries? Traditional '''dictionaries''' such as Hornby's (1974) ''Oxford Advanced Learner's '''Dictionary''' of Current English'' include not only ''categorial'' information in their entries, but also information about the range of ''Complements'' which a given item permits (this information is represented by the use of a number/letter code).}}";
    let mut errors = Vec::new();
    parse_wikitext(
        input,
        Default::default(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
}

#[test]
fn test_wiktionary_nested_formatting_with_combined_open_or_close() {
    let input = r"0''a'''b'''''c'''d''e'''''f'''''g''h'''i'''''j'''k''l'''''m'''''";
    let mut errors = Vec::new();
    assert_eq!(
        parse_wikitext(
            input,
            Default::default(),
            &mut Box::new(|error| errors.push(error))
        )
        .root_section,
        Section {
            headline: Headline::new("", 1),
            paragraphs: vec![Paragraph {
                lines: vec![Line::Normal {
                    text: Text {
                        pieces: vec![
                            TextPiece::Text {
                                formatting: TextFormatting::Normal,
                                text: "0".to_string()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::Italic,
                                text: "a".into()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::ItalicBold,
                                text: "b".into()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::Normal,
                                text: "c".to_string()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::Bold,
                                text: "d".into()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::ItalicBold,
                                text: "e".into()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::Normal,
                                text: "f".to_string()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::ItalicBold,
                                text: "g".into()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::Bold,
                                text: "h".into()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::Normal,
                                text: "i".to_string()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::ItalicBold,
                                text: "j".into()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::Italic,
                                text: "k".into()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::Normal,
                                text: "l".to_string()
                            },
                            TextPiece::Text {
                                formatting: TextFormatting::ItalicBold,
                                text: "m".into()
                            },
                        ]
                    }
                }],
            }],
            subsections: Default::default(),
        }
    );
    assert!(errors.is_empty());
}

#[test]
fn test_complex_internal_links() {
    let input = r#"[[File:Free Beer.jpg|thumb|A sign advertising '''free''' beer (obtainable without payment). It is a joke: every day the sign is read, the free beer will be available &quot;tomorrow&quot; ([[ergo]] never).]]
[[File:Buy one, get one free ^ - geograph.org.uk - 153952.jpg|thumb|A &quot;buy one get one '''free'''&quot; sign at a flower stand (obtainable without additional payment).]]
[[File:Berkeley Farms Fat-Free Half &amp; Half.jpg|thumb|This food product ([[half and half]]) is labelled &quot;fat '''free'''&quot;, meaning it contains no detectable fat.]]"#;
    let mut errors = Vec::new();
    parse_wikitext(
        input,
        Default::default(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
}

#[test]
fn test_section_headers() {
    let input = "<ref name=\"ISample\"/><ref name=\"COttoni\"/>";
    let mut errors = Vec::new();
    parse_wikitext(
        input,
        Default::default(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
}

#[test]
fn test_headlines() {
    let input = "==abc===c==b==g== a \n ==c==";
    let mut errors = Vec::new();
    let parsed = parse_wikitext(
        input,
        "title".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    let headlines = parsed.list_headlines();
    assert_eq!(headlines, vec![Headline::new("title", 1),]);

    let input = "==abc== \n =c= \n==b==g== a \n=====c=====";
    let mut errors = Vec::new();
    let parsed = parse_wikitext(
        input,
        "title".to_string(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
    let headlines = parsed.list_headlines();
    assert_eq!(
        headlines,
        vec![
            Headline::new("title", 1),
            Headline::new("abc", 2),
            Headline::new("c", 5),
        ]
    );
}

#[test]
fn test_equals_in_tag() {
    let input = "{{fake==|English}}\n{{fake===|Noun}}\n{{fake====|Synonyms}}";
    let mut errors = Vec::new();
    parse_wikitext(
        input,
        Default::default(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert!(errors.is_empty());
}

#[test]
fn test_complex_double_brace_expression() {
    let input_json = "\"{{der3|vi\\n|{{vi-l|y tá|醫佐|[[nurse]]}}\\n|{{vi-l|Đông y|東醫|[[traditional]] [[East Asian]] medicine]]}}\\n|{{vi-l|y học|醫學|[[medicine]]}}\\n|{{vi-l|Tây y|西醫|[[modern]] [[medicine]]}}\\n|{{vi-l|pháp y|法醫|[[forensic]] [[science]]}}\\n|{{vi-l|y khoa|醫科|[[medicine]]}}\\n|{{vi-l|y sĩ|醫士|([[junior]]) [[physician]]}}\\n|{{vi-l|y tế|醫濟|[[health care]]}}\\n|{{vi-l|nan y|難醫|(of [[disease]]) [[difficult]] to [[cure]]}}\\n|{{vi-l|lương y|良醫|([[literary]]) a [[good]] [[physician]]}}\\n|{{vi-l|y sinh|醫生|[[physician]]}}\\n|{{vi-l|y dược|醫藥|[[medicine]] and [[pharmacy]]}}\\n|{{vi-l|y viện|醫院|([[literary]]) [[hospital]]}}\\n|{{vi-l|lương y như từ mẫu|良醫如慈母|([[literary]]) a [[good]] [[physician]] is [[like]] a good [[mother]]}}\\n|{{vi-l|y đạo|醫道|([[literary]]) [[art]] of [[healing]]}}\\n|{{vi-l|y lệnh|醫令|[[doctor]]'s [[instructions]]}}\\n}}\"";
    let input: String = serde_json::from_str(input_json).unwrap();
    let mut errors = Vec::new();
    parse_wikitext(
        &input,
        Default::default(),
        &mut Box::new(|error| errors.push(error)),
    );
    assert_eq!(
        errors,
        vec![
            ParserErrorKind::UnmatchedDoubleCloseBracket.into_parser_error(TextPosition {
                line: 3,
                column: 58
            })
        ]
    );
}

#[test]
fn test_multiple_root_sections() {
    let input_json = "\"=a=\\nsome text\\n=b=\\nsome more text\\n=c=\"";
    let input: String = serde_json::from_str(input_json).unwrap();
    let mut errors = Vec::new();
    let parsed = parse_wikitext(
        &input,
        Default::default(),
        &mut Box::new(|error| errors.push(error)),
    );

    assert_eq!(
        errors,
        vec![
            ParserErrorKind::SecondRootSection {
                label: "a".to_string()
            }
            .into_parser_error(TextPosition { line: 1, column: 1 }),
            ParserErrorKind::SecondRootSection {
                label: "b".to_string()
            }
            .into_parser_error(TextPosition { line: 3, column: 1 }),
            ParserErrorKind::SecondRootSection {
                label: "c".to_string()
            }
            .into_parser_error(TextPosition { line: 5, column: 1 }),
        ]
    );

    assert_eq!(
        parsed,
        Wikitext {
            root_section: Section {
                headline: Headline::new("", 1),
                paragraphs: Vec::new(),
                subsections: Vec::new()
            },
        }
    );
}
