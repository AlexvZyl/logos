use crate::prelude::*;
use std::io::Read;

pub fn decompress_xz(compressed: &[u8]) -> Result<String> {
    let mut decompressor = xz2::read::XzDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decompressor.read_to_end(&mut decompressed)?;

    return Ok(String::from_utf8(decompressed)?);
}

// TODO: Is there a better way to check this other than just checking the extension?

pub fn is_xml_file(path: &std::path::Path) -> bool {
    path.extension().and_then(|p| p.to_str()) == Some("xml")
}

pub fn is_xz_compressed_xml(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|p| p.to_str())
        .map_or(false, |ext| ext == "xz")
        && path
            .file_stem()
            .and_then(|p| p.to_str())
            .map_or(false, |stem| stem.ends_with("xml"))
}
