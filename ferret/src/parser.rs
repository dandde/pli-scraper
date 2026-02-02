use anyhow::Result;
use tl::{ParserOptions, VDom};

pub struct FerretParser;

impl FerretParser {
    pub fn parse(content: &str) -> Result<VDom<'_>> {
        let options = ParserOptions::default().track_ids();
        let vdom =
            tl::parse(content, options).map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;
        Ok(vdom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid() {
        let html = "<div><p>Hello</p></div>";
        let vdom = FerretParser::parse(html).expect("Failed to parse valid HTML");
        assert!(vdom.children().len() > 0);
    }
}
