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

pub mod metadata;
pub mod prejudice;
pub mod scan;
pub mod tags;
pub mod util;
