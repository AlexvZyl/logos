#![allow(dead_code)]

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
    for part in bible.get_verse_iter("Ephesians", 1, 1).unwrap() {
        print!("{}", part);
    }
    println!();
}
