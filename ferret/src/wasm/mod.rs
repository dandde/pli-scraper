use crate::analyzer::{AnalysisResult, Analyzer, StatsAnalyzer};
use crate::parser::FerretParser;
use crate::walker::DomWalker;
use serde::Serialize;
use tl::VDom;
use wasm_bindgen::prelude::*;

mod renderer;
use renderer::{render_html_tree_string, render_tree_string};

#[wasm_bindgen]
pub struct FerretSession {
    // Pointers for manual drops
    input_ptr: *mut str,
    vdom_ptr: *mut VDom<'static>,

    // The active components
    walker: Option<DomWalker<'static>>,
    analyzer: StatsAnalyzer,

    // State
    is_complete: bool,
}

#[wasm_bindgen]
impl FerretSession {
    #[wasm_bindgen(constructor)]
    pub fn new(html: String) -> FerretSession {
        // 1. Leak the input string to get a 'static reference
        let input_boxed = html.into_boxed_str();
        let input_ptr = Box::into_raw(input_boxed);
        let input_ref = unsafe { &*input_ptr };

        // 2. Parse the HTML (produces VDom<'static>)
        let vdom = tl::parse(input_ref, tl::ParserOptions::default()).unwrap(); // TODO: Handle error better

        // 3. Leak the VDom to get a 'static reference for the walker
        let vdom_boxed = Box::new(vdom);
        let vdom_ptr = Box::into_raw(vdom_boxed);
        let vdom_ref = unsafe { &*vdom_ptr };

        // 4. Initialize Walker and Analyzer
        let roots = vdom_ref.children().to_vec();
        let walker = DomWalker::new(roots, vdom_ref.parser());
        let analyzer = StatsAnalyzer::new(5);

        FerretSession {
            input_ptr,
            vdom_ptr,
            walker: Some(walker),
            analyzer,
            is_complete: false,
        }
    }

    pub fn step(&mut self, chunk_size: usize) -> bool {
        if self.is_complete || self.walker.is_none() {
            return false;
        }

        let walker = self.walker.as_mut().unwrap();
        let mut processed = 0;

        while processed < chunk_size {
            if let Some((_handle, node, depth)) = walker.next() {
                self.analyzer.visit(node, depth);
                processed += 1;
            } else {
                self.is_complete = true;
                return false; // No more work
            }
        }

        true // More work available
    }

    pub fn get_result(&self) -> JsValue {
        let mut res = self.analyzer.result();
        res.files_analyzed = if self.is_complete { 1 } else { 0 }; // Partial?
        let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
        res.serialize(&serializer).unwrap_or(JsValue::NULL)
    }
}

impl Drop for FerretSession {
    fn drop(&mut self) {
        // Drop walker first as it borrows from vdom
        self.walker = None;

        // Reconstruct and drop VDom
        if !self.vdom_ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(self.vdom_ptr);
            }
        }

        // Reconstruct and drop input string
        if !self.input_ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(self.input_ptr);
            }
        }
    }
}

#[derive(Serialize)]
pub struct WasmAnalysisResult {
    pub data: AnalysisResult,
    pub tree_view: String,
    pub html_tree: String,
}

#[wasm_bindgen]
pub fn analyze_html(content: &str) -> JsValue {
    use serde::Serialize;
    let mut result = AnalysisResult::default();
    let vdom = match FerretParser::parse(content) {
        Ok(v) => v,
        Err(_) => {
            // Return empty structure on error
            return serde_wasm_bindgen::to_value(&WasmAnalysisResult {
                data: result,
                tree_view: String::new(),
                html_tree: String::new(),
            })
            .unwrap();
        }
    };
    let roots = vdom.children().to_vec();
    let walker = DomWalker::new(roots, vdom.parser());
    let mut analyzer = StatsAnalyzer::new(5);
    for (_handle, node, depth) in walker {
        analyzer.visit(node, depth);
    }
    result = analyzer.result();
    result.files_analyzed = 1;

    let wasm_result = WasmAnalysisResult {
        tree_view: render_tree_string(&result),
        html_tree: render_html_tree_string(&result),
        data: result,
    };

    let serializer = serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true);
    wasm_result.serialize(&serializer).unwrap()
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn run_legacy_poc() -> Result<(), JsValue> {
    use web_sys::window;

    let window = window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // 1. Create a Header
    let header = document.create_element("h1")?;
    header.set_text_content(Some("Ferret WASM POC (Legacy)"));
    body.append_child(&header)?;

    // 2. Create Status Div
    let status_div = document.create_element("div")?;
    status_div.set_id("status");
    status_div.set_text_content(Some("Initializing..."));
    body.append_child(&status_div)?;

    // 3. Run Analysis on Hardcoded Sample
    let html_sample = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test</title></head>
        <body>
            <div id="main" class="container">
                <h1>Hello WASM</h1>
                <p>This is analyzed by Ferret running in the browser.</p>
                <ul>
                    <li>Item 1</li>
                    <li>Item 2</li>
                </ul>
            </div>
        </body>
        </html>
    "#;

    status_div.set_text_content(Some("Analyzing sample HTML..."));

    let mut session = FerretSession::new(html_sample.to_string());

    // Run all steps synchronously for POC
    while session.step(100) {}

    let result = session.get_result();

    // 4. Display Result
    let result_pre = document.create_element("pre")?;
    result_pre.set_attribute(
        "style",
        "background: #f0f0f0; padding: 1em; border-radius: 4px; overflow: auto;",
    )?;

    // Convert JsValue (Object) to JSON string
    let json_str = js_sys::JSON::stringify_with_replacer_and_space(
        &result,
        &JsValue::NULL,
        &JsValue::from(2),
    )?;
    let json_text: String = json_str.into();

    result_pre.set_text_content(Some(&json_text));
    body.append_child(&result_pre)?;

    status_div.set_text_content(Some("Analysis Complete."));
    status_div.set_attribute("style", "color: green; font-weight: bold;")?;

    Ok(())
}
