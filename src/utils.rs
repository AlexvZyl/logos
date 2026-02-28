use crate::prelude::*;
use std::io::Read;

pub fn decompress_xz(path: &str) -> Result<String> {
    let compressed = std::fs::read(path)?;

    let mut decompressor = xz2::read::XzDecoder::new(compressed.as_slice());
    let mut decompressed = Vec::new();
    decompressor.read_to_end(&mut decompressed)?;

    return Ok(String::from_utf8(decompressed)?);
}
