use crate::egui::TextBuffer;
use regex::Regex;
use std::any::TypeId;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Section {
    heading: String,
    expanded: bool,
    level: usize,
    children: Vec<Box<dyn Node>>,
}

trait Node: Debug {
    fn len(&self, flatten: bool) -> usize;
    fn string(&self, flatten: bool) -> String;
    fn insert(&mut self, text: &str, pos: usize);
    fn translate(&mut self, pos: usize) -> usize;
    fn collapse(&mut self, path: &[usize]);
    fn expand(&mut self, path: &[usize]);
}

impl Node for String {
    fn len(&self, _: bool) -> usize {
        return self.len();
    }

    fn string(&self, _: bool) -> String {
        return self.to_string();
    }

    fn insert(&mut self, text: &str, pos: usize) {
        self.insert_str(pos, text);
    }

    fn translate(&mut self, pos: usize) -> usize {
        return pos;
    }

    fn collapse(&mut self, _: &[usize]) {
        panic!("cannot collapse string");
    }

    fn expand(&mut self, _: &[usize]) {
        panic!("cannot expand string");
    }
}

impl Default for Section {
    fn default() -> Self {
        Self {
            heading: String::new(),
            expanded: true,
            level: 0,
            children: Vec::new(),
        }
    }
}

impl Node for Section {
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
        /*if self.level > 0 {
            output += " ";
        }*/
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

    fn insert(&mut self, text: &str, pos: usize) {
        let mut cur = pos;
        if self.level > 0 {
            // TODO: handle reparse if it editing the heading marker
            if cur < self.level {
                return;
            }
            cur -= self.level;

            if cur < self.heading.len() {
                self.heading.insert_str(cur, text);
                return;
            }
            cur -= self.heading.len();
        }

        for n in &mut self.children {
            let len = n.len(false);
            if cur < len {
                n.insert(text, cur);
                return;
            }
            cur -= len;
        }
    }

    fn translate(&mut self, pos: usize) -> usize {
        let mut cur = 0;
        if self.level > 0 {
            cur += self.level + 1 + self.heading.len();
            if pos < cur {
                return pos;
            }
        }

        let mut offset = 0;
        if self.expanded {
            for n in &mut self.children {
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

    //fn delete(&mut self, range: std::ops::Range<usize>) {}

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
}

pub struct Note {
    internal: String,
    pub root: Section,
    repr: String,
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
                nodes.push(Box::new(text[..range.start].to_string()));
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
        nodes.push(Box::new(text));
        return nodes;
    }

    // parse the remainder of the file and stick the last heading on it
    //   TODO: I don't like having a second copy of this here
    nodes.push(Box::new(Section {
        heading: heading,
        expanded: true,
        level: level,
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
        self.internal = self.root.string(true);
        println!("translates to {}", self.root.translate(char_index));
        self.internal
            .insert_str(self.root.translate(char_index), text);
        println!("inserted to {}", self.internal);
        self.root.children = parse(self.internal.clone());
        self.repr = self.root.string(false);
        println!("flattened to {}", self.repr);
        // TODO: add editable flag to node items and return 0 if in a generated section
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
    use crate::note::{Node, Note, Section, parse};
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
    }

    #[test]
    fn test_note_insert() {
        let mut note = Note::new("# A\n\na\n".to_string());
        note.insert_text("#", 4);
        assert_eq!("# A\n#\na\n", note.as_str());
    }
}
