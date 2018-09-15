extern crate env_logger;
extern crate ffmpeg;
#[macro_use]
extern crate handlebars;
#[macro_use]
extern crate log;
extern crate serde;
extern crate sha2;
#[macro_use]
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate regex;
#[cfg(test)]
extern crate tempfile;

mod metadata;
mod prejudice;
mod scan;
mod util;

// TODO: argument parsing
// TODO: --checksum: SHA-256 checksum
// TODO: --tags: show metadata tags (--all-tags)
// TODO: --scan: scan::get_scan_type with decode_frames = true
fn main() {
    env_logger::init();
    ffmpeg::init().expect("failed to init FFmpeg");
    unsafe {
        ffmpeg::ffi::av_log_set_level(ffmpeg::ffi::AV_LOG_FATAL);
    }
    // TODO: make sure path exists and is file
    if let Some(path) = &std::env::args().nth(1) {
        match metadata::metadata(path, false, false) {
            Ok(pretty) => println!("{}", pretty),
            Err(error) => println!("Error: {}", error),
        }
    } else {
        println!("Error: no file specified");
    }
}
