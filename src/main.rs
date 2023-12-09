use std::{env, fs};
use std::path::{Path, PathBuf};
use xml::reader::{XmlEvent, EventReader};
use std::collections::HashMap;
use std::fs::File;
use std::process::exit;
use xml::common::{Position, TextPosition};

type TermFreq = HashMap::<String, usize>;
type TermFreqIndex = HashMap::<PathBuf, TermFreq>;

struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) -> &'a [char] {
        while !self.content.is_empty() && self.content[0].is_ascii_whitespace() {
            self.content = &self.content[1..];
        }

        self.content
    }

    fn strip_left(&mut self, n: usize) -> &'a [char] {
        let token = &self.content[0..n];
        self.content = &self.content[n..];
        token
    }

    fn strip_left_while<P>(&mut self, mut predicate: P) -> &'a [char] where P: FnMut(&char) -> bool {
        let mut i = 0;
        while i < self.content.len() && predicate(&self.content[i]) {
            i += 1;
        }
        return self.strip_left(i);
    }

    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();

        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self.strip_left_while(|e| e.is_numeric() || e.is_ascii_punctuation()));
        }

        if self.content[0].is_alphabetic() {
            return Some(self.strip_left_while(|&e| e.is_alphanumeric()));
        }

        return Some(self.strip_left(1));
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn xml_to_string(_file_path: &Path) -> Option<String> {
    let file = File::open(_file_path).map_err(|err| {
        eprintln!("ERROR: could not open file {file_path}: {err}", file_path = _file_path.display());
    }).ok()?;

    let event_reader = EventReader::new(file);
    let mut content = String::new();

    for event in event_reader.into_iter() {
        let event = event.map_err(|err| {
            let TextPosition {row, column} = err.position();
            let msg = err.msg();
            eprintln!("{file_path}:{row}:{column}: ERROR: {msg}", file_path = _file_path.display());
        }).ok()?;

        if let XmlEvent::Characters(text) = event {
            content.push_str(&text);
            content.push(' ');
        }
    }
    Some(content)
}

fn index_doc(_doc_path: &Path) -> Option<TermFreq> {
    let content = match xml_to_string(&_doc_path) {
        Some(string) => string.chars().collect::<Vec<_>>(),
        None => return None,
    };

    let mut tf = TermFreq::new();

    let lexer = Lexer::new(&content);
    for token in lexer {
        let term = token.iter().map(|e| e.to_ascii_uppercase()).collect::<String>();
        if let Some(count) = tf.get_mut(&term) {
            *count += 1;
        }
        else {
            tf.insert(term, 1);
        }
    }

    return Some(tf);
}

fn index_all(_dir_path: &str) -> Option<TermFreqIndex> {
    let mut tfi = TermFreqIndex::new();
    let dir = fs::read_dir(_dir_path).map_err(|err| {
        eprintln!("ERROR: could not open directory {_dir_path}: {err}");
        return None::<TermFreqIndex>;
    }).unwrap();

    'next: for entry in dir {
        let file_path = entry.unwrap().path();
        println!("Indexing {file_path:?}...");

        let tf = match index_doc(&file_path) {
            Some(data) => data,
            None => continue 'next,
        };

        tfi.insert(file_path, tf);
    }

    Some(tfi)
}

fn hint(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands:");
    eprintln!("    index <folder>         index the <folder> and save the index to index.json file");
    eprintln!("    search <index-file>    check how many documents are indexed in the file (searching is not implemented yet)");
}

fn main() {
    let mut args = env::args();
    let program = args.next().expect("path to program is provided");

    let subcommand = args.next().unwrap_or_else(|| {
        eprintln!("ERROR: no subcommand is provided");
        hint(&program);
        exit(1);
    });

    match subcommand.as_str() {
        "index" => {
            let dir_path = args.next().unwrap_or_else(|| {
                eprintln!("ERROR: no directory provided for indexing");
                exit(1);
            });

            let tfi = index_all(&dir_path).unwrap();
            println!("{dir_path} contains {count} files", count = tfi.len());

            let index_path = "index.json";
            let index_file = File::create(index_path).map_err(|err| {
                eprintln!("ERROR: could not create index file {index_path}: {err}");
            }).unwrap();

            println!("Saving {index_path}...");
            serde_json::to_writer(index_file, &tfi).unwrap();
        },

        "search" => {
            let index_path = args.next().unwrap_or_else(|| {
                eprintln!("ERROR: no path to index is provided");
                exit(1);
            });

            let index_file = match File::open(&index_path) {
                Ok(file) => file,
                Err(_) => {
                    eprintln!("ERROR: invalid index file path");
                    exit(1);
                }
            };

            println!("Reading {index_path} index file...");
            let tfi: TermFreqIndex = serde_json::from_reader(index_file).unwrap();
            println!("{index_path} contains {count} files", count = tfi.len());
        },

        _ => {
            eprintln!("ERROR: unknown subcommand {subcommand}");
            exit(1);
        },
    }
}
