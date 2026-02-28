mod bible;
mod error;
mod filesystem;
mod prelude;

use crate::bible::Bible;
use env_logger::Env;
use std::path::Path;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let bible = Bible::from_file(Path::new("assets/eng-kjv.osis.xml.xz")).unwrap();
    println!("{:?}", bible.index["Genesis"])
}
