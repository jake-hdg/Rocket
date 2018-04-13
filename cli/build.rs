extern crate include_dir;

use std::env;
use std::path::Path;
use include_dir::include_dir;

fn main() {
    let outdir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&outdir).join("assets.rs");

    include_dir("templates")
        .as_variable("TEMPLATES")
        .to_file(dest_path)
        .unwrap();
}
