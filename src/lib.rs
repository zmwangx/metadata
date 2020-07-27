extern crate ffmpeg_next as ffmpeg;
#[macro_use]
extern crate handlebars;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate sha2;

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[cfg(test)]
extern crate tempfile;

pub mod media_file;
pub mod prejudice;
pub mod render;
pub mod scan;
pub mod stream;
pub mod tags;
pub mod util;

pub use media_file::MediaFileMetadata;
pub use render::Render;
pub use scan::ScanType;
pub use stream::{
    AttachmentMetadata, AudioMetadata, DataMetadata, StreamMetadata, SubtitleMetadata,
    UnknownMetadata, VideoMetadata,
};
pub use tags::{Tags, ToTags};
