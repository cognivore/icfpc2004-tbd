use std::collections::HashMap;
use crate::dev_server::{Request, ResponseBuilder, HandlerResult, serve_forever};
use crate::cartography::World;
use crate::geography::{Contents, MapToken::*};
use crate::biology::Color::*;
use crate::geometry::Pos;

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

    red_program: String,
    black_program: String,
}

impl Background {
    fn new(m: &Match) -> Self {
        let world = std::fs::read_to_string(&m.world).unwrap();
        let world = World::from_map_string(&world);

        let mut rocks = Vec::new();
        let mut red_anthill = Vec::new();
        let mut black_anthill = Vec::new();
        for (&Pos { x, y }, token) in &world.data {
            match token {
                Rock => rocks.push((x, y)),
                Clear(Contents { anthill: Some(Red), .. }) => red_anthill.push((x, y)),
                Clear(Contents { anthill: Some(Black), .. }) => black_anthill.push((x, y)),
                Clear(Contents { anthill: None, .. }) => {}
            }
        }

        let red_program = std::fs::read_to_string(&m.red).unwrap();
        let black_program = std::fs::read_to_string(&m.black).unwrap();

        Background {
            rocks,
            red_anthill,
            black_anthill,
            red_program,
            black_program,
        }
    }
}

// Things that change
#[derive(serde::Serialize)]
struct ReplayFrame {
    frame_no: usize,
    food: Vec<(u8, u8, u16)>,  // (x, y, amount)
    ants: Vec<Ant>,
}

#[derive(serde::Serialize)]
struct Ant {
    id: u8,
    color: &'static str,  // "red" or "black"
    x: u8,
    y: u8,
    dir: i32,  // E = 0 and then clockwise
    has_food: bool,
    state: u16,
    resting: u8,
}

impl ReplayFrame {
    fn new(frame_no: usize, w: &World) -> Self {
        let mut food = Vec::new();
        let mut ants = Vec::new();
        for (&Pos { x, y }, token) in &w.data {
            match token {
                Rock => {}
                Clear(Contents { food: f, ant, .. }) => {
                    if f.0 > 0 {
                        food.push((x, y, f.0));
                    }
                    if let Some(ant) = ant {
                        ants.push(Ant {
                            id: ant.id,
                            color: match ant.color {
                                Red => "red",
                                Black => "black",
                            },
                            x,
                            y,
                            dir: ant.direction as i32,
                            has_food: ant.has_food,
                            state: ant.state.0,
                            resting: ant.resting,
                        });
                    }
                }
            }
        }
        ReplayFrame {
            frame_no,
            food,
            ants,
        }
    }
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
                    let bg = Background::new(&m);
                    resp.code("200 OK")
                        .body(serde_json::to_vec(&bg).unwrap())
                }
                "/frame" => {
                    let m = &query["match"];
                    let m: Match = serde_json::from_str(&m).unwrap();
                    let frame_no = query["frame_no"].parse().unwrap();

                    let world = std::fs::read_to_string(&m.world).unwrap();
                    let mut world = World::from_map_string(&world);
                    for _ in 0..frame_no {
                        world.fake_round();
                    }
                    let frame = ReplayFrame::new(frame_no, &world);

                    resp.code("200 OK")
                        .body(serde_json::to_vec(&frame).unwrap())
                }
                _ => handle_static(req, resp)
            }
        }
    })
}
