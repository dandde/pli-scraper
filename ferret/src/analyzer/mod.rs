use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tl::Node;

pub mod stream;

pub trait Analyzer {
    fn visit(&mut self, node: &Node, depth: usize) -> bool;
    fn result(&self) -> AnalysisResult;
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub tags: HashMap<String, TagStats>,
    pub files_analyzed: usize,
    pub max_depth: usize,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct TagStats {
    pub name: String,
    pub count: usize,
    pub attributes: HashMap<String, AttributeStats>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AttributeStats {
    pub name: String,
    pub count: usize,
    pub value_counts: HashMap<String, usize>,
}

pub struct StatsAnalyzer {
    result: AnalysisResult,
    top_values_limit: usize,
}

impl StatsAnalyzer {
    pub fn new(top_values_limit: usize) -> Self {
        Self {
            result: AnalysisResult {
                tags: HashMap::new(),
                files_analyzed: 1, // Single file scope
                max_depth: 0,
            },
            top_values_limit,
        }
    }
}

impl Analyzer for StatsAnalyzer {
    fn visit(&mut self, node: &Node, depth: usize) -> bool {
        if depth > self.result.max_depth {
            self.result.max_depth = depth;
        }

        if let Some(tag) = node.as_tag() {
            let tag_name = tag.name().as_utf8_str().to_string();

            let tag_stats = self
                .result
                .tags
                .entry(tag_name.clone())
                .or_insert_with(|| TagStats {
                    name: tag_name,
                    count: 0,
                    attributes: HashMap::new(),
                });

            tag_stats.count += 1;

            for (key, val_opt) in tag.attributes().iter() {
                let attr_name = key.as_ref().to_string();
                let val_str = val_opt
                    .as_ref()
                    .map(|v| v.as_ref().to_string())
                    .unwrap_or_default();

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
                // Optimization: Don't track if over very large limit, but for now simple map
                if attr_stats.value_counts.len() < self.top_values_limit
                    || attr_stats.value_counts.contains_key(&val_str)
                {
                    *attr_stats.value_counts.entry(val_str).or_insert(0) += 1;
                }
            }
        }

        true // Continue visiting children
    }

    fn result(&self) -> AnalysisResult {
        self.result.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::FerretParser;
    use crate::walker::DomWalker;

    #[test]
    fn test_stats() {
        let html = r#"<div class="container"><p>Hello</p><p class="text">World</p></div>"#;
        let vdom = FerretParser::parse(html).unwrap();
        let walker = DomWalker::new(vdom.children().to_vec(), vdom.parser());
        let mut analyzer = StatsAnalyzer::new(5);

        for (_handle, node, depth) in walker {
            analyzer.visit(node, depth);
        }

        let result = analyzer.result();

        assert_eq!(result.tags.get("div").map(|t| t.count), Some(1));
        assert_eq!(result.tags.get("p").map(|t| t.count), Some(2));

        let div_stats = result.tags.get("div").unwrap();
        assert_eq!(div_stats.attributes.get("class").map(|a| a.count), Some(1));
        assert_eq!(
            div_stats
                .attributes
                .get("class")
                .unwrap()
                .value_counts
                .get("container"),
            Some(&1)
        );
    }
}
