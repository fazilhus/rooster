use std::{env, fs};
use std::path::{Path, PathBuf};
use xml::reader::{XmlEvent, EventReader};
use std::collections::HashMap;
use std::fs::File;
use std::process::exit;
use tiny_http::{Header, Method, Request, Response};
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

    fn next_token(&mut self) -> Option<String> {
        self.trim_left();

        if self.content.is_empty() {
            return None;
        }

        if self.content[0].is_numeric() {
            return Some(self
                .strip_left_while(|e| e.is_numeric() || e.is_ascii_punctuation())
                .iter().collect());
        }

        if self.content[0].is_alphabetic() {
            return Some(self
                .strip_left_while(|&e| e.is_alphanumeric())
                .iter()
                .map(|e| e.to_ascii_uppercase())
                .collect());
        }

        return Some(self
            .strip_left(1)
            .iter().collect());
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

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
        if let Some(count) = tf.get_mut(&token) {
            *count += 1;
        }
        else {
            tf.insert(token, 1);
        }
    }

    return Some(tf);
}

fn index_all(_dir_path: &Path, tfi: &mut TermFreqIndex) -> Result<(), ()> {
    let dir = fs::read_dir(_dir_path).map_err(|err| {
        eprintln!("ERROR: could not open directory {dir_path}: {err}",
                  dir_path = _dir_path.display());
    }).unwrap();

    'next: for entry in dir {
        let entry = entry.map_err(|err| {
            eprintln!("ERROR: could not read file {dir_path}: {err}", dir_path = _dir_path.display());
        })?;

        let file_path = entry.path();
        println!("Indexing {file_path:?}...");

        let file_type = entry.file_type().map_err(|err| {
            eprintln!("ERROR: could not determine type of file {file_path}: {err}",
                      file_path = file_path.display());
        })?;

        if file_type.is_dir() {
            index_all(&file_path, tfi)?;
            continue 'next;
        }

        let tf = match index_doc(&file_path) {
            Some(data) => data,
            None => continue 'next,
        };

        tfi.insert(file_path, tf);
    }

    Ok(())
}

fn serve_static_file(request: Request, file_path: &str, content_type: &str) -> Result<(), ()> {
    println!("INFO: incoming request! method: {:?}, url: {:?}",
             request.method(),
             request.url());

    let file = File::open(file_path).map_err(|err| {
        eprintln!("ERROR: could not open {file_path}: {err}");
    })?;


    let content_type = Header::from_bytes(b"Content-Type", content_type.as_bytes())?;
    let response = Response::from_file(file).with_header(content_type);
    request.respond(response).map_err(|err| {
        eprintln!("ERROR: could not serve static file {file_path}: {err}");
    })?;

    Ok(())
}

fn serve_404(request: Request) -> Result<(), ()> {
    request.respond(Response::from_string("Error 404").with_status_code(404))
        .map_err(|err| {
            eprintln!("ERROR: could not respond to request: {err}");
        })
}

fn serve_request(mut request: Request) -> Result<(), ()> {
    match (request.method(), request.url()) {
        (Method::Get, "/") | (Method::Get, "/index.html") => {
            serve_static_file(request, "index.html", "text/html; charset=utf-8")
        },

        (Method::Get, "/index.js") => {
            serve_static_file(request, "index.js", "text/javascript; charset=utf-8")
        },

        (Method::Post, "/api/search") => {
            let mut buf = String::new();
            request.as_reader().read_to_string(&mut buf).map_err(|err| {
                eprintln!("ERROR: could not interpret body as utf-8: {err}");
            })?;
            let search_query: Vec<char> = buf.chars().collect();

            for token in Lexer::new(&search_query) {
                println!("{token:?}");
            }

            request.respond(Response::from_string("ok")).map_err(|err| {
                eprintln!("ERROR: {err}")
            })
        },

        _ => {
            serve_404(request)
        },
    }
}

fn hint(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands:");
    eprintln!("    index <folder>         index the <folder> and save the index to index.json file");
    eprintln!("    search <index-file>    check how many documents are indexed in the file (searching is not implemented yet)");
    eprintln!("    serve [address]        start local HTTP server");
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

            let mut tfi = TermFreqIndex::new();
            index_all(Path::new(&dir_path), &mut tfi).unwrap();
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

        "serve" => {
            let address = args.next().unwrap_or("127.0.0.1:8000".to_string());
            let server = tiny_http::Server::http(&address).map_err(|err| {
                eprintln!("ERROR: could not start server: {err}");
                exit(1);
            }).unwrap();

            println!("Listening at http://{address}");
            for request in server.incoming_requests() {
                serve_request(request).unwrap();
            }
            todo!();
        },

        _ => {
            eprintln!("ERROR: unknown subcommand {subcommand}");
            exit(1);
        },
    }
}
