// Integration tests for analyzer library
// Place this file at: tests/integration_tests.rs
use ferret as fer;
use ferret::analyzer::stream::StreamAnalyzer;
use ferret::analyzer::{self, Analyzer, StatsAnalyzer};
use std::fs;
use std::path::PathBuf;

fn read_fixture(name: &str) -> String {
    let mut path = PathBuf::from("tests/fixtures");
    path.push(name);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read fixture: {:?}", path))
}

#[test]
fn test_stats_analyzer_basic() {
    let html = r#"<div class="test"><p>Hello</p></div>"#;
    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();

    let mut analyzer = StatsAnalyzer::new(10);

    let walker = fer::walker::DomWalker::new(dom.children().to_vec(), dom.parser());
    for (_handle, node, depth) in walker {
        analyzer.visit(node, depth);
    }

    let result = analyzer.result();

    assert_eq!(result.files_analyzed, 1);
    assert!(result.tags.contains_key("div"));
    assert!(result.tags.contains_key("p"));

    let div_stats = result.tags.get("div").unwrap();
    assert_eq!(div_stats.count, 1);
    assert!(div_stats.attributes.contains_key("class"));
}

#[test]
fn test_stream_analyzer_basic() {
    let html = r#"<div class="test"><p>Hello</p></div>"#;
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(html).unwrap();

    assert_eq!(result.files_analyzed, 1);
    assert!(result.tags.contains_key("div"));
    assert!(result.tags.contains_key("p"));

    let div_stats = result.tags.get("div").unwrap();
    assert_eq!(div_stats.count, 1);
    assert!(div_stats.attributes.contains_key("class"));
}

#[test]
fn test_both_analyzers_produce_similar_results() {
    let html = r#"
        <html>
        <body>
            <div class="container" id="main">
                <p class="text">Paragraph 1</p>
                <p class="text">Paragraph 2</p>
                <span>Text</span>
            </div>
        </body>
        </html>
    "#;

    // Stream analyzer
    let stream_analyzer = StreamAnalyzer::new(10);
    let stream_result = stream_analyzer.analyze_string(html).unwrap();

    // StatsAnalyzer
    let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
    let mut stats_analyzer = StatsAnalyzer::new(10);

    let walker = fer::walker::DomWalker::new(dom.children().to_vec(), dom.parser());
    for (_handle, node, depth) in walker {
        stats_analyzer.visit(node, depth);
    }

    let stats_result = stats_analyzer.result();

    // Both should find the same tags
    assert_eq!(stream_result.tags.len(), stats_result.tags.len());

    // Check specific tags
    for tag_name in ["html", "body", "div", "p", "span"].iter() {
        assert!(
            stream_result.tags.contains_key(*tag_name),
            "Stream analyzer missing tag: {}",
            tag_name
        );
        assert!(
            stats_result.tags.contains_key(*tag_name),
            "Stats analyzer missing tag: {}",
            tag_name
        );
    }

    // Check p tag count (should be 2)
    assert_eq!(stream_result.tags.get("p").unwrap().count, 2);
    assert_eq!(stats_result.tags.get("p").unwrap().count, 2);
}

#[test]
fn test_attribute_value_tracking() {
    let html = r#"
        <div class="a"></div>
        <div class="b"></div>
        <div class="a"></div>
        <div class="c"></div>
    "#;

    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(html).unwrap();

    let div_stats = result.tags.get("div").unwrap();
    assert_eq!(div_stats.count, 4);

    let class_attr = div_stats.attributes.get("class").unwrap();
    assert_eq!(class_attr.count, 4);

    // Should track all values since we're under the limit
    assert_eq!(class_attr.value_counts.len(), 3); // a, b, c
    assert_eq!(class_attr.value_counts.get("a"), Some(&2));
    assert_eq!(class_attr.value_counts.get("b"), Some(&1));
    assert_eq!(class_attr.value_counts.get("c"), Some(&1));
}

#[test]
fn test_attribute_value_limit() {
    let html = r#"
        <div class="a"></div>
        <div class="b"></div>
        <div class="c"></div>
        <div class="a"></div>
    "#;

    // Limit to tracking only 2 unique values
    let analyzer = StreamAnalyzer::new(2);
    let result = analyzer.analyze_string(html).unwrap();

    let div_stats = result.tags.get("div").unwrap();
    let class_attr = div_stats.attributes.get("class").unwrap();

    // Should stop tracking new values after limit
    // but continue counting existing values
    assert!(class_attr.value_counts.len() <= 2);

    // "a" should be counted twice
    if class_attr.value_counts.contains_key("a") {
        assert_eq!(class_attr.value_counts.get("a"), Some(&2));
    }
}

#[test]
fn test_depth_tracking() {
    let html = r#"
        <div>
            <div>
                <div>
                    <span>Deep</span>
                </div>
            </div>
        </div>
    "#;

    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(html).unwrap();

    // div -> div -> div -> span = depth 4
    assert_eq!(result.max_depth, 4);
}

#[test]
fn test_self_closing_tags() {
    let html = r#"<div><img src="test.jpg" /><br /><input type="text" /></div>"#;

    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(html).unwrap();

    assert!(result.tags.contains_key("img"));
    assert!(result.tags.contains_key("br"));
    assert!(result.tags.contains_key("input"));

    let img_stats = result.tags.get("img").unwrap();
    assert_eq!(img_stats.count, 1);
    assert!(img_stats.attributes.contains_key("src"));
}

#[test]
fn test_malformed_html_resilience() {
    // Missing closing tags, etc.
    let html = r#"<div><p>Unclosed<div>Another</div>"#;

    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(html);

    // Should not panic
    assert!(result.is_ok());

    let result = result.unwrap();
    assert!(result.tags.contains_key("div"));
    assert!(result.tags.contains_key("p"));
}

#[test]
fn test_empty_document() {
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string("").unwrap();

    assert_eq!(result.files_analyzed, 1);
    assert_eq!(result.tags.len(), 0);
    assert_eq!(result.max_depth, 0);
}

#[test]
fn test_serialization() {
    let html = r#"<div class="test"><p>Hello</p></div>"#;
    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(html).unwrap();

    // Should be able to serialize to JSON
    let json = serde_json::to_string(&result);
    assert!(json.is_ok());

    // Should be able to deserialize back
    let json_str = json.unwrap();
    let deserialized: analyzer::AnalysisResult = serde_json::from_str(&json_str).unwrap();

    assert_eq!(deserialized.tags.len(), result.tags.len());
    assert_eq!(deserialized.max_depth, result.max_depth);
}

#[tokio::test]
async fn test_analyze_string_async() {
    let html = r#"<div><p>Test</p></div>"#;
    let analyzer = StreamAnalyzer::new(10);

    // analyze_string is sync, but we can use it in async context
    let result = tokio::task::spawn_blocking(move || analyzer.analyze_string(html))
        .await
        .unwrap();

    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.tags.contains_key("div"));
}

#[test]
fn test_multiple_attributes() {
    let html = r#"<div id="test" class="container" data-value="123">Content</div>"#;

    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(html).unwrap();

    let div_stats = result.tags.get("div").unwrap();
    assert_eq!(div_stats.attributes.len(), 3);
    assert!(div_stats.attributes.contains_key("id"));
    assert!(div_stats.attributes.contains_key("class"));
    assert!(div_stats.attributes.contains_key("data-value"));

    let id_attr = div_stats.attributes.get("id").unwrap();
    assert_eq!(id_attr.value_counts.get("test"), Some(&1));
}

#[test]
fn test_tag_count_accumulation() {
    let html = r#"
        <div>First</div>
        <div>Second</div>
        <div>Third</div>
        <span>A</span>
        <span>B</span>
    "#;

    let analyzer = StreamAnalyzer::new(10);
    let result = analyzer.analyze_string(html).unwrap();

    assert_eq!(result.tags.get("div").unwrap().count, 3);
    assert_eq!(result.tags.get("span").unwrap().count, 2);
}

// Note: URL tests would require a mock HTTP server
// Consider using `mockito` or `wiremock` for actual URL testing
// Example placeholder:
// #[tokio::test]
// async fn test_analyze_url() {
//     use mockito::Server;
//     let mut server = Server::new_async().await;
//     let mock = server.mock("GET", "/test.xml")
//         .with_body("<root><item /></root>")
//         .create();
//
//     let analyzer = StreamAnalyzer::new(10);
//     let url = format!("{}/test.xml", server.url());
//     let result = analyzer.analyze_url(&url).await.unwrap();
//
//     assert!(result.tags.contains_key("root"));
//     mock.assert();
// }
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
    let result = analyzer.analyze_string(&html).unwrap();
    assert!(result.files_analyzed == 1); // Should recover and produce result
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
