use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::net::SocketAddr;

use ferret::analyzer::{Analyzer, StatsAnalyzer};
use ferret::parser::FerretParser;
use ferret::walker::DomWalker;
use indicatif::{ProgressBar, ProgressStyle};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path();

    // Extract and validate the target URL
    let target_url = match extract_target_url(path) {
        Ok(url) => url,
        Err(e) => {
            let mut res = Response::new(Body::from(e.to_string()));
            *res.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(res);
        }
    };

    println!("Proxying request to: {}", target_url);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    pb.set_message(format!("Fetching {}", target_url));
    let mut status = StatusCode::OK;
    let body_str = match async { reqwest::get(target_url).await?.text().await }.await {
        Ok(b) => b,
        Err(err) => {
            status = err.status().unwrap_or(StatusCode::BAD_REQUEST);
            format!("Proxy error: {}", err)
        }
    };

    if status != StatusCode::OK {
        pb.finish_with_message("Fetch failed");
        let mut res = Response::new(Body::from(body_str));
        *res.status_mut() = status;
        return Ok(res);
    }

    pb.set_message("Analyzing HTML...");
    // Ferret Analysis
    let response_body = match analyze_html(&body_str) {
        Ok(json) => {
            pb.finish_with_message(format!("Analysis complete for {}", target_url));
            json
        }
        Err(e) => {
            pb.finish_with_message("Analysis failed");
            status = StatusCode::INTERNAL_SERVER_ERROR;
            format!("Analysis error: {}", e)
        }
    };

    let mut res = Response::new(Body::from(response_body));
    *res.status_mut() = status;
    res.headers_mut().insert(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );

    // Add CORS headers to allow cross-origin requests
    res.headers_mut().insert(
        hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN,
        hyper::header::HeaderValue::from_static("*"),
    );
    res.headers_mut().insert(
        hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
        hyper::header::HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    res.headers_mut().insert(
        hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
        hyper::header::HeaderValue::from_static("*"),
    );

    Ok(res)
}

fn extract_target_url(path: &str) -> Result<&str, String> {
    // Remove leading slash and reconstruct the URL
    let target_url = if path.starts_with('/') {
        &path[1..]
    } else {
        path
    };

    // Validate that we have a proper URL
    if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
        return Err(format!(
            "Invalid URL format. Expected /http://... or /https://..., got: {}",
            path
        ));
    }

    Ok(target_url)
}

fn analyze_html(html: &str) -> Result<String> {
    let vdom = FerretParser::parse(html).map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;
    let walker = DomWalker::new(vdom.children().to_vec(), vdom.parser());
    let mut analyzer = StatsAnalyzer::new(10); // Find top 10 values for attributes

    for (_handle, node, depth) in walker {
        analyzer.visit(node, depth);
    }

    let result = analyzer.result();
    let json = serde_json::to_string_pretty(&result)?;
    Ok(json)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Check if there's an environment variable for the port
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    // Parse the port into a u16
    let port = port.parse::<u16>()?;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    println!("Ferret Analysis Server listening on {}", addr);
    println!("Usage: http://{}/<target-url>", addr);
    println!("Example: http://{}/https://example.com", addr);

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    // Then bind and serve...
    let server = Server::bind(&addr).serve(make_service);

    // And run forever...
    server.await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_target_url_valid() {
        assert_eq!(
            extract_target_url("/https://example.com"),
            Ok("https://example.com")
        );
        assert_eq!(
            extract_target_url("/http://example.com/foo"),
            Ok("http://example.com/foo")
        );
        assert_eq!(
            extract_target_url("https://example.com"),
            Ok("https://example.com")
        );
    }

    #[test]
    fn test_extract_target_url_invalid() {
        assert!(extract_target_url("/ftp://example.com").is_err());
        assert!(extract_target_url("invalid-url").is_err());
    }

    #[test]
    fn test_analyze_html_basic() {
        let html = r#"<html><body><h1>Hello</h1></body></html>"#;
        let json = analyze_html(html).expect("Analysis failed");

        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Invalid JSON");

        assert!(parsed.get("tags").is_some());
        let tags = parsed["tags"].as_object().unwrap();

        assert!(tags.contains_key("h1"));
        assert!(tags.contains_key("body"));
        assert_eq!(tags["h1"]["count"], 1);
    }
}
