//! Minimal arena DOM used by the style/layout stages.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Document,
    Element,
    Text,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub kind: Kind,
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub text: String,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct Doc {
    pub nodes: Vec<Node>,
}

impl Default for Doc {
    fn default() -> Self {
        Self::new()
    }
}

impl Doc {
    pub fn new() -> Doc {
        Doc {
            nodes: vec![Node {
                kind: Kind::Document,
                tag: "#document".into(),
                attrs: vec![],
                text: String::new(),
                parent: None,
                children: vec![],
            }],
        }
    }

    pub fn add_element(&mut self, parent: usize, tag: &str, attrs: Vec<(String, String)>) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node {
            kind: Kind::Element,
            tag: tag.to_ascii_lowercase(),
            attrs,
            text: String::new(),
            parent: Some(parent),
            children: vec![],
        });
        self.nodes[parent].children.push(id);
        id
    }

    pub fn add_text(&mut self, parent: usize, text: &str) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node {
            kind: Kind::Text,
            tag: String::new(),
            attrs: vec![],
            text: text.to_string(),
            parent: Some(parent),
            children: vec![],
        });
        self.nodes[parent].children.push(id);
        id
    }

    pub fn is_element(&self, id: usize) -> bool {
        self.nodes[id].kind == Kind::Element
    }

    pub fn tag(&self, id: usize) -> &str {
        &self.nodes[id].tag
    }

    pub fn attr(&self, id: usize, name: &str) -> Option<&str> {
        self.nodes[id]
            .attrs
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn has_class(&self, id: usize, class: &str) -> bool {
        self.attr(id, "class")
            .map(|c| c.split_whitespace().any(|x| x == class))
            .unwrap_or(false)
    }

    /// Concatenated text of all descendant text nodes.
    pub fn text_content(&self, id: usize) -> String {
        let mut out = String::new();
        self.collect_text(id, &mut out);
        out
    }

    fn collect_text(&self, id: usize, out: &mut String) {
        for &c in &self.nodes[id].children {
            match self.nodes[c].kind {
                Kind::Text => out.push_str(&self.nodes[c].text),
                Kind::Element => self.collect_text(c, out),
                Kind::Document => {}
            }
        }
    }

    /// First element with the given tag, depth first.
    pub fn find_tag(&self, tag: &str) -> Option<usize> {
        (0..self.nodes.len()).find(|&i| self.is_element(i) && self.nodes[i].tag == tag)
    }
}
