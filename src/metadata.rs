use ffmpeg::codec::decoder::audio::Audio;
use ffmpeg::codec::decoder::video::Video;
use ffmpeg::format::context::Input;
use ffmpeg::util::channel_layout::ChannelLayout;
use ffmpeg::util::rational::Rational;
use ffmpeg::{self, DictionaryRef, Stream};
use handlebars::Handlebars;
use regex::Regex;
use std::fs;
use std::io;
use std::path::Path;
use std::str::from_utf8_unchecked;

use prejudice;
use scan::{self, ScanType};
use util;

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

trait Tags {
    fn to_tags(&self) -> Vec<(String, String)>;

    fn to_filtered_tags(&self) -> Vec<(String, String)> {
        self.to_tags()
            .iter()
            .filter(|(k, _)| !Self::tag_is_boring(&k))
            .cloned()
            .collect()
    }

    fn tag_is_boring(key: &str) -> bool {
        lazy_static!{
            // Some fixed names, plus tags beginning with an underscore (e.g.,
            // _STATISTICS_* tags by mkvmerge), or in reversed domain name notation
            // (e.g., com.apple.quicktime.player.* tags).
            static ref BORING_PATTERN: Regex = Regex::new(r"^((major_brand|minor_version|compatible_brands|creation_time|handler_name|encoder)$|_|com\.)").unwrap();
        }
        BORING_PATTERN.is_match(key)
    }
}

impl<'a> Tags for DictionaryRef<'a> {
    fn to_tags(&self) -> Vec<(String, String)> {
        self.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }
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

// TODO: use None (Option<String>) rather than empty strings.

// Returns the two representations of the stream's frate rate.
// Could be invalid, with an empty string representation!
fn get_frame_rate(video: &Video) -> (Rational, String) {
    let r_frame_rate = video.frame_rate().unwrap();
    let r_frame_rate_num = r_frame_rate.numerator();
    let r_frame_rate_den = r_frame_rate.denominator();
    let frame_rate: String;
    if r_frame_rate_den == 0 {
        frame_rate = "".to_string();
    } else if r_frame_rate_num % r_frame_rate_den == 0 {
        frame_rate = format!("{} fps", r_frame_rate_num / r_frame_rate_den);
    } else {
        frame_rate = format!(
            "{:.2} fps",
            r_frame_rate_num as f64 / r_frame_rate_den as f64
        );
    }
    (r_frame_rate, frame_rate)
}

fn get_channel_layout(audio: &Audio) -> (ChannelLayout, String) {
    let layout = audio.channel_layout();
    let layout_string: String;
    let nb_channels = audio.channels();
    let mut buf = [0u8; 128];
    unsafe {
        ffmpeg::ffi::av_get_channel_layout_string(
            buf.as_mut_ptr() as *mut i8,
            128,
            nb_channels as i32,
            layout.bits(),
        );
        layout_string = from_utf8_unchecked(&buf).to_string();
    }
    (layout, layout_string)
}

// Returns (((width, height), sar, dar),
//          (pixel_dimensions_str, sar_str, dar_str)).
//
// sar is the sample aspect ratio (aka pixel aspect ratio); dar is the
// display aspect ratio. The following is satisfied:
//
//     width/height * sar = dar
//
// width:height is also sometimes called the storage aspect ratio; do
// not confuse with sar.
//
// *_str are string representations.
fn get_dimensions_and_aspect_radio(
    video: &Video,
) -> (((u32, u32), Rational, Rational), (String, String, String)) {
    let width = video.width();
    let height = video.height();
    // If sample_aspect_ratio is not available (i.e., 1:1),
    // video.aspect_ratio() produces Rational(0, 1), so
    // apparently we need to correct this.
    let sar = if video.aspect_ratio().numerator() == 0 {
        Rational(1, 1)
    } else {
        video.aspect_ratio().reduce()
    };
    let dar = sar * Rational(width as i32, height as i32);
    (
        ((width, height), sar, dar),
        (
            format!("{}x{}", width, height),
            format!("{}:{}", sar.numerator(), sar.denominator()),
            format!("{}:{}", dar.numerator(), dar.denominator()),
        ),
    )
}

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
    let pixel_dimensions: String;
    let sample_aspect_ratio: String;
    let display_aspect_ratio: String;
    let scan_type: Option<ScanType> = scan::get_scan_type(&mut format_ctx, options.decode_frames)?;
    let frame_rate: String;
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
    match format_ctx.streams().best(ffmpeg::media::Type::Video) {
        Some(stream) => {
            let video = stream.codec().decoder().video()?;
            let dim_sar_dar = get_dimensions_and_aspect_radio(&video).1;
            pixel_dimensions = dim_sar_dar.0;
            sample_aspect_ratio = dim_sar_dar.1;
            display_aspect_ratio = dim_sar_dar.2;
            frame_rate = get_frame_rate(&video).1;
        }
        None => {
            pixel_dimensions = "".to_string();
            sample_aspect_ratio = "".to_string();
            display_aspect_ratio = "".to_string();
            frame_rate = "".to_string();
        }
    };
    let mut stream_metadata_strings = Vec::new();
    for stream in format_ctx.streams() {
        stream_metadata_strings.push(stream_metadata(stream)?);
    }
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
                "streams": stream_metadata_strings,

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

// stream_metadata is similar to avcodec_string (libavcodec/utils.c), which is
// used by ffmpeg/ffprobe's to display stream info.
// https://ffmpeg.org/doxygen/4.0/group__lavc__misc.html#ga6d4056568b5ab73d2e55800d9a5caa66
pub fn stream_metadata(stream: Stream) -> io::Result<String> {
    let stream_index: usize = stream.index();
    let metadata = stream.metadata();
    let codec_par = stream.parameters();
    let codec_ctx = stream.codec();
    let medium = codec_ctx.medium();
    let decoder = codec_ctx.decoder();
    let template: &str;
    let values;
    match medium {
        ffmpeg::media::Type::Video => {
            let video = decoder.video()?;
            let codec_desc = prejudice::codec_description(&codec_par);
            let pixel_fmt = video.format().descriptor().map(|d| d.name());
            let color_range = video.color_range().name();
            let color_space = video.color_space().name();
            let color_primaries = video.color_primaries().name();
            let color_trc = video.color_transfer_characteristic().name();
            let color_specs = [color_range.map(|v| v.to_string()), {
                // https://github.com/FFmpeg/FFmpeg/blob/n4.0.2/libavcodec/utils.c#L1220-L1233
                if color_space.or(color_primaries).or(color_trc).is_some() {
                    if color_space == color_primaries && color_space == color_trc {
                        Some(color_space.unwrap_or("unknown").to_string())
                    } else {
                        Some(format!(
                            "{}/{}/{}",
                            color_space.unwrap_or("unknown"),
                            color_primaries.unwrap_or("unknown"),
                            color_trc.unwrap_or("unknown")
                        ))
                    }
                } else {
                    None
                }
            }].iter()
                .filter(|v| v.is_some())
                .map(|v| v.clone().unwrap())
                .collect::<Vec<_>>()
                .join(", ");
            let dim_sar_dar = get_dimensions_and_aspect_radio(&video).1;
            let frame_rate = get_frame_rate(&video).1;
            let bit_rate_num = video.bit_rate();
            let bit_rate = if bit_rate_num != 0 {
                format!("{:.0} kb/s", bit_rate_num as f64 / 1000f64)
            } else {
                "".to_string()
            };
            template = "#{{{stream_index}}}: Video\
                        , {{{codec_desc}}}\
                        {{#if pixel_fmt}}\
                        , {{{pixel_fmt}}}{{#if color_specs}} ({{{color_specs}}}){{/if}}\
                        {{/if}}\
                        , {{{pixel_dimensions}}} \
                        (SAR {{{sample_aspect_ratio}}}, DAR {{{display_aspect_ratio}}})\
                        {{#if frame_rate}}\
                        , {{{frame_rate}}}\
                        {{/if}}\
                        {{#if bit_rate}}\
                        , {{{bit_rate}}}\
                        {{/if}}\
                        ";
            values = json!({
                "stream_index": stream_index,
                "codec_desc": codec_desc,
                "pixel_fmt": pixel_fmt,
                "color_specs": color_specs,
                "pixel_dimensions": dim_sar_dar.0,
                "sample_aspect_ratio": dim_sar_dar.1,
                "display_aspect_ratio": dim_sar_dar.2,
                "frame_rate": frame_rate,
                "bit_rate": bit_rate,
            });
        }

        ffmpeg::media::Type::Audio => {
            let audio = decoder.audio()?;
            let language = metadata.get("language").or(metadata.get("LANGUAGE"));
            let codec_desc = prejudice::codec_description(&codec_par);
            let sample_rate_num = audio.rate();
            let sample_rate = format!("{} Hz", sample_rate_num);
            let (_, channel_layout) = get_channel_layout(&audio);
            let bit_rate_num = audio.bit_rate();
            let bit_rate = if bit_rate_num != 0 {
                format!("{:.0} kb/s", bit_rate_num as f64 / 1000f64)
            } else {
                "".to_string()
            };
            template = "#{{{stream_index}}}: Audio\
                        {{#if language}} ({{{language}}}){{/if}}\
                        , {{{codec_desc}}}\
                        , {{{sample_rate}}}\
                        , {{{channel_layout}}}\
                        {{#if bit_rate}}\
                        , {{{bit_rate}}}\
                        {{/if}}\
                        ";
            values = json!({
                "stream_index": stream_index,
                "language": language,
                "codec_desc": codec_desc,
                "sample_rate": sample_rate,
                "channel_layout": channel_layout,
                "bit_rate": bit_rate,
            });
        }

        ffmpeg::media::Type::Subtitle => {
            let language = metadata.get("language").or(metadata.get("LANGUAGE"));
            template = "#{{{stream_index}}}: Subtitle\
                        {{#if language}} ({{{language}}}){{/if}}\
                        , {{{codec_desc}}}\
                        ";
            let codec_desc = prejudice::codec_description(&codec_par);
            values = json!({
                "stream_index": stream_index,
                "language": language,
                "codec_desc": codec_desc,
            });
        }

        ffmpeg::media::Type::Data => {
            template = "#{{{stream_index}}}: Data";
            values = json!({
                "stream_index": stream_index,
            });
        }

        ffmpeg::media::Type::Attachment => {
            template = "#{{{stream_index}}}: Attachment";
            values = json!({
                "stream_index": stream_index,
            });
        }

        ffmpeg::media::Type::Unknown => {
            template = "#{{{stream_index}}}: Unknown";
            values = json!({
                "stream_index": stream_index,
            });
        }
    };
    Ok(Handlebars::new()
        .render_template(template, &values)
        .expect(&format!(
            "stream {} template rendering failure",
            stream_index
        )))
}
