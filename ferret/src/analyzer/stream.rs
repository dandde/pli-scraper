use crate::analyzer::{AnalysisResult, AttributeStats, TagStats};
use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

/// Stream-based analyzer for large files and URLs
///
/// Unlike StatsAnalyzer which loads the entire document into memory,
/// StreamAnalyzer processes XML/HTML in a streaming fashion, making it
/// suitable for:
/// - Large files that don't fit comfortably in memory
/// - Remote URLs that need to be fetched
/// - Situations where you want to start processing before the entire file is loaded
pub struct StreamAnalyzer {
    pub top_values_limit: usize,
    pub proxy_url: Option<String>,
}

impl StreamAnalyzer {
    /// Create a new stream analyzer
    ///
    /// # Arguments
    /// * `top_values_limit` - Maximum number of unique values to track per attribute
    pub fn new(top_values_limit: usize) -> Self {
        Self {
            top_values_limit,
            proxy_url: None,
        }
    }

    /// Create a stream analyzer configured to use a CORS proxy
    ///
    /// # Arguments
    /// * `top_values_limit` - Maximum number of unique values to track per attribute
    /// * `proxy_url` - Base URL of the CORS proxy server (e.g., "http://localhost:8080/")
    ///
    /// # Example
    /// ```
    /// let analyzer = StreamAnalyzer::with_proxy(10, "http://localhost:8080/".to_string());
    /// let result = analyzer.analyze_url("https://example.com/data.xml").await?;
    /// ```
    pub fn with_proxy(top_values_limit: usize, proxy_url: String) -> Self {
        Self {
            top_values_limit,
            proxy_url: Some(proxy_url),
        }
    }

    /// Analyze a local file
    ///
    /// # Arguments
    /// * `path` - Path to the XML/HTML file
    ///
    /// # Example
    /// ```
    /// use std::path::Path;
    /// let analyzer = StreamAnalyzer::new(10);
    /// let result = analyzer.analyze_file(Path::new("data.xml"))?;
    /// ```
    pub fn analyze_file(&self, path: &Path) -> Result<AnalysisResult> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.analyze_reader(reader)
    }

    /// Analyze content from a URL
    ///
    /// If a proxy URL is configured via `with_proxy()`, requests will be
    /// routed through the proxy to bypass CORS restrictions.
    ///
    /// # Arguments
    /// * `url` - Full URL to the XML/HTML resource
    ///
    /// # Example
    /// ```
    /// let analyzer = StreamAnalyzer::new(10);
    /// let result = analyzer.analyze_url("https://example.com/data.xml").await?;
    /// ```
    pub async fn analyze_url(&self, url: &str) -> Result<AnalysisResult> {
        let target_url = if let Some(proxy) = &self.proxy_url {
            // Route through proxy by appending the target URL
            // Proxy format: http://proxy.example.com/{target_url}
            format!(
                "{}{}",
                proxy,
                url.trim_start_matches("https://")
                    .trim_start_matches("http://")
            )
        } else {
            url.to_string()
        };

        let response = reqwest::get(&target_url).await?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let content = response.text().await?;
        let reader = Cursor::new(content.into_bytes());
        self.analyze_reader(reader)
    }

    /// Analyze content from a string
    ///
    /// Useful for testing or when you already have the content loaded.
    ///
    /// # Arguments
    /// * `content` - XML/HTML content as a string
    ///
    /// # Example
    /// ```
    /// let analyzer = StreamAnalyzer::new(10);
    /// let html = r#"<html><body><div id="test">Hello</div></body></html>"#;
    /// let result = analyzer.analyze_string(html)?;
    /// ```
    pub fn analyze_string(&self, content: &str) -> Result<AnalysisResult> {
        let reader = Cursor::new(content.as_bytes());
        self.analyze_reader(reader)
    }

    /// Core analysis logic that works with any BufRead implementation
    ///
    /// This is the internal method that all other public methods delegate to.
    /// It performs streaming XML/HTML parsing using quick-xml.
    fn analyze_reader<R: std::io::BufRead>(&self, reader: R) -> Result<AnalysisResult> {
        let mut reader = Reader::from_reader(reader);
        reader.trim_text(true);
        reader.check_end_names(false); // Be permissive with HTML

        let mut buf = Vec::new();
        let mut result = AnalysisResult::default();
        result.files_analyzed = 1;

        // Depth tracking is approximate in streaming mode without strict XML
        let mut depth = 0;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    depth += 1;
                    if depth > result.max_depth {
                        result.max_depth = depth;
                    }

                    self.process_element(&e, &mut result);
                }
                Ok(Event::Empty(e)) => {
                    // Self-closing tags like <img /> or <br />
                    self.process_element(&e, &mut result);
                }
                Ok(Event::End(_)) => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => {
                    // Ignore errors to be resilient with malformed HTML/XML
                }
                _ => (),
            }
            buf.clear();
        }

        Ok(result)
    }

    /// Process a single XML/HTML element (tag and its attributes)
    ///
    /// This method updates the result statistics for a given tag.
    fn process_element<'a>(
        &self,
        e: &quick_xml::events::BytesStart<'a>,
        result: &mut AnalysisResult,
    ) {
        // Process Tag
        let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
        let tag_stats = result
            .tags
            .entry(tag_name.clone())
            .or_insert_with(|| TagStats {
                name: tag_name,
                count: 0,
                attributes: HashMap::new(),
            });
        tag_stats.count += 1;

        // Process Attributes
        for attr in e.attributes() {
            if let Ok(attr) = attr {
                let attr_name = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                let attr_val = String::from_utf8_lossy(&attr.value).to_string();

                let attr_stats = tag_stats
                    .attributes
                    .entry(attr_name.clone())
                    .or_insert_with(|| AttributeStats {
                        name: attr_name,
                        count: 0,
                        value_counts: HashMap::new(),
                    });
                attr_stats.count += 1;

                // Track top N values
                // Optimization: Don't track new values if we've hit the limit,
                // but continue counting existing values
                if attr_stats.value_counts.len() < self.top_values_limit
                    || attr_stats.value_counts.contains_key(&attr_val)
                {
                    *attr_stats.value_counts.entry(attr_val).or_insert(0) += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_html_analysis() {
        let analyzer = StreamAnalyzer::new(10);
        let html = r#"<html><head><title>Test</title></head><body class="main"><div id="content">Hello</div></body></html>"#;
        let result = analyzer.analyze_string(html).unwrap();

        assert_eq!(result.files_analyzed, 1);
        assert!(result.tags.contains_key("html"));
        assert!(result.tags.contains_key("div"));

        let div_stats = result.tags.get("div").unwrap();
        assert_eq!(div_stats.count, 1);
        assert!(div_stats.attributes.contains_key("id"));

        let id_attr = div_stats.attributes.get("id").unwrap();
        assert_eq!(id_attr.count, 1);
        assert_eq!(id_attr.value_counts.get("content"), Some(&1));
    }

    #[test]
    fn test_self_closing_tags() {
        let analyzer = StreamAnalyzer::new(10);
        let html = r#"<div><img src="test.jpg" /><br /></div>"#;
        let result = analyzer.analyze_string(html).unwrap();

        assert!(result.tags.contains_key("img"));
        assert!(result.tags.contains_key("br"));

        let img_stats = result.tags.get("img").unwrap();
        assert_eq!(img_stats.count, 1);
    }

    #[test]
    fn test_depth_tracking() {
        let analyzer = StreamAnalyzer::new(10);
        let html = r#"<div><div><div><span>Deep</span></div></div></div>"#;
        let result = analyzer.analyze_string(html).unwrap();

        assert_eq!(result.max_depth, 4); // div -> div -> div -> span
    }

    #[test]
    fn test_attribute_value_limit() {
        let analyzer = StreamAnalyzer::new(2); // Only track 2 unique values
        let html = r#"
            <div class="a"></div>
            <div class="b"></div>
            <div class="c"></div>
            <div class="a"></div>
        "#;
        let result = analyzer.analyze_string(html).unwrap();

        let div_stats = result.tags.get("div").unwrap();
        let class_attr = div_stats.attributes.get("class").unwrap();

        // Should track "a" and "b", but not "c" (unless it was encountered before the limit)
        // The "a" value should be counted twice
        assert!(class_attr.value_counts.contains_key("a"));
        assert_eq!(class_attr.value_counts.get("a"), Some(&2));
    }

    #[test]
    fn test_malformed_html() {
        let analyzer = StreamAnalyzer::new(10);
        // Missing closing tags, attributes without values, etc.
        let html = r#"<div><p>Unclosed paragraph<div>Another div</div>"#;
        let result = analyzer.analyze_string(html);

        // Should not panic, should produce some result
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_url_analysis_placeholder() {
        // This would require a mock HTTP server for proper testing
        // In a real test suite, you'd use something like `mockito` or `wiremock`
        // to create a mock HTTP server that returns test HTML
    }
}
