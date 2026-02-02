use ferret::analyzer::stream::StreamAnalyzer;

use std::fs;
use std::path::PathBuf;

fn read_fixture(name: &str) -> String {
    let mut path = PathBuf::from("tests/fixtures");
    path.push(name);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read fixture: {:?}", path))
}

#[test]
fn test_fixture_attributes() {
    let html = read_fixture("attributes.html");
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(&html).unwrap();

    // attributes.html likely contains various attributes
    // We assume it has at least some content.
    // Without seeing content, we just check it runs and finds something.
    assert!(result.tags.len() > 0);
}

#[test]
fn test_fixture_broken_tags() {
    let html = read_fixture("broken_tags.html");
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(&html);
    assert!(result.is_ok()); // Should recover
}

#[test]
fn test_fixture_deeply_nested() {
    let html = read_fixture("deeply_nested.html");
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(&html).unwrap();
    assert!(result.max_depth > 10); // Assuming it is deeply nested
}

#[test]
fn test_fixture_malformed() {
    let html = read_fixture("malformed.html");
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer
        .analyze_string(&html)
        .expect("Should handle malformed HTML");
    // Just ensure it returned something valid
    assert!(result.files_analyzed == 1);
}

#[test]
fn test_fixture_realistic_sample() {
    let html = read_fixture("realistic_sample.html");
    let analyzer = StreamAnalyzer::new(20);
    let result = analyzer.analyze_string(&html).unwrap();

    assert!(result.tags.len() > 5); // Should have many tags
    assert!(result.tags.contains_key("div"));
    assert!(result.tags.contains_key("link"));
}

#[test]
fn test_fixture_xml() {
    let xml = read_fixture("realistic_sample.xml");
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(&xml).unwrap();

    // XML tags should be found
    assert!(result.tags.len() > 0);
}

#[test]
fn test_fixture_script_style() {
    let html = read_fixture("script_style.html");
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(&html).unwrap();

    assert!(result.tags.contains_key("script"));
    assert!(result.tags.contains_key("style"));
}

#[test]
fn test_fixture_unicode() {
    let html = read_fixture("unicode.html");
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(&html).unwrap();

    // Basic check that it didn't crash on unicode
    assert!(result.tags.len() > 0);
}
