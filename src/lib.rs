extern crate ffmpeg;
#[macro_use]
extern crate handlebars;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate sha2;
#[macro_use]
extern crate serde_json;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate tempfile;

pub mod metadata;
pub mod prejudice;
pub mod render;
pub mod scan;
pub mod stream;
pub mod tags;
pub mod util;

pub use stream::{
    AttachmentMetadata, AudioMetadata, DataMetadata, StreamMetadata, SubtitleMetadata,
    UnknownMetadata, VideoMetadata,
};
