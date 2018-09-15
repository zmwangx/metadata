extern crate clap;
extern crate env_logger;
extern crate ffmpeg;
#[macro_use]
extern crate handlebars;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
extern crate sha2;
#[macro_use]
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate tempfile;

use clap::App;
use metadata::MediaFileMetadataOptions;
use std::path::Path;
use std::process;

mod metadata;
mod prejudice;
mod scan;
mod tags;
mod util;

// TODO: add integration tests
fn main() {
    match run_main() {
        true => process::exit(0),
        false => process::exit(1),
    }
}

fn run_main() -> bool {
    env_logger::init();

    let matches = App::new("metadata")
        .version("0.1.0")
        .author("Zhiming Wang <metadata@zhimingwang.org>")
        .about("Media file metadata for human consumption.")
        .args_from_usage(
            "-c, --checksum     'Include file checksum(s)'
            -t, --tags          'Print metadata tags, except mundane ones'
            -A, --all-tags      'Print all metadata tags'
            --scan              'Decode frames to determine scan type \
                                 (slower, but may determine interlaced more accurately; \
                                  see man page for details)'
            <FILE>...           'Media file(s)'",
        )
        .get_matches();
    let files = matches.values_of("FILE").unwrap();
    let options = MediaFileMetadataOptions {
        include_checksum: matches.is_present("checksum"),
        include_tags: matches.is_present("tags") || matches.is_present("all-tags"),
        include_all_tags: matches.is_present("all-tags"),
        decode_frames: matches.is_present("scan"),
    };

    let mut successful = true;

    if ffmpeg::init().is_err() {
        eprintln!("Error: failed to initialize libav*");
        return false;
    }
    unsafe {
        ffmpeg::ffi::av_log_set_level(ffmpeg::ffi::AV_LOG_FATAL);
    }

    for file in files {
        if !Path::new(file).is_file() {
            eprintln!("Error: \"{}\" does not exist or is not a file", file);
            successful = false;
            continue;
        }
        match metadata::metadata(&file, &options) {
            Ok(pretty) => println!("{}", pretty),
            Err(error) => {
                eprintln!("Error: {}", error);
                successful = false;
            }
        }
    }

    successful
}
