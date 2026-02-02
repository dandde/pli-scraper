use crate::analyzer::AnalysisResult;
use colored::*;

pub struct TreeDisplay;

impl TreeDisplay {
    pub fn render(report: &AnalysisResult) {
        println!("📦 Files analyzed: {}", report.files_analyzed);

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

            println!(
                "{}{}{}",
                tag_prefix,
                tag.name.bright_cyan(),
                format!(" ({})", tag.count).yellow()
            );

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

                println!(
                    "{}{}@{}{}",
                    child_indent,
                    attr_prefix,
                    attr.name,
                    format!(" ({})", attr.count).dimmed()
                );

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
                    }; // Simple tree, might need adjusting logic
                       // The user example shows `├── ── val` logic.
                       // Copying user's style:
                       // ├── @class
                       // │   ├── ── container

                    println!(
                        "{}{}{} {} ({})",
                        full_val_indent,
                        val_prefix,
                        "──".dimmed(), // User's style extra dash
                        val,
                        count
                    );
                }
            }
        }
    }
}

pub struct FlatDisplay;

impl FlatDisplay {
    pub fn render(report: &AnalysisResult) {
        println!("📦 Files analyzed: {}", report.files_analyzed);

        let mut sorted_tags: Vec<_> = report.tags.values().collect();
        sorted_tags.sort_by(|a, b| b.count.cmp(&a.count));

        println!(
            "{:<20} {:<10} {:<30} {:<10}",
            "TAG", "COUNT", "ATTRIBUTE", "ATTR COUNT"
        );
        println!("{}", "-".repeat(70));

        for tag in sorted_tags {
            if tag.attributes.is_empty() {
                println!("{:<20} {:<10} {:<30} {:<10}", tag.name, tag.count, "-", "-");
            } else {
                let mut sorted_attrs: Vec<_> = tag.attributes.values().collect();
                sorted_attrs.sort_by(|a, b| b.count.cmp(&a.count));

                for (i, attr) in sorted_attrs.iter().enumerate() {
                    if i == 0 {
                        println!(
                            "{:<20} {:<10} {:<30} {:<10}",
                            tag.name, tag.count, attr.name, attr.count
                        );
                    } else {
                        println!("{:<20} {:<10} {:<30} {:<10}", "", "", attr.name, attr.count);
                    }
                }
            }
        }
    }
}
