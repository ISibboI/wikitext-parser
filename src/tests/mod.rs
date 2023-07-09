use crate::parse_wikitext;
use crate::wikitext::Headline;

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
        parse_wikitext(&input, "free".to_string()).unwrap();
    }
}

#[test]
fn test_wiktionary_nested_formatting() {
    let input = r"{{quote-book|en|year=1988|author=Andrew Radford|title=Transformational grammar: a first course|location=Cambridge, UK|publisher=Cambridge University Press|page=339|chapter=7|passage=But what other kind(s) of syntactic information should be included in Lexical Entries? Traditional '''dictionaries''' such as Hornby's (1974) ''Oxford Advanced Learner's '''Dictionary''' of Current English'' include not only ''categorial'' information in their entries, but also information about the range of ''Complements'' which a given item permits (this information is represented by the use of a number/letter code).}}";
    parse_wikitext(&input, Default::default()).unwrap();
}

#[test]
fn test_complex_internal_links() {
    let input = r#"[[File:Free Beer.jpg|thumb|A sign advertising '''free''' beer (obtainable without payment). It is a joke: every day the sign is read, the free beer will be available &quot;tomorrow&quot; ([[ergo]] never).]]
[[File:Buy one, get one free ^ - geograph.org.uk - 153952.jpg|thumb|A &quot;buy one get one '''free'''&quot; sign at a flower stand (obtainable without additional payment).]]
[[File:Berkeley Farms Fat-Free Half &amp; Half.jpg|thumb|This food product ([[half and half]]) is labelled &quot;fat '''free'''&quot;, meaning it contains no detectable fat.]]"#;
    parse_wikitext(&input, Default::default()).unwrap();
}

#[test]
fn test_headlines() {
    let input = "==abc===c==b==g== a \n ==c==";
    let parsed = parse_wikitext(&input, "title".to_string()).unwrap();
    let headlines = parsed.list_headlines();
    assert_eq!(
        headlines,
        vec![
            Headline::new("title", 1),
            Headline::new("b", 2),
            Headline::new("c", 2),
        ]
    );
}
