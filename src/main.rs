use std::{env, fs};
use std::path::{Path};
use xml::reader::{XmlEvent, EventReader};
use std::fs::File;
use std::process::exit;
use xml::common::{Position, TextPosition};

mod model;
use model::*;

mod server;

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

fn hint(program: &str) {
    eprintln!("Usage: {program} [SUBCOMMAND] [OPTIONS]");
    eprintln!("Subcommands:");
    eprintln!("    index <folder>                index the <folder> and save the index to index.json file");
    eprintln!("    search <index-file> <query>   check how many documents are indexed in the file (searching is not implemented yet)");
    eprintln!("    serve <index-file> [address]  start local HTTP server");
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
                hint(&program);
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
                hint(&program);
                exit(1);
            });

            let prompt = args.next().unwrap_or_else(|| {
                eprintln!("ERROR: no prompt is provided");
                hint(&program);
                exit(1);
            }).chars().collect::<Vec<_>>();

            let index_file = File::open(&index_path).map_err(|err| {
                eprintln!("ERROR: could not open index file {index_path}: {err}");
                exit(1);
            }).unwrap();

            let tfi: TermFreqIndex = serde_json::from_reader(index_file).map_err(|err| {
                eprintln!("ERROR: could not parse index file {index_path}: {err}");
                exit(1);
            }).unwrap();

            for (path, rank) in search_query(&prompt, &tfi).iter().take(10) {
                println!("{path:?} {rank}");
            }
        },

        "serve" => {
            let index_path = args.next().unwrap_or_else(|| {
                eprintln!("ERROR: no path to index is provided");
                hint(&program);
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

            let address = args.next().unwrap_or("127.0.0.1:8000".to_string());
            server::start(&address, &tfi).ok();
        },

        _ => {
            eprintln!("ERROR: unknown subcommand {subcommand}");
            hint(&program);
            exit(1);
        },
    }
}
