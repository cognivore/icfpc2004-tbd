use std::path::Path;
use std::io::Write;

fn traverse_dir(path: &Path, qs: &mut String, cb: &mut impl FnMut(&str, &str)) {
    println!("{} {}", path.display(), qs);
    for p in std::fs::read_dir(path).unwrap() {
        let p = p.unwrap();
        let is_dir = p.metadata().unwrap().is_dir();
        let p = p.path();
        let module_name = p.file_stem().unwrap().to_str().unwrap();
        let old_len = qs.len();
        qs.push_str(module_name);
        qs.push_str("::");
        if is_dir {
            traverse_dir(&p, qs, cb);
        } else {
            println!("cargo:rerun-if-changed={}", p.to_str().unwrap());
            let source = std::fs::read_to_string(&p).unwrap();
            let mut it = source.split_terminator('\n');
            while let Some(line) = it.next() {
                if line.trim_end() != "// ENTRY_POINT" {
                    continue;
                }
                let line = it.next().unwrap();
                let line = line.strip_prefix("pub fn ").unwrap();
                let end = line.find("()").unwrap();
                let fn_name = &line[..end];
                println!("fn {}", fn_name);
                cb(fn_name, &qs);
            }
        }
        qs.truncate(old_len);
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let mut name_to_qs = std::collections::HashMap::new();
    traverse_dir(&Path::new("src"), &mut String::new(), &mut |fn_name, qs| {
        println!();
        if let Some(old_qs) = name_to_qs.insert(fn_name.to_string(), qs.to_string()) {
            println!("Duplicate entry point name {:?} in", fn_name);
            println!("  {}", old_qs);
            println!("  {}", qs);
        }
    });
    let mut name_to_qs: Vec<_> = name_to_qs.into_iter().collect();
    name_to_qs.sort();

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let fout = std::fs::File::create(Path::new(&out_dir).join("entry_points.rs")).unwrap();
    let mut fout = std::io::BufWriter::new(fout);
    writeln!(fout, "const ENTRY_POINTS: &[(&str, fn())] = &[").unwrap();
    for (fn_name, qs) in &name_to_qs {
        writeln!(fout, "    ({:?}, {}{}),", fn_name, qs, fn_name).unwrap();
    }
    writeln!(fout, "];").unwrap();
}
