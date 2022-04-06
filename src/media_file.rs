use crate::ffmpeg;
use crate::ffmpeg::media::Type;
use crate::ffmpeg::util::rational::Rational;
use std::fs;
use std::io;
use std::path::Path;

use crate::prejudice;
use crate::scan::{self, ScanType};
use crate::stream::{parse_stream_meatadata, StreamMetadata};
use crate::tags::{Tags, ToTags};
use crate::util;

#[derive(Clone, Debug, Serialize)]
pub struct MediaFileMetadataOptions {
    pub include_checksum: bool,
    pub include_tags: bool,
    pub include_all_tags: bool,
    pub decode_frames: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct StreamTags {
    pub index: usize,
    pub tags: Tags,
}

#[derive(Clone, Debug, Serialize)]
pub struct MediaFileMetadata {
    #[serde(rename = "options")]
    pub options: MediaFileMetadataOptions,

    pub path: String,
    pub file_name: String,
    pub file_size: u64,
    pub file_size_base10: String,
    pub file_size_base2: String,
    pub hash: Option<String>,

    pub title: Option<String>,

    pub container_format: String,

    #[serde(skip_serializing)]
    pub _duration: Option<f64>,
    pub duration: Option<String>,

    pub width: Option<u32>,
    pub height: Option<u32>,
    pub pixel_dimensions: Option<String>,
    #[serde(skip_serializing)]
    pub _sample_aspect_ratio: Option<Rational>,
    pub sample_aspect_ratio: Option<String>,
    #[serde(skip_serializing)]
    pub _display_aspect_ratio: Option<Rational>,
    pub display_aspect_ratio: Option<String>,

    #[serde(skip_serializing)]
    pub _scan_type: Option<ScanType>,
    pub scan_type: Option<String>,

    #[serde(skip_serializing)]
    pub _frame_rate: Option<Rational>,
    pub frame_rate: Option<String>,

    #[serde(skip_serializing)]
    pub _bit_rate: Option<u64>,
    pub bit_rate: Option<String>,

    #[serde(skip_serializing)]
    pub _streams_metadata: Vec<StreamMetadata>,
    pub streams_metadata_rendered: Vec<String>,

    pub tags: Tags,
    pub filtered_tags: Tags,
    pub streams_tags: Vec<StreamTags>,
    pub streams_filtered_tags: Vec<StreamTags>,
}

impl MediaFileMetadata {
    pub fn new<P: AsRef<Path>>(path: &P) -> io::Result<MediaFileMetadata> {
        let mut format_ctx = ffmpeg::format::input(path)?;

        let path = path.as_ref();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
        let file_size = fs::metadata(path)?.len();
        let file_size_base10 = util::human_size(file_size, util::Base::Base10);
        let file_size_base2 = util::human_size(file_size, util::Base::Base2);

        // let hash = if options.include_checksum {
        //     Some(util::sha256_hash(path)?)
        // } else {
        //     None
        // };

        let container_format = prejudice::format_name(&format_ctx.format(), path);

        let _duration = if format_ctx.duration() >= 0 {
            Some(format_ctx.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64)
        } else {
            None
        };
        let duration = _duration.map(util::format_seconds);

        let _scan_type = { scan::get_scan_type(&mut format_ctx)? };
        let scan_type = _scan_type.clone().map(|s| s.to_string());

        let _bit_rate = match format_ctx.bit_rate() {
            0 => None,
            _ => Some(format_ctx.bit_rate() as u64),
        };
        let bit_rate = if let Some(rate) = _bit_rate {
            Some(format!("{:.0} kb/s", rate as f64 / 1000f64))
        } else if let Some(seconds) = _duration {
            if seconds > 0f64 {
                Some(format!(
                    "{:.0} kb/s",
                    (file_size * 8) as f64 / seconds / 1000f64
                ))
            } else {
                None
            }
        } else {
            None
        };

        let mut _streams_metadata = Vec::new();
        for stream in format_ctx.streams() {
            _streams_metadata.push(parse_stream_meatadata(stream)?);
        }
        let streams_metadata_rendered = _streams_metadata
            .iter()
            .map(|m| {
                m.render_default().unwrap_or_else(|_| {
                    panic!("failed to render metadata for stream #{}", m.index())
                })
            })
            .collect::<Vec<_>>();

        let best_vstream_index = format_ctx.streams().best(Type::Video).map(|s| s.index());
        let best_vstream_metadata =
            best_vstream_index.map(|i| _streams_metadata[i].video_metadata().unwrap());
        let (
            width,
            height,
            pixel_dimensions,
            _sample_aspect_ratio,
            sample_aspect_ratio,
            _display_aspect_ratio,
            display_aspect_ratio,
            _frame_rate,
            frame_rate,
        ) = if let Some(m) = best_vstream_metadata {
            (
                Some(m.width),
                Some(m.height),
                Some(m.pixel_dimensions),
                Some(m._sample_aspect_ratio),
                Some(m.sample_aspect_ratio),
                Some(m._display_aspect_ratio),
                Some(m.display_aspect_ratio),
                m._frame_rate,
                m.frame_rate,
            )
        } else {
            (None, None, None, None, None, None, None, None, None)
        };

        let tagdict = format_ctx.metadata();
        let title = tagdict
            .get("title")
            .or_else(|| tagdict.get("TITLE"))
            .map(|s| s.to_string());

        let tags = tagdict.to_tags();
        let filtered_tags = tagdict.to_filtered_tags();

        let streams_tags = format_ctx
            .streams()
            .map(|s| StreamTags {
                index: s.index(),
                tags: s.metadata().to_tags(),
            })
            .collect();
        let streams_filtered_tags = format_ctx
            .streams()
            .map(|s| StreamTags {
                index: s.index(),
                tags: s.metadata().to_filtered_tags(),
            })
            .collect();

        Ok(MediaFileMetadata {
            options: MediaFileMetadataOptions {
                include_checksum: false,
                include_tags: false,
                include_all_tags: false,
                decode_frames: false,
            },
            path: path.to_str().unwrap().to_string(),
            file_name,
            file_size,
            file_size_base10,
            file_size_base2,
            hash: None,
            title,
            container_format,
            _duration,
            duration,
            width,
            height,
            pixel_dimensions,
            _sample_aspect_ratio,
            sample_aspect_ratio,
            _display_aspect_ratio,
            display_aspect_ratio,
            _scan_type,
            scan_type,
            _frame_rate,
            frame_rate,
            _bit_rate,
            bit_rate,
            _streams_metadata,
            streams_metadata_rendered,
            tags,
            filtered_tags,
            streams_tags,
            streams_filtered_tags,
        })
    }

    pub fn include_checksum(&mut self, on: bool) -> io::Result<&mut MediaFileMetadata> {
        if on {
            self.options.include_checksum = true;
            self.hash = Some(util::sha256_hash(Path::new(&self.path))?);
        } else {
            self.options.include_checksum = false;
        }
        Ok(self)
    }

    pub fn include_tags(&mut self, on: bool) -> &mut MediaFileMetadata {
        if on {
            self.options.include_tags = true;
        } else {
            self.options.include_tags = false;
        }
        self
    }

    pub fn include_all_tags(&mut self, on: bool) -> &mut MediaFileMetadata {
        if on {
            self.options.include_tags = true;
            self.options.include_all_tags = true;
        } else {
            self.options.include_all_tags = false;
        }
        self
    }
}
