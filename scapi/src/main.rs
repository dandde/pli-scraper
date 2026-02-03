use anyhow::Result;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::io::Read;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use ferret::analyzer::{AnalysisResult, Analyzer, StatsAnalyzer};
use ferret::exporter::{CsvExporter, Exporter, GraphVisualizerExporter, HtmlTreeExporter};
use ferret::parser::FerretParser;
use ferret::reporter::{FlatDisplay, TreeDisplay};
use ferret::walker::DomWalker;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Deserialize)]
struct ReportParams {
    format: Option<String>,
}

#[derive(Deserialize)]
struct ExportParams {
    format: Option<String>,
}

async fn handler_report(
    Path(target_url): Path<String>,
    Query(params): Query<ReportParams>,
) -> impl IntoResponse {
    // Reconstruct URL if needed (axum *path wildcard matches the rest of the path including slashes)
    // However, if the user passes `api/report/https://example.com`, `target_url` will be `https://example.com`.
    // Validating URL scheme.
    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid URL format. Expected http://... or https://...",
        )
            .into_response();
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    pb.set_message(format!("Fetching {}", target_url));
    let body_str = match reqwest::get(&target_url).await {
        Ok(resp) => match resp.text().await {
            Ok(text) => text,
            Err(e) => {
                pb.finish_with_message("Fetch failed");
                return (
                    StatusCode::BAD_REQUEST,
                    format!("Failed to read body: {}", e),
                )
                    .into_response();
            }
        },
        Err(err) => {
            pb.finish_with_message("Fetch failed");
            let code = err.status().map(|s| s.as_u16()).unwrap_or(400);
            return (
                StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_REQUEST),
                format!("Proxy error: {}", err),
            )
                .into_response();
        }
    };

    pb.set_message("Analyzing HTML...");
    // Ferret Analysis
    let analysis_result = match analyze_html(&body_str) {
        Ok(result) => {
            pb.finish_with_message(format!("Analysis complete for {}", target_url));
            result
        }
        Err(e) => {
            pb.finish_with_message("Analysis failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Analysis error: {}", e),
            )
                .into_response();
        }
    };

    match params.format.as_deref() {
        Some("tree") => {
            let report = TreeDisplay::render(&analysis_result);
            Response::builder()
                .header("Content-Type", "text/plain")
                .body(axum::body::Body::from(report))
                .unwrap()
                .into_response()
        }
        Some("flat") => {
            let report = FlatDisplay::render(&analysis_result);
            Response::builder()
                .header("Content-Type", "text/plain")
                .body(axum::body::Body::from(report))
                .unwrap()
                .into_response()
        }
        _ => Json(analysis_result).into_response(),
    }
}

async fn handler_export(
    Path(target_url): Path<String>,
    Query(params): Query<ExportParams>,
) -> impl IntoResponse {
    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        return (
            StatusCode::BAD_REQUEST,
            "Invalid URL format. Expected http://... or https://...",
        )
            .into_response();
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    pb.set_message(format!("Fetching {}", target_url));
    let body_str = match reqwest::get(&target_url).await {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(e) => return (StatusCode::BAD_REQUEST, format!("Fetch error: {}", e)).into_response(),
    };

    pb.set_message("Analyzing HTML...");
    let analysis_result = match analyze_html(&body_str) {
        Ok(res) => {
            pb.finish_with_message("Done");
            res
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Analysis error: {}", e),
            )
                .into_response()
        }
    };

    let (exporter, content_type, extension): (Box<dyn Exporter>, &str, &str) =
        match params.format.as_deref() {
            Some("csv") => (Box::new(CsvExporter), "text/csv", "csv"),
            Some("html") => (Box::new(HtmlTreeExporter), "text/html", "html"),
            Some("graph") => (Box::new(GraphVisualizerExporter), "text/html", "html"),
            _ => (Box::new(CsvExporter), "text/csv", "csv"),
        };

    match tempfile::NamedTempFile::new() {
        Ok(temp_file) => {
            let temp_path = temp_file.path().to_owned();

            if let Err(e) = exporter.export(&analysis_result, &temp_path) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Export error: {}", e),
                )
                    .into_response();
            }

            let mut content = String::new();
            if let Ok(mut file) = std::fs::File::open(&temp_path) {
                if let Err(e) = file.read_to_string(&mut content) {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("File read error: {}", e),
                    )
                        .into_response();
                }
            } else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to open temp file",
                )
                    .into_response();
            }

            Response::builder()
                .header("Content-Type", content_type)
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"report.{}\"", extension),
                )
                .body(axum::body::Body::from(content))
                .unwrap()
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create temp file: {}", e),
        )
            .into_response(),
    }
}

fn analyze_html(html: &str) -> Result<AnalysisResult> {
    let vdom = FerretParser::parse(html).map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;
    let walker = DomWalker::new(vdom.children().to_vec(), vdom.parser());
    let mut analyzer = StatsAnalyzer::new(10);

    for (_handle, node, depth) in walker {
        analyzer.visit(node, depth);
    }
    Ok(analyzer.result())
}

#[tokio::main]
async fn main() -> Result<()> {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port = port.parse::<u16>()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    println!("Ferret Axum Server listening on {}", addr);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/report/*url", get(handler_report))
        .route("/api/export/*url", get(handler_export))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_html_basic() {
        let html = r#"<html><body><h1>Hello</h1></body></html>"#;
        let result = analyze_html(html).expect("Analysis failed");
        assert!(result.tags.contains_key("h1"));
        assert_eq!(result.tags.get("h1").unwrap().count, 1);
    }
}
