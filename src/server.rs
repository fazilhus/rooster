use std::fs::File;
use std::{io, str};
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

fn serve_static_file(request: Request, file_path: &str, content_type: &str) -> io::Result<()> {
    println!("INFO: incoming request! method: {:?}, url: {:?}",
             request.method(),
             request.url());

    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("ERROR: could not serve file {file_path}: {err}");
            if err.kind() == io::ErrorKind::NotFound {
                return serve_404(request);
            }
            return serve_500(request);
        },
    };


    let content_type = Header::from_bytes(b"Content-Type", content_type.as_bytes())
        .unwrap();
    let response = Response::from_file(file).with_header(content_type);
    request.respond(response)
}

fn serve_api_search(mut request: Request, tfi: &TermFreqIndex) -> io::Result<()> {
    let mut buf = Vec::new();
    if let Err(err) = request.as_reader().read_to_end(&mut buf) {
        eprintln!("ERROR: could not interpret body as utf-8: {err}");
        return serve_500(request);
    };

    let query: Vec<char> = match str::from_utf8(&buf) {
        Ok(query) => query.chars().collect(),
        Err(err) => {
            eprintln!("ERROR: could not interpret query as utf-8: {err}");
            return serve_400(request, "Query must be a valid utf-8");
        }
    };
    let result = search_query(&query, &tfi);

    let json = match serde_json::to_string(&result
        .iter().take(10)
        .collect::<Vec<_>>()) {
        Ok(json) => json,
        Err(err) => {
            eprintln!("ERROR: could not convert results to JSON: {err}");
            return serve_500(request);
        }
    };


    let content_type = Header::from_bytes("Content-Type", "application/json")
        .unwrap();
    let response = Response::from_string(&json)
        .with_header(content_type);
    request.respond(response)
}

fn serve_400(request: Request, msg: &str) -> io::Result<()> {
    request.respond(Response::from_string(format!("Error 400: {msg}"))
        .with_status_code(400))
}

fn serve_404(request: Request) -> io::Result<()> {
    request.respond(Response::from_string("Error 404")
        .with_status_code(404))
}

fn serve_500(request: Request) -> io::Result<()> {
    request.respond(Response::from_string("Error 500")
        .with_status_code(500))
}

fn serve_request(request: Request, tfi: &TermFreqIndex) -> io::Result<()> {
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