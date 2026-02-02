use crate::analyzer::AnalysisResult;

pub fn render_tree_string(report: &AnalysisResult) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    writeln!(out, "📦 Files analyzed: {}", report.files_analyzed).unwrap();

    let mut sorted_tags: Vec<_> = report.tags.values().collect();
    sorted_tags.sort_by(|a, b| b.count.cmp(&a.count));

    for (i, tag) in sorted_tags.iter().enumerate() {
        let is_last_tag = i == sorted_tags.len() - 1;
        let tag_prefix = if is_last_tag {
            "└── "
        } else {
            "├── "
        };

        writeln!(out, "{}{} ({})", tag_prefix, tag.name, tag.count).unwrap();

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
                format!(" ({})", attr.count)
            )
            .unwrap();

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
                    full_val_indent, val_prefix, "──", val, count
                )
                .unwrap();
            }
        }
    }
    out
}

pub fn render_html_tree_string(report: &AnalysisResult) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    writeln!(out, "<ul class='tree'>").unwrap();

    let mut sorted_tags: Vec<_> = report.tags.values().collect();
    sorted_tags.sort_by(|a, b| b.count.cmp(&a.count));

    for tag in sorted_tags {
        writeln!(
            out,
            "<li><details open><summary><span class='tag'>{}</span> <span class='count'>({})</span></summary>",
            tag.name, tag.count
        )
        .unwrap();

        if !tag.attributes.is_empty() {
            writeln!(out, "<ul>").unwrap();
            let mut sorted_attrs: Vec<_> = tag.attributes.values().collect();
            sorted_attrs.sort_by(|a, b| b.count.cmp(&a.count));

            for attr in sorted_attrs {
                writeln!(
                    out,
                    "<li><details><summary><span class='attr'>@{}</span> <span class='count'>({})</span></summary>",
                    attr.name, attr.count
                )
                .unwrap();

                if !attr.value_counts.is_empty() {
                    writeln!(out, "<ul>").unwrap();
                    let mut sorted_vals: Vec<_> = attr.value_counts.iter().collect();
                    sorted_vals.sort_by(|a, b| b.1.cmp(a.1));

                    for (val, count) in sorted_vals.iter().take(10) {
                        writeln!(
                            out,
                            "<li><span class='val'>{}</span> <span class='count'>({})</span></li>",
                            val, count
                        )
                        .unwrap();
                    }
                    writeln!(out, "</ul>").unwrap();
                }
                writeln!(out, "</details></li>").unwrap();
            }
            writeln!(out, "</ul>").unwrap();
        }
        writeln!(out, "</details></li>").unwrap();
    }

    writeln!(out, "</ul>").unwrap();
    out
}
