use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::dev_server::{Request, ResponseBuilder, HandlerResult, serve_forever};
use crate::cartography::World;
use crate::geography::{Contents, MapToken::*};
use crate::biology::Color::*;
use crate::{neurology::{Instruction, parse_ant}, geometry::Pos, number_theory::Random};

//use crate::dump_trace::*;


// Keep type definitions in sync with vis/types.ts.

#[derive(serde::Deserialize, serde::Serialize)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Match {
    world: String,
    red: String,
    black: String,
    seed: u32,
}

// Things that don't change
#[derive(serde::Serialize)]
struct Background {
    rocks: Vec<(u8, u8)>,
    red_anthill: Vec<(u8, u8)>,
    black_anthill: Vec<(u8, u8)>,

    red_brain: String,
    black_brain: String,
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

        let red_brain = std::fs::read_to_string(&m.red).unwrap();
        let black_brain = std::fs::read_to_string(&m.black).unwrap();

        Background {
            rocks,
            red_anthill,
            black_anthill,
            red_brain,
            black_brain,
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

struct CacheEntry {
    ant_brains: [Vec<Instruction>; 2],

    frame_no: usize,
    rng: Random,
    world: World,
}

impl CacheEntry {
    fn new(m: &Match) -> Self {
        let ant_brains = [
            parse_ant(&std::fs::read_to_string(&m.red).unwrap()),
            parse_ant(&std::fs::read_to_string(&m.black).unwrap()),
        ];
        let rng = Random::new(m.seed);

        let world = std::fs::read_to_string(&m.world).unwrap();
        let world = World::from_map_string(&world);
        CacheEntry {
            ant_brains,
            frame_no: 0,
            rng,
            world,
        }
    }

    fn get_frame(&mut self, m: &Match, frame_no: usize) -> ReplayFrame {
        if self.frame_no > frame_no {
            let world = std::fs::read_to_string(&m.world).unwrap();
            self.world = World::from_map_string(&world);
            self.rng = Random::new(m.seed);
            self.frame_no = 0;
        }
        for _ in self.frame_no..frame_no {
            //dump_world(self.world.clone(), self.frame_no);
            self.world.round(&self.ant_brains, &mut self.rng);
        }
        self.frame_no = frame_no;
        ReplayFrame::new(frame_no, &self.world)
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

    let cache: HashMap<Match, CacheEntry> = HashMap::new();
    let cache = Arc::new(Mutex::new(cache));

    serve_forever(listener, || {
        let cache = Arc::clone(&cache);
        move |req, resp| {
            let (path, query) = match req.path.find('?') {
                Some(idx) => (&req.path[..idx], &req.path[idx + 1..]),
                None => (req.path, "")
            };
            let query: HashMap<_, _> = url::form_urlencoded::parse(query.as_bytes()).collect();
            match path {
                "/" => {
                    resp.code("302 Found")
                        .header("Location", "/vis/select_match.html")
                        .no_body()
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

                    let frame = cache.lock().unwrap()
                        .entry(m.clone())
                        .or_insert_with(|| CacheEntry::new(&m))
                        .get_frame(&m, frame_no);

                    resp.code("200 OK")
                        .body(serde_json::to_vec(&frame).unwrap())
                }
                _ => handle_static(req, resp)
            }
        }
    })
}
