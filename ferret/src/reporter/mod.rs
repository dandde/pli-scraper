use crate::analyzer::AnalysisResult;
use colored::*;
use std::fmt::Write;

pub struct TreeDisplay;

impl TreeDisplay {
    pub fn render(report: &AnalysisResult) -> String {
        let mut out = String::new();
        writeln!(out, "📦 Files analyzed: {}", report.files_analyzed).unwrap();

        // Sort tags by count desc
        let mut sorted_tags: Vec<_> = report.tags.values().collect();
        sorted_tags.sort_by(|a, b| b.count.cmp(&a.count));

        for (i, tag) in sorted_tags.iter().enumerate() {
            let is_last_tag = i == sorted_tags.len() - 1;
            let tag_prefix = if is_last_tag {
                "└── "
            } else {
                "├── "
            };

            writeln!(
                out,
                "{}{}{}",
                tag_prefix,
                tag.name.bright_cyan(),
                format!(" ({})", tag.count).yellow()
            )
            .unwrap();

            // Sort attributes by count desc
            let mut sorted_attrs: Vec<_> = tag.attributes.values().collect();
            sorted_attrs.sort_by(|a, b| b.count.cmp(&a.count));

            let child_indent = if is_last_tag { "    " } else { "│   " };

            for (j, attr) in sorted_attrs.iter().enumerate() {
                let is_last_attr = j == sorted_attrs.len() - 1;
                let attr_prefix = if is_last_attr {
                    "└── "
                } else {
                    "├── "
                };

                writeln!(
                    out,
                    "{}{}@{}{}",
                    child_indent,
                    attr_prefix,
                    attr.name,
                    format!(" ({})", attr.count).dimmed()
                )
                .unwrap();

                // Print top values
                let val_indent = if is_last_attr { "    " } else { "│   " };
                let full_val_indent = format!("{}{}", child_indent, val_indent);

                let mut sorted_vals: Vec<_> = attr.value_counts.iter().collect();
                sorted_vals.sort_by(|a, b| b.1.cmp(a.1));

                for (k, (val, count)) in sorted_vals.iter().take(5).enumerate() {
                    let is_last_val = k == sorted_vals.len().min(5) - 1;
                    let val_prefix = if is_last_val {
                        "└── "
                    } else {
                        "├── "
                    };

                    writeln!(
                        out,
                        "{}{}{} {} ({})",
                        full_val_indent,
                        val_prefix,
                        "──".dimmed(),
                        val,
                        count
                    )
                    .unwrap();
                }
            }
        }
        out
    }
}

pub struct FlatDisplay;

impl FlatDisplay {
    pub fn render(report: &AnalysisResult) -> String {
        let mut out = String::new();
        writeln!(out, "📦 Files analyzed: {}", report.files_analyzed).unwrap();

        let mut sorted_tags: Vec<_> = report.tags.values().collect();
        sorted_tags.sort_by(|a, b| b.count.cmp(&a.count));

        writeln!(
            out,
            "{:<20} {:<10} {:<30} {:<10}",
            "TAG", "COUNT", "ATTRIBUTE", "ATTR COUNT"
        )
        .unwrap();
        writeln!(out, "{}", "-".repeat(70)).unwrap();

        for tag in sorted_tags {
            if tag.attributes.is_empty() {
                writeln!(
                    out,
                    "{:<20} {:<10} {:<30} {:<10}",
                    tag.name, tag.count, "-", "-"
                )
                .unwrap();
            } else {
                let mut sorted_attrs: Vec<_> = tag.attributes.values().collect();
                sorted_attrs.sort_by(|a, b| b.count.cmp(&a.count));

                for (i, attr) in sorted_attrs.iter().enumerate() {
                    if i == 0 {
                        writeln!(
                            out,
                            "{:<20} {:<10} {:<30} {:<10}",
                            tag.name, tag.count, attr.name, attr.count
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            out,
                            "{:<20} {:<10} {:<30} {:<10}",
                            "", "", attr.name, attr.count
                        )
                        .unwrap();
                    }
                }
            }
        }
        out
    }
}
