use ffmpeg;
use ffmpeg::format::context::Input;
use ffmpeg::media::Type;
use handlebars::Handlebars;
use std::fs;
use std::io;
use std::path::Path;

use prejudice;
use scan::{self, ScanType};
use tags::Tags;
use util;

use stream::parse_stream_meatadata;

// The width (20) here really should not be hard-coded, but the
// handlebars_helper! macro does not seem to support inline helpers with
// additional arguments (a statement I basically pulled out of my ass).
handlebars_helper!(padkey: |key: str| format!("{:<20}", &[key, ": "].join("")));

pub struct MediaFileMetadataOptions {
    pub include_checksum: bool,
    pub include_tags: bool,
    pub include_all_tags: bool,
    pub decode_frames: bool,
}

// TODO: rewrite this module in an object-oriented way, i.e., define and
// use the following structs:
//
// pub struct MediaFileMetadata {
//     path: Path,
//     file_name: String,
//     ...
//     streams: Vec<StreamMetadata>,
//     serialization_options: MediaFileMetadataOptions,
// }
//
// pub struct StreamMetadata {
//     stream_index: usize,
//     type: ffmpeg::media::Type,
//     ...
// }
//
// pub struct MediaFileMetadataOptions {
//     include_checksum: bool,
//     include_tags: bool,
// }
//
// And the use the builder pattern to handle MediaFileMetadataOptions.

pub fn metadata<P: AsRef<Path>>(
    path: &P,
    options: &MediaFileMetadataOptions,
) -> io::Result<String> {
    let mut format_ctx: Input = ffmpeg::format::input(path)?;
    let path: &Path = path.as_ref();
    let title: Option<String> = {
        let tags = format_ctx.metadata();
        tags.get("title")
            .or(tags.get("TITLE"))
            .map(|s| s.to_string())
    };
    let file_name: String = path.file_name().unwrap().to_str().unwrap().to_string();
    let file_size: u64 = fs::metadata(path)?.len();
    let file_size_base10: String = util::human_size(file_size, util::Base::Base10);
    let file_size_base2: String = util::human_size(file_size, util::Base::Base2);
    let container_format: String = prejudice::format_name(&format_ctx.format(), path);
    let duration_seconds: Option<f64> = if format_ctx.duration() >= 0 {
        Some(format_ctx.duration() as f64 / ffmpeg::ffi::AV_TIME_BASE as f64)
    } else {
        None
    };
    let duration: String =
        duration_seconds.map_or("Not available".to_string(), util::format_seconds);
    let scan_type: Option<ScanType> = scan::get_scan_type(&mut format_ctx, options.decode_frames)?;
    let bit_rate_num: i64 = unsafe { (*format_ctx.as_ptr()).bit_rate };
    let bit_rate: String = if bit_rate_num != 0 {
        format!("{:.0} kb/s", bit_rate_num as f64 / 1000f64)
    } else if duration_seconds.is_some() && duration_seconds.unwrap() > 0f64 {
        format!(
            "{:.0} kb/s",
            (file_size * 8) as f64 / duration_seconds.unwrap() / 1000f64
        )
    } else {
        "".to_string()
    };

    let mut streams_metadata = Vec::new();
    for stream in format_ctx.streams() {
        streams_metadata.push(parse_stream_meatadata(stream)?);
    }
    let streams_metadata_rendered = streams_metadata
        .iter()
        .map(|m| {
            m.render_default().expect(&format!(
                "failed to render metadata for stream #{}",
                m.index()
            ))
        })
        .collect::<Vec<_>>();

    let best_vstream_index = format_ctx.streams().best(Type::Video).map(|s| s.index());
    let best_vstream_metadata =
        best_vstream_index.map(|i| streams_metadata[i].video_metadata().unwrap());
    let (pixel_dimensions, sample_aspect_ratio, display_aspect_ratio, frame_rate) =
        if let Some(m) = best_vstream_metadata {
            (
                Some(m.pixel_dimensions),
                Some(m.sample_aspect_ratio),
                Some(m.display_aspect_ratio),
                Some(m.frame_rate),
            )
        } else {
            (None, None, None, None)
        };
    // let mut stream_metadata_strings = Vec::new();
    // for stream in format_ctx.streams() {
    //     stream_metadata_strings.push(stream_metadata(stream)?);
    // }
    let hash = if options.include_checksum {
        Some(util::sha256_hash(path)?)
    } else {
        None
    };

    let mut handlebars = Handlebars::new();
    handlebars.register_helper("padkey", Box::new(padkey));
    Ok(handlebars
        .render_template(
            "\
             {{#if title}}\
             Title:                  {{{title}}}\n\
             {{/if}}\
             Filename:               {{{file_name}}}\n\
             File size:              {{{file_size}}} ({{{file_size_base10}}}, {{{file_size_base2}}})\n\
             {{#if hash}}\
             SHA-256 digest:         {{{hash}}}\n\
             {{/if}}\
             Container format:       {{{container_format}}}\n\
             Duration:               {{{duration}}}\n\
             {{#if pixel_dimensions}}\
             Pixel dimensions:       {{{pixel_dimensions}}}\n\
             {{/if}}\
             {{#if sample_aspect_ratio}}\
             Sample aspect ratio:    {{{sample_aspect_ratio}}}\n\
             {{/if}}\
             {{#if display_aspect_ratio}}\
             Display aspect ratio:   {{{display_aspect_ratio}}}\n\
             {{/if}}\
             {{#if scan_type}}\
             Scan type:              {{{scan_type}}}\n\
             {{/if}}\
             {{#if frame_rate}}\
             Frame rate:             {{{frame_rate}}}\n\
             {{/if}}\
             Bit rate:               {{{bit_rate}}}\n\
             {{#each streams as |stream|}}    {{{stream}}}\n{{/each}}\
             \
             {{#if tags}}\
             Tags:\n\
             {{#each tags as |kv|}}    {{padkey kv.0}}{{{kv.1}}}\n{{/each}}\
             {{/if}}\
             \
             {{#if stream_tags}}\
             {{#each stream_tags as |s|}}\
             {{#if s.tags}}  #{{{s.index}}}\n\
             {{#each s.tags as |kv|}}    {{padkey kv.0}}{{{kv.1}}}\n{{/each}}\
             {{/if}}\
             {{/each}}\
             {{/if}}\
             ",
            &json!({
                "title": title,
                "file_name": file_name,
                "file_size": file_size,
                "file_size_base10": file_size_base10,
                "file_size_base2": file_size_base2,
                "hash": hash,
                "container_format": container_format,
                "duration": duration,
                "pixel_dimensions": pixel_dimensions,
                "sample_aspect_ratio": sample_aspect_ratio,
                "display_aspect_ratio": display_aspect_ratio,
                "scan_type": scan_type,
                "frame_rate": frame_rate,
                "bit_rate": bit_rate,
                "streams": streams_metadata_rendered,

                "tags": if options.include_all_tags {
                    Some(format_ctx.metadata().to_tags())
                } else if options.include_tags {
                    Some(format_ctx.metadata().to_filtered_tags())
                } else {
                    None
                },

                "stream_tags": if options.include_tags {
                    Some(format_ctx
                            .streams()
                            .map(|s| json!({
                                "index": s.index(),
                                "tags": if options.include_all_tags { s.metadata().to_tags() } else { s.metadata().to_filtered_tags() },
                            }))
                            .collect::<Vec<_>>())
                } else {
                    None
                },
            }),
        )
        .expect("format template rendering failure"))
}
