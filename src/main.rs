pub mod example_module;
pub mod dev_server;
pub mod prelude;
pub mod geometry;
pub mod biology;
pub mod geography;
pub mod cartography;
pub mod vis_server;

// produced by build.rs
include!(concat!(env!("OUT_DIR"), "/entry_points.rs"));

fn main() {
    let mut it = std::env::args();
    it.next().unwrap();
    if let Some(fn_name) = it.next() {
        let ep = ENTRY_POINTS.iter().find(|(n, _)| n == &fn_name);
        if let Some(ep) = ep {
            ep.1();
            return;
        } else {
            println!("Entry points {:?} not found.", fn_name);
        }
    } else {
        println!("Entry point not specified.");
    }
    println!("Possible entry points:");
    for (fn_name, _) in ENTRY_POINTS {
        println!(" - {}", fn_name);
    }
    std::process::exit(1);
}
