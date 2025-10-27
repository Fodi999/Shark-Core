#![forbid(unsafe_code)]

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Load raw weights from a file path. Returns Ok(vec) or Err if IO fails.
pub fn load_weights(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let p = Path::new(path);
    let mut file = if p.exists() {
        File::open(p)?
    } else {
        // try default weights location in crate
        let default = Path::new("weights/model_int4.bin");
        File::open(default)?
    };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Load file containing f32 values in little-endian and return Vec<f32>
pub fn load_f32_file(path: &str) -> Result<Vec<f32>, std::io::Error> {
    let mut f = if Path::new(path).exists() {
        File::open(path)?
    } else {
        File::open(Path::new("weights/model_int4.bin"))?
    };
    let mut buf = Vec::new();
    f.read_to_end(&mut buf)?;
    let n = buf.len() / 4;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let b = &buf[i*4..i*4+4];
        out.push(f32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    }
    Ok(out)
}
