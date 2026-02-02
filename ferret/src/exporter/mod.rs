use crate::analyzer::AnalysisResult;
use anyhow::Result;
use askama::Template;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub trait Exporter {
    fn export(&self, result: &AnalysisResult, path: &Path) -> Result<()>;
}

pub struct JsonExporter;

impl Exporter for JsonExporter {
    fn export(&self, result: &AnalysisResult, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        serde_json::to_writer_pretty(file, result)?;
        Ok(())
    }
}

pub struct CsvExporter;

impl Exporter for CsvExporter {
    fn export(&self, result: &AnalysisResult, path: &Path) -> Result<()> {
        let mut wtr = csv::Writer::from_path(path)?;

        // Write headers
        wtr.write_record(&[
            "Tag",
            "Count",
            "Attribute",
            "Attribute Count",
            "Value",
            "Value Count",
        ])?;

        // Flatten the nested structure for CSV
        for (tag_name, tag_stats) in &result.tags {
            // Write tag level info even if no attributes
            if tag_stats.attributes.is_empty() {
                wtr.write_record(&[tag_name, &tag_stats.count.to_string(), "", "", "", ""])?;
            } else {
                for (attr_name, attr_stats) in &tag_stats.attributes {
                    if attr_stats.value_counts.is_empty() {
                        wtr.write_record(&[
                            tag_name,
                            &tag_stats.count.to_string(),
                            attr_name,
                            &attr_stats.count.to_string(),
                            "",
                            "",
                        ])?;
                    } else {
                        for (val, val_count) in &attr_stats.value_counts {
                            wtr.write_record(&[
                                tag_name,
                                &tag_stats.count.to_string(),
                                attr_name,
                                &attr_stats.count.to_string(),
                                val,
                                &val_count.to_string(),
                            ])?;
                        }
                    }
                }
            }
        }

        wtr.flush()?;
        Ok(())
    }
}

pub struct HtmlTreeExporter;

impl Exporter for HtmlTreeExporter {
    fn export(&self, result: &AnalysisResult, path: &Path) -> Result<()> {
        let mut file = File::create(path)?;
        writeln!(file, "<!DOCTYPE html><html><head><style>")?;
        writeln!(file, "body {{ font-family: sans-serif; }}")?;
        writeln!(file, "ul {{ list-style-type: none; }}")?;
        writeln!(file, ".tag {{ color: #2c3e50; font-weight: bold; }}")?;
        writeln!(file, ".attr {{ color: #e67e22; }}")?;
        writeln!(file, ".val {{ color: #27ae60; }}")?;
        writeln!(file, ".count {{ color: #7f8c8d; font-size: 0.9em; }}")?;
        writeln!(file, "</style></head><body>")?;
        writeln!(file, "<h1>Analysis Report</h1>")?;
        writeln!(file, "<p>Files analyzed: {}</p>", result.files_analyzed)?;
        writeln!(file, "<ul>")?;

        let mut sorted_tags: Vec<_> = result.tags.values().collect();
        sorted_tags.sort_by(|a, b| b.count.cmp(&a.count));

        for tag in sorted_tags {
            writeln!(file, "<li><details><summary><span class='tag'>{}</span> <span class='count'>({})</span></summary>", tag.name, tag.count)?;

            if !tag.attributes.is_empty() {
                writeln!(file, "<ul>")?;
                let mut sorted_attrs: Vec<_> = tag.attributes.values().collect();
                sorted_attrs.sort_by(|a, b| b.count.cmp(&a.count));

                for attr in sorted_attrs {
                    writeln!(file, "<li><details><summary><span class='attr'>@{}</span> <span class='count'>({})</span></summary>", attr.name, attr.count)?;

                    if !attr.value_counts.is_empty() {
                        writeln!(file, "<ul>")?;
                        let mut sorted_vals: Vec<_> = attr.value_counts.iter().collect();
                        sorted_vals.sort_by(|a, b| b.1.cmp(a.1));

                        for (val, count) in sorted_vals.iter().take(10) {
                            writeln!(file, "<li><span class='val'>{}</span> <span class='count'>({})</span></li>", val, count)?;
                        }
                        writeln!(file, "</ul>")?;
                    }
                    writeln!(file, "</details></li>")?;
                }
                writeln!(file, "</ul>")?;
            }
            writeln!(file, "</details></li>")?;
        }

        writeln!(file, "</ul></body></html>")?;
        Ok(())
    }
}

#[derive(Template)]
#[template(path = "graph_visualizer.html")]
pub struct GraphVisualizerTemplate<'a> {
    pub data: &'a AnalysisResult,
}

// Helper filter for JSON serialization in templates
mod filters {
    use crate::analyzer::AnalysisResult;

    #[allow(dead_code)]
    pub fn json(data: &AnalysisResult) -> askama::Result<String> {
        serde_json::to_string(data).map_err(|e| askama::Error::Custom(Box::new(e)))
    }
}

pub struct GraphVisualizerExporter;

impl Exporter for GraphVisualizerExporter {
    fn export(&self, result: &AnalysisResult, path: &Path) -> Result<()> {
        let template = GraphVisualizerTemplate { data: result };
        let content = template.render()?;
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }
}
