use crate::egui::TextBuffer;
use regex::Regex;
use std::any::TypeId;
use std::fmt::Debug;

#[derive(Clone, Debug, PartialEq)]
pub enum MarkdownType {
    None,
    Heading,
    Paragraph,
    Bold,
    Italic,
    Link,
}

#[derive(Clone, Debug, PartialEq)]
enum NodeType {
    MarkdownString,
    Section,
}

#[derive(Debug)]
pub struct Section {
    heading: String,
    expanded: bool,
    level: usize,
    mdtype: MarkdownType,
    children: Vec<Box<dyn Node>>,
}

#[derive(Clone, Debug)]
pub struct MarkdownString {
    pub text: String,
    pub mdtype: MarkdownType,
}

impl MarkdownString {
    pub fn new(content: String) -> Self {
        return Self {
            text: content,
            mdtype: MarkdownType::Paragraph,
        };
    }
}

trait Node: Debug {
    fn type_id(&self) -> NodeType;
    fn md_type(&self) -> MarkdownType;
    fn len(&self, flatten: bool) -> usize;
    fn string(&self, flatten: bool) -> String;
    fn insert(&mut self, text: &str, pos: usize) -> bool;
    fn translate(&self, pos: usize) -> usize;
    fn inv_translate(&self, pos: usize) -> usize;
    fn toggle(&mut self, path: &[usize]);
    fn collapse(&mut self, path: &[usize]);
    fn expand(&mut self, path: &[usize]);
    fn path(&self, pos: usize) -> Vec<usize>;
    fn markdown(&self) -> Vec<MarkdownString>;
    fn get_node(&self, pos: usize) -> MarkdownString;
}

impl Node for MarkdownString {
    fn type_id(&self) -> NodeType {
        NodeType::MarkdownString
    }

    fn md_type(&self) -> MarkdownType {
        self.mdtype.clone()
    }

    fn len(&self, _: bool) -> usize {
        return self.text.len();
    }

    fn string(&self, _: bool) -> String {
        return self.text.to_string();
    }

    fn insert(&mut self, text: &str, pos: usize) -> bool {
        self.text.insert_str(pos, text);
        return true;
    }

    fn translate(&self, pos: usize) -> usize {
        return pos;
    }

    fn inv_translate(&self, pos: usize) -> usize {
        return pos;
    }

    fn toggle(&mut self, _: &[usize]) {
        panic!("cannot toggle string");
    }

    fn collapse(&mut self, _: &[usize]) {
        panic!("cannot collapse string");
    }

    fn expand(&mut self, _: &[usize]) {
        panic!("cannot expand string");
    }

    fn path(&self, _: usize) -> Vec<usize> {
        return Vec::<usize>::new();
    }

    fn markdown(&self) -> Vec<MarkdownString> {
        return vec![self.clone()];
    }

    fn get_node(&self, _: usize) -> MarkdownString {
        return self.clone();
    }
}

impl Default for Section {
    fn default() -> Self {
        Self {
            heading: String::new(),
            expanded: true,
            level: 0,
            mdtype: MarkdownType::None,
            children: Vec::new(),
        }
    }
}

impl Node for Section {
    fn type_id(&self) -> NodeType {
        NodeType::Section
    }

    fn md_type(&self) -> MarkdownType {
        self.mdtype.clone()
    }

    fn len(&self, flatten: bool) -> usize {
        let mut length = 0;
        if self.level > 0 {
            length = self.level /*+ 1*/ + self.heading.len();
        }
        if self.expanded || flatten {
            for n in &self.children {
                length += n.len(flatten);
            }
        }
        return length;
    }

    fn string(&self, full: bool) -> String {
        let mut output = "".to_string();
        for _ in 0..self.level {
            output += "#";
        }
        output += self.heading.as_str();
        if full || self.expanded {
            for node in &self.children {
                output += node.string(full).as_str();
            }
        }
        return output;
    }

    /*fn convert(&self, pos: usize) {
        let mut cur = 0;

    }*/

    fn insert(&mut self, text: &str, pos: usize) -> bool {
        let mut cur = pos;
        if self.level > 0 {
            // TODO: handle reparse if it editing the heading marker
            if cur < self.level {
                return false;
            }
            cur -= self.level;

            if cur < self.heading.len() {
                self.heading.insert_str(cur, text);
                return true;
            }
            cur -= self.heading.len();
        }

        for n in &mut self.children {
            let len = n.len(false);
            if cur < len {
                if !n.insert(text, cur) {
                    return false;
                }
                return true;
            }
            cur -= len;
        }
        return false;
    }

    fn translate(&self, pos: usize) -> usize {
        let mut cur = 0;
        if self.level > 0 {
            cur += self.level + 1 + self.heading.len();
            if pos < cur {
                return pos;
            }
        }

        let mut offset = 0;
        if self.expanded {
            for n in &self.children {
                let display_len = n.len(false);
                if pos - cur < display_len {
                    return n.translate(pos - cur) + cur + offset;
                }
                cur += display_len;
                offset += n.len(true) - display_len;
            }
        }
        return pos + offset;
    }

    fn inv_translate(&self, pos: usize) -> usize {
        let mut cur = 0;
        if self.level > 0 {
            cur += self.level + self.heading.len();
            if pos < cur {
                return pos;
            }
        }

        // just assume we're inside this section since the parent wouldn't have called if we weren't
        if !self.expanded {
            // heading text contains a newline, remove this if that changes
            return cur - 1;
        }

        let mut offset = 0;
        for n in &self.children {
            let full_len = n.len(true);
            if pos - cur < full_len {
                return n.inv_translate(pos - cur) + cur - offset;
            }
            cur += full_len;
            offset += full_len - n.len(false);
        }

        return cur - offset;
    }

    //fn delete(&mut self, range: std::ops::Range<usize>) {}

    fn toggle(&mut self, path: &[usize]) {
        if path.len() == 0 {
            if self.level == 0 {
                return;
            }
            self.expanded = !self.expanded;
        } else {
            self.children[path[0]].toggle(&path[1..]);
        }
    }

    fn collapse(&mut self, path: &[usize]) {
        if path.len() == 0 {
            self.expanded = false;
        } else {
            self.children[path[0]].collapse(&path[1..]);
        }
    }

    fn expand(&mut self, path: &[usize]) {
        if path.len() == 0 {
            self.expanded = true;
        } else {
            self.children[path[0]].expand(&path[1..]);
        }
    }

    fn path(&self, pos: usize) -> Vec<usize> {
        let mut cur = self.level + self.heading.len();
        if pos < cur {
            return Vec::<usize>::new();
        }

        for (i, n) in self.children.iter().enumerate() {
            cur += n.len(false);
            if pos <= cur {
                if n.type_id() == NodeType::MarkdownString {
                    return Vec::<usize>::new();
                }
                let mut tmp = n.path(pos);
                tmp.insert(0, i);
                return tmp;
            }
        }
        return Vec::<usize>::new();
    }

    fn markdown(&self) -> Vec<MarkdownString> {
        let mut hstring = "".to_string();
        for _ in 0..self.level {
            hstring += "#";
        }
        hstring += &self.heading;

        let mut md = vec![MarkdownString {
            text: hstring,
            mdtype: MarkdownType::Heading,
        }];

        if self.expanded {
            for c in &self.children {
                md.append(&mut c.markdown());
            }
        }

        return md;
    }

    fn get_node(&self, pos: usize) -> MarkdownString {
        let mut cur = self.level + self.heading.len();
        if pos < cur {
            let mut hstring = "".to_string();
            for _ in 0..self.level {
                hstring += "#";
            }
            hstring += &self.heading;

            return MarkdownString {
                text: hstring,
                mdtype: self.md_type(),
            };
        }

        for n in &self.children {
            let len = n.len(false);
            if pos <= cur + len {
                return n.get_node(pos - cur);
            }
            cur += len;
        }
        return MarkdownString {
            text: "".to_string(),
            mdtype: MarkdownType::None,
        };
    }
}

#[derive(Debug)]
pub struct Note {
    internal: String,
    pub root: Section,
    repr: String,
}

fn parse_strings(text: String) -> Vec<Box<dyn Node>> {
    let mut output: Vec<Box<dyn Node>> = vec![];
    // TODO: handle the different types right
    let regexes: [(Regex, MarkdownType); 3] = [
        (Regex::new(r"\*\*[^\*]+\*\*").unwrap(), MarkdownType::Bold),
        (Regex::new(r"_[^_]+_").unwrap(), MarkdownType::Italic),
        (
            Regex::new(r"@@([\\/A-Za-z0-9_-]+)").unwrap(),
            MarkdownType::Link,
        ),
    ];

    let mut lines = text.split('\n').peekable();

    while let Some(line) = lines.next() {
        let is_last = lines.peek().is_none();
        let mut t = String::from(line);
        if !is_last {
            t += "\n";
        }

        let mut first_match: Option<((usize, usize), MarkdownType)> = None;
        let mut rerun = true;
        while rerun {
            rerun = false;
            first_match = None;
            for r in &regexes {
                if let Some(mat) = r.0.find(t.as_str()) {
                    let range = mat.range();

                    // give up early if there was a match before this
                    if let Some(first) = &first_match
                        && first.0.0 < range.start
                    {
                        continue;
                    }

                    first_match = Some(((range.start, range.end), r.1.clone()));
                }
            }

            if let Some(first) = &first_match {
                if first.0.0 > 0 {
                    output.push(Box::new(MarkdownString {
                        text: t[..first.0.0].to_string(),
                        mdtype: MarkdownType::Paragraph,
                    }));
                }

                output.push(Box::new(MarkdownString {
                    text: t[first.0.0..first.0.1].to_string(),
                    mdtype: first.1.clone(),
                }));
                t = t[first.0.1..].to_string();
                rerun = true;
            }
        }

        if t.len() > 0 {
            output.push(Box::new(MarkdownString {
                text: t,
                mdtype: MarkdownType::Paragraph,
            }));
        }
    }
    return output;
}

fn parse(text: String) -> Vec<Box<dyn Node>> {
    let mut nodes: Vec<Box<dyn Node>> = Vec::new();

    let mut level = 0;
    let mut pos = 0;
    let mut heading = "".to_string();

    let re = Regex::new(r"(?m)^(#+)([^\n]+)$").unwrap();
    for caps in re.captures_iter(text.as_str()) {
        // initialize the level if this is the first heading we've encountered
        if level == 0 {
            let cap = caps.get(1).unwrap();
            let range = cap.range();

            level = cap.len();
            pos = caps.get(2).unwrap().range().end;
            heading = caps.get(2).unwrap().as_str().to_string();
            if text.len() > pos && &text[pos..pos + 1] == "\n" {
                heading += "\n";
                pos += 1;
            }

            if range.start > 0 {
                nodes.extend(parse_strings(text[..range.start].to_string()));
            }

            continue;
        } else if caps.get(1).unwrap().len() > level {
            continue;
        }

        let range = caps.get(0).unwrap().range();
        nodes.push(Box::new(Section {
            heading: heading,
            expanded: true,
            level: level,
            mdtype: MarkdownType::Heading,
            children: parse(text[pos..range.start].to_string()),
        }));
        heading = caps.get(2).unwrap().as_str().to_string();
        pos = range.end;
        level = caps.get(1).unwrap().len();
        if text.len() > pos && &text[pos..pos + 1] == "\n" {
            heading += "\n";
            pos += 1;
        }
    }

    if level == 0 {
        nodes.extend(parse_strings(text));
        //nodes.push(Box::new(MarkdownString::new(text)));
        return nodes;
    }

    // parse the remainder of the file and stick the last heading on it
    //   TODO: I don't like having a second copy of this here
    nodes.push(Box::new(Section {
        heading: heading,
        expanded: true,
        level: level,
        mdtype: MarkdownType::Heading,
        children: parse(text[pos..].to_string()),
    }));

    return nodes;
}

impl Note {
    pub fn new(content: String) -> Self {
        let mut tmp = Self {
            internal: content.clone(),
            root: Section::default(),
            repr: "".to_string(),
        };
        tmp.root.children = parse(content.clone());
        tmp.repr = content;
        return tmp;
    }

    pub fn full(&self) -> &str {
        return self.internal.as_str();
    }

    pub fn refresh(&mut self) {
        self.repr = self.root.string(false);
    }

    pub fn path(&self, pos: usize) -> Vec<usize> {
        println!("{:?}", self.root);
        return self.root.path(pos);
    }

    pub fn toggle(&mut self, path: &[usize]) {
        self.root.toggle(path);
    }

    pub fn markdown(&self) -> Vec<MarkdownString> {
        self.root.markdown()
    }

    pub fn get_node(&self, pos: usize) -> MarkdownString {
        self.root.get_node(pos)
    }

    pub fn translate(&self, pos: usize) -> usize {
        self.root.translate(pos)
    }

    pub fn inv_translate(&self, pos: usize) -> usize {
        self.root.inv_translate(pos)
    }
}

impl Default for Note {
    fn default() -> Self {
        Self {
            internal: "".to_string(),
            root: Section::default(),
            repr: "".to_string(),
        }
    }
}

impl TextBuffer for Note {
    fn is_mutable(&self) -> bool {
        // TODO: once I add the backlinks/table of contents generation:
        //   consider the cursor location and disable mutable when not in actual file text
        return true;
    }
    fn as_str(&self) -> &str {
        return self.repr.as_str();
    }
    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        // TODO: add editable flag to node items and return 0 if in a generated section
        // try for a fast insert first
        if !self.root.insert(text, char_index) {
            // do a full render and re-parse if not
            self.internal = self.root.string(true);
            self.internal
                .insert_str(self.root.translate(char_index), text);
            self.root.children = parse(self.internal.clone());
        }
        self.repr = self.root.string(false);
        return text.len();
    }
    fn delete_char_range(&mut self, char_range: std::ops::Range<usize>) {
        // TODO: navigate the sections to find the right area to mess with
        //   re-parse file when crossing section boundaries
        self.internal = self.root.string(true);
        self.internal.drain(std::ops::Range {
            start: self.root.translate(char_range.start),
            end: self.root.translate(char_range.end),
        });
        self.root.children = parse(self.internal.clone());
        self.repr = self.root.string(false);
    }

    // Implement it like the following:
    fn type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use crate::note::{MarkdownType, Node, Note, Section, parse};
    use eframe::egui::TextBuffer;

    #[test]
    fn test_parse() {
        let mut sec = Section::default();
        let mut example = "# Big Head\n## Little Head\nSome body\nMore Body##Second Little Head\none body\n# Another Big One\nend";
        sec.children = parse(example.to_string());
        println!("{:?}", sec);
        assert_eq!(example, sec.string(true));

        example = "not starting with a heading\n# Now Heading\nasdfasdf\nasdf\n";
        sec.children = parse(example.to_string());
        assert_eq!(example, sec.string(true));

        example = "# A\n## B\n### C";
        sec.children = parse(example.to_string());
        println!("{:?}", sec);
        assert_eq!(example, sec.string(true));

        example = "# A\n### B\n## C";
        sec.children = parse(example.to_string());
        println!("{:?}", sec);
        assert_eq!(example, sec.string(true));

        example = "# A";
        sec.children = parse(example.to_string());
        println!("{:?}", sec);
        assert_eq!(example, sec.string(true));

        example = "# A\n#\na\n";
        sec.children = parse(example.to_string());
        println!("{:?}", sec);
        assert_eq!(example, sec.string(true));
    }

    #[test]
    fn test_expand() {
        let mut sec = Section::default();
        let example = "# A\n## B\nbbbbb\n## C\nccccc";
        sec.children = parse(example.to_string());

        sec.collapse(&[0usize]);
        assert_eq!("# A\n", sec.string(false));
        assert_eq!(4, sec.len(false));

        sec.expand(&[0usize]);
        sec.collapse(&[0usize, 1usize]);
        assert_eq!("# A\n## B\nbbbbb\n## C\n", sec.string(false));
        assert_eq!(20, sec.len(false));
    }

    #[test]
    fn test_insert() {
        let mut sec = Section::default();
        let example = "# A\n## B\nbbbbb\n## C\nccccc";
        sec.children = parse(example.to_string());
        sec.collapse(&[0usize, 0usize]);
        println!("{}", sec.string(false));
        sec.insert("d", 15);
        assert_eq!("# A\n## B\n## C\ncdcccc", sec.string(false));
    }

    #[test]
    fn test_translate() {
        let mut sec = Section::default();
        let example = "# A\n## B\nbbbbb\n## C\nccccc";
        sec.children = parse(example.to_string());
        println!("{}", sec.string(false));

        assert_eq!(1, sec.translate(1));
        assert_eq!(2, sec.translate(2));
        assert_eq!(4, sec.translate(4));
        println!("testing 10");
        assert_eq!(10, sec.translate(10));
        assert_eq!(example.len(), sec.translate(example.len()));

        sec.collapse(&[0usize, 0usize]);
        println!("{}", sec.string(false));
        println!("testing 10, folded");
        assert_eq!(16, sec.translate(10));
        assert_eq!(8, sec.inv_translate(11));
        assert_eq!(11, sec.inv_translate(17));
    }

    #[test]
    fn test_path() {
        let mut sec = Section::default();
        let example = "# A\n## B\nbbbbb\n## C\nccccc";
        sec.children = parse(example.to_string());

        assert_eq!([0usize, 1usize], sec.path(20).iter().as_slice());
    }

    #[test]
    fn test_note_insert() {
        let mut note = Note::new("# A\n\na\n".to_string());
        note.insert_text("#", 4);
        assert_eq!("# A\n#\na\n", note.as_str());
    }

    #[test]
    fn test_markdown() {
        let mut sec = Section::default();
        let example = "# A\n## B\nbbbbb\n## C\nccccc";
        sec.children = parse(example.to_string());

        let md = sec.markdown();
        assert_eq!(MarkdownType::Heading, md[0].mdtype);
        assert_eq!(MarkdownType::Heading, md[1].mdtype);
        assert_eq!(MarkdownType::Heading, md[2].mdtype);
        assert_eq!(MarkdownType::Paragraph, md[3].mdtype);
        assert_eq!(MarkdownType::Heading, md[4].mdtype);
        assert_eq!(MarkdownType::Paragraph, md[5].mdtype);
    }
}
