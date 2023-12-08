use std::{fs, io};
use std::path::Path;
use xml::reader::{XmlEvent, EventReader};
use std::collections::HashMap;

#[derive(Debug)]
struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) -> &'a [char] {
        while self.content.len() > 0 && self.content[0].is_ascii_whitespace() {
            self.content = &self.content[1..];
        }

        self.content
    }
    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();
        if self.content.len() == 0 {
            return None;
        }

        if self.content[0].is_alphabetic() {
            let mut i = 0;
            while i < self.content.len() && self.content[i].is_alphanumeric() {
                i += 1;
            }
            let token = &self.content[0..i];
            self.content = &self.content[i..];
            return Some(token);
        }

        let token = &self.content[0..1];
        self.content = &self.content[1..];
        return Some(token);
        todo!("Invalid token starts with {}", self.content[0]);
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn xml_to_string<P: AsRef<Path>>(_file_path: P) -> io::Result<String> {
    let file = fs::File::open(_file_path)?;
    let event_reader = EventReader::new(file);

    let mut content = String::new();
    for event in event_reader.into_iter() {
        if let XmlEvent::Characters(text) = event.expect("TODO") {
            content.push_str(&text);
            content.push(' ');
        }
    }
    Ok(content)
}

fn index_doc(_doc_ontent: &str) -> HashMap<String, usize> {
    todo!("not implemented");
}

fn main() {
    let file_path = "../docs.gl/gl4/glClear.xhtml";
    let content = xml_to_string(&file_path).unwrap()
        .chars()
        .collect::<Vec<_>>();

    let lexer = Lexer::new(&content);
    for token in lexer {
        println!("{}", token.iter().collect::<String>());
    }
    /*
    let all_docs: HashMap<Path, HashMap<String, usize>> = HashMap::new();

    let dir_path = "../docs.gl/gl4";
    let dir = fs::read_dir(dir_path).unwrap();
    for entry in dir {
        let file_path = entry.unwrap().path();
        let content = xml_to_string(&file_path).unwrap()
            .chars()
            .collect::<Vec<_>>();

        let lexer = Lexer::new(&content);
        println!("{file_path:?} -> {}", content.len());
    }
    */
}
