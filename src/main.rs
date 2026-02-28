mod error;
mod prelude;
mod utils;

fn main() {
    let result = utils::decompress_xz("../assets/eng-kjv.osis.xml.xz");
}
