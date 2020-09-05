use std::collections::HashMap;
use crate::dev_server::{Request, ResponseBuilder, HandlerResult, serve_forever};
use crate::cartography::World;
use crate::geography::{Contents, MapToken::*};
use crate::biology::Color::*;

// Keep type definitions in sync with vis/types.ts.

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug)]
struct Match {
    world: String,
    red: String,
    black: String,
}

// Things that don't change
#[derive(serde::Serialize)]
struct Background {
    rocks: Vec<(u8, u8)>,
    red_anthill: Vec<(u8, u8)>,
    black_anthill: Vec<(u8, u8)>,
}

// Things that change
#[derive(serde::Serialize)]
struct ReplayFrame {
    food: Vec<(i32, i32, i32)>,  // (x, y, amount)
    ants: Vec<Ant>,
}

#[derive(serde::Serialize)]
struct Ant {
    color: &'static str,  // "red" or "black"
    x: i32,
    y: i32,
    dir: i32,  // E = 0 and then clockwise
    has_food: bool,
}

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
            let (path, query) = match req.path.find('?') {
                Some(idx) => (&req.path[..idx], &req.path[idx + 1..]),
                None => (req.path, "")
            };
            let query: HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes()).collect();
            // dbg!(query);
            match path {
                "/" => {
                    let m = Match {
                        world: "data/tiny.world".to_string(),
                        red: "data/sample.ant".to_string(),
                        black: "data/sample.ant".to_string(),
                    };
                    let s = serde_json::to_string(&m).unwrap();
                    resp.code("200 OK")
                        .header("Content-Type", "text/html; charset=utf8")
                        .body(format!("
                            <p>TODO: Draw the rest of the fucking owl.</p>
                            <p><a href='/vis/index.html#{}'>example match</a></p>
                        ", htmlescape::encode_attribute(&s)))
                }
                "/background" => {
                    let m = &query["match"];
                    let m: Match = serde_json::from_str(&m).unwrap();

                    let world = std::fs::read_to_string(m.world).unwrap();
                    let world = World::from_map_string(&world);

                    let mut rocks = Vec::new();
                    let mut red_anthill = Vec::new();
                    let mut black_anthill = Vec::new();
                    for (pos, token) in &world.data {
                        match token {
                            Rock => rocks.push((pos.x, pos.y)),
                            Clear(Contents { anthill: Some(Red), .. }) => red_anthill.push((pos.x, pos.y)),
                            Clear(Contents { anthill: Some(Black), .. }) => black_anthill.push((pos.x, pos.y)),
                            Clear(Contents { anthill: None, .. }) => {}
                        }
                    }
                    let bg = Background {
                        rocks,
                        red_anthill,
                        black_anthill,
                    };

                    resp.code("200 OK")
                        .body(serde_json::to_vec(&bg).unwrap())
                }
                "/frame" => {
                    let m = &query["match"];
                    let _m: Match = serde_json::from_str(&m).unwrap();
                    let _frame_no: i32 = query["frame_no"].parse().unwrap();

                    // TODO: actual stuff
                    let frame = ReplayFrame {
                        food: vec![(3, 3, 9), (4, 4, 5)],
                        ants: vec![
                            Ant { color: "red", x: 1, y: 1, dir: 1, has_food: false },
                            Ant { color: "black", x: 1, y: 2, dir: 2, has_food: true },
                        ],
                    };
                    resp.code("200 OK")
                        .body(serde_json::to_vec(&frame).unwrap())
                }
                _ => handle_static(req, resp)
            }
        }
    })
}
