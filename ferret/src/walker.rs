use std::collections::VecDeque;
use tl::{NodeHandle, Parser};

pub struct DomWalker<'a> {
    parser: &'a Parser<'a>,
    queue: VecDeque<(NodeHandle, usize)>,
}

impl<'a> DomWalker<'a> {
    pub fn new(roots: Vec<NodeHandle>, parser: &'a Parser<'a>) -> Self {
        let mut queue = VecDeque::new();
        // Initial roots
        for handle in roots {
            queue.push_back((handle, 0));
        }

        Self { parser, queue }
    }
}

impl<'a> Iterator for DomWalker<'a> {
    type Item = (NodeHandle, &'a tl::Node<'a>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (handle, depth) = self.queue.pop_front()?;
        // Safe unwrap because handle comes from same parser
        let node = handle.get(self.parser)?;

        // If node has children, push them to the FRONT (DFS)
        if let Some(children) = node.children() {
            // Collect to vector first because InlineVecIter doesn't support rev()
            let children_vec: Vec<_> = children.top().iter().collect();
            for child in children_vec.into_iter().rev() {
                self.queue.push_front((child.clone(), depth + 1));
            }
        }

        Some((handle, node, depth))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::FerretParser;

    #[test]
    fn test_traversal_order() {
        let html = "<div><p>Child</p></div>";
        let vdom = FerretParser::parse(html).unwrap();
        let walker = DomWalker::new(vdom.children().to_vec(), vdom.parser());

        let nodes: Vec<_> = walker
            .map(|(_h, n, depth)| (n.as_tag().map(|t| t.name().as_utf8_str()), depth))
            .collect();

        // Expected: div (0), p (1), Child (text node, 2)
        // Since we are iterating nodes, text is also a node.

        // Root children: div
        // div children: p
        // p children: "Child" (text)

        assert_eq!(nodes[0], (Some("div".into()), 0));
        assert_eq!(nodes[1], (Some("p".into()), 1));
        assert_eq!(nodes[2].1, 2); // Text node at depth 2
    }
}
