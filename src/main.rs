use std::io;
use std::fs::File;
use std::process::exit;
use xml::reader::{XmlEvent, EventReader};

fn xml_to_string(file_path: &str) -> io::Result<String> {
    let file = File::open(file_path)?;
    let event_reader = EventReader::new(file);

    let mut content = String::new();
    for event in event_reader.into_iter() {
        if let XmlEvent::Characters(text) = event.expect("TODO") {
            content.push_str(&text);
        }
    }
    Ok(content)
}
fn main() {
    let file_path = "../docs.gl/gl4/glClear.xhtml";
    let content = xml_to_string(file_path).expect("TODO");
    println!("{content}");
}
