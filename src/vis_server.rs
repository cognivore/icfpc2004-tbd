use crate::dev_server::{Request, ResponseBuilder, HandlerResult, serve_forever};

fn handle_static(req: &Request, resp: ResponseBuilder) -> HandlerResult {
    let path = req.path.strip_prefix('/').unwrap();
    let path = std::path::Path::new(path);
    if let Ok(data) = std::fs::read(path) {
        let content_type = match path.extension().map(|s| s.to_str().unwrap()) {
            Some("js") => "application/javascript",
            Some("html") => "text/html; charset=utf8",
            Some("css") => "text/css",
            _ => "text/plain",
        };
        resp.code("200 OK")
            .header("Content-Type", content_type)
            .body(data)
    } else {
        resp.code("404 Not Found").body("not found")
    }
}

// ENTRY_POINT
pub fn vis_server() {
    let listener = std::net::TcpListener::bind("127.0.0.1:8000").unwrap();
    eprintln!("serving at http://127.0.0.1:8000 ...");

    serve_forever(listener, || {
        move |req, resp| {
            match req.path {
                "/" => resp.code("200 OK")
                    .header("Content-Type", "text/html; charset=utf8")
                    .body("
                        <p>TODO: Draw the rest of the fucking owl.</p>
                        <p>Meanwhile, see <a href='/vis/index.html'>static demo</a>.</p>
                    "),

                _ => handle_static(req, resp)
            }
        }
    })
}
