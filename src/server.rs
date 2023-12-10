use std::fs::File;
use tiny_http::{Header, Method, Request, Response, Server};
use crate::model::{search_query, TermFreqIndex};

pub fn start(address: &str, tfi: &TermFreqIndex) -> Result<(), ()> {
    let server = Server::http(&address).map_err(|err| {
        eprintln!("ERROR: could not start HTTP server at {address}: {err}");
    })?;

    println!("INFO: Listening at http://{address}/");

    for request in server.incoming_requests() {
        serve_request(request, tfi).ok();
    }

    eprintln!("ERROR: the server socket has shutdown");
    Err(())
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

fn serve_api_search(mut request: Request, tfi: &TermFreqIndex) -> Result<(), ()> {
    let mut buf = String::new();
    request.as_reader().read_to_string(&mut buf).map_err(|err| {
        eprintln!("ERROR: could not interpret body as utf-8: {err}");
    })?;
    let query: Vec<char> = buf.chars().collect();
    let result = search_query(&query, &tfi);

    let json = serde_json::to_string(&result
        .iter().take(10)
        .collect::<Vec<_>>())
        .map_err(|err| {
            eprintln!("ERROR: could not convert search results to JSON: {err}");
        })?;
    let content_type = Header::from_bytes("Content-Type", "application/json")?;
    let response = Response::from_string(&json)
        .with_header(content_type);
    request.respond(response).map_err(|err| {
        eprintln!("ERROR: could not serve a request: {err}");
    })
}

fn serve_404(request: Request) -> Result<(), ()> {
    request.respond(Response::from_string("Error 404").with_status_code(404))
        .map_err(|err| {
            eprintln!("ERROR: could not respond to request: {err}");
        })
}

fn serve_request(request: Request, tfi: &TermFreqIndex) -> Result<(), ()> {
    match (request.method(), request.url()) {
        (Method::Get, "/") | (Method::Get, "/index.html") => {
            serve_static_file(request, "index.html", "text/html; charset=utf-8")
        },

        (Method::Get, "/index.js") => {
            serve_static_file(request, "index.js", "text/javascript; charset=utf-8")
        },

        (Method::Post, "/api/search") => {
            serve_api_search(request, tfi)
        },

        _ => {
            serve_404(request)
        },
    }
}