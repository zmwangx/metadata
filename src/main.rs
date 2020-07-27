extern crate metadata;

extern crate clap;
extern crate env_logger;
extern crate ffmpeg_next as ffmpeg;

use clap::App;
use metadata::{MediaFileMetadata, Render};
use std::io;
use std::path::Path;
use std::process;

fn main() {
    process::exit(if run_main() { 0 } else { 1 });
}

fn run_main() -> bool {
    env_logger::init();

    let matches = App::new("metadata")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Zhiming Wang <metadata@zhimingwang.org>")
        .about("Media file metadata for human consumption.")
        .args_from_usage(
            "-c, --checksum     'Include file checksum(s)'
            -t, --tags          'Print metadata tags, except mundane ones'
            -A, --all-tags      'Print all metadata tags'
            -s, --scan          'Decode frames to determine scan type \
                                 (slower, but determines interlaced more accurately; \
                                  see man page for details)'
            <FILE>...           'Media file(s)'",
        )
        .get_matches();
    let files = matches.values_of("FILE").unwrap();
    let include_checksum = matches.is_present("checksum");
    let include_tags = matches.is_present("tags");
    let include_all_tags = matches.is_present("all-tags");
    let decode_frames = matches.is_present("scan");

    let mut successful = true;

    if ffmpeg::init().is_err() {
        eprintln!("Error: failed to initialize libav*");
        return false;
    }
    unsafe {
        ffmpeg::ffi::av_log_set_level(ffmpeg::ffi::AV_LOG_FATAL);
    }

    let build_media_file_metadata = |file: &str| -> io::Result<MediaFileMetadata> {
        let mut meta = MediaFileMetadata::new(&file)?;
        meta.include_checksum(include_checksum)?
            .include_tags(include_tags)
            .include_all_tags(include_all_tags)
            .decode_frames(decode_frames)?;
        Ok(meta)
    };

    for file in files {
        if !Path::new(file).is_file() {
            eprintln!("Error: \"{}\" does not exist or is not a file", file);
            successful = false;
            continue;
        }
        match build_media_file_metadata(&file) {
            Ok(m) => match m.render_default() {
                Ok(rendered) => println!("{}", rendered),
                Err(_) => {
                    eprintln!("Error: failed to render metadata for \"{}\"", file);
                    successful = false;
                }
            },
            Err(error) => {
                eprintln!("Error: {}", error);
                successful = false;
            }
        }
    }

    successful
}
