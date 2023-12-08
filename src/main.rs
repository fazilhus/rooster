use std::{fs, io};
use std::path::Path;
use xml::reader::{XmlEvent, EventReader};

fn xml_to_string<P: AsRef<Path>>(file_path: P) -> io::Result<String> {
    let file = fs::File::open(file_path)?;
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

fn main() {
    let dir_path = "../docs.gl/gl4";
    let dir = fs::read_dir(dir_path).unwrap();
    for entry in dir {
        let file_path = entry.unwrap().path();
        let content = xml_to_string(&file_path).unwrap();
        println!("{file_path:?} -> {}", content.len());
    }
}
