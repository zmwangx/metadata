use crate::ffmpeg::codec::{self, Context, Parameters};
use crate::ffmpeg::color;
use crate::ffmpeg::util::format::pixel::Pixel;
use crate::ffmpeg::util::rational::Rational;
use crate::ffmpeg::Stream;
use std::io;

use crate::prejudice;

#[derive(Clone, Debug, Serialize)]
pub struct VideoMetadata {
    pub index: usize,

    #[serde(skip_serializing)]
    pub _codec: codec::Id,
    pub codec_desc: String,
    #[serde(skip_serializing)]
    pub _pixel_fmt: Pixel,
    pub pixel_fmt: Option<String>,
    #[serde(skip_serializing)]
    pub _color_range: color::Range,
    pub color_range: Option<String>,
    #[serde(skip_serializing)]
    pub _color_space: color::Space,
    pub color_space: Option<String>,
    #[serde(skip_serializing)]
    pub _color_primaries: color::Primaries,
    pub color_primaries: Option<String>,
    #[serde(skip_serializing)]
    pub _color_trc: color::TransferCharacteristic,
    pub color_trc: Option<String>,
    pub color_spec_str: String,

    pub width: u32,
    pub height: u32,
    pub pixel_dimensions: String,
    #[serde(skip_serializing)]
    pub _sample_aspect_ratio: Rational,
    pub sample_aspect_ratio: String,
    #[serde(skip_serializing)]
    pub _display_aspect_ratio: Rational,
    pub display_aspect_ratio: String,

    #[serde(skip_serializing)]
    pub _frame_rate: Option<Rational>,
    pub frame_rate: Option<String>,

    #[serde(skip_serializing)]
    pub _bit_rate: Option<u64>,
    pub bit_rate: Option<String>,
}

impl VideoMetadata {
    pub fn new(
        index: usize,
        stream: Stream,
        codec_ctx: Context,
        codec_par: &Parameters,
    ) -> io::Result<VideoMetadata> {
        let video = codec_ctx.decoder().video()?;

        let _codec = codec_par.id();
        let codec_desc = prejudice::codec_description(&codec_par);

        let _pixel_fmt = video.format();
        let pixel_fmt = _pixel_fmt.descriptor().map(|d| d.name().to_string());
        let _color_range = video.color_range();
        let color_range = _color_range.name();
        let _color_space = video.color_space();
        let color_space = _color_space.name();
        let _color_primaries = video.color_primaries();
        let color_primaries = _color_primaries.name();
        let _color_trc = video.color_transfer_characteristic();
        let color_trc = _color_trc.name();
        let color_spec_str = [color_range.map(|v| v.to_string()), {
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
        }]
        .iter()
        .filter(|v| v.is_some())
        .map(|v| v.clone().unwrap())
        .collect::<Vec<_>>()
        .join(", ");

        // sar is the sample aspect ratio (aka pixel aspect ratio); dar is the
        // display aspect ratio. The following is satisfied:
        //
        //     width/height * sar = dar
        //
        // width:height is also sometimes called the storage aspect ratio; do
        // not confuse with sar.
        //
        // If sample_aspect_ratio is not available (i.e., 1:1),
        // video.aspect_ratio() produces Rational(0, 1), so apparently we need
        // to correct this.
        let width = video.width();
        let height = video.height();
        let pixel_dimensions = format!("{}x{}", width, height);
        let _sar = video.aspect_ratio();
        let _sar = match _sar.numerator() {
            0 => Rational(1, 1),
            _ => _sar.reduce(),
        };
        let sar = format!("{}:{}", _sar.numerator(), _sar.denominator());
        let _dar = _sar * Rational(width as i32, height as i32);
        let dar = format!("{}:{}", _dar.numerator(), _dar.denominator());

        let _frame_rate = stream.avg_frame_rate();
        let _frame_rate = match _frame_rate.denominator() {
            0 => None,
            _ => Some(_frame_rate.reduce()),
        };
        let frame_rate = _frame_rate.map(|rate| match rate.denominator() {
            1 => format!("{} fps", rate.numerator()),
            _ => format!(
                "{:.2} fps",
                rate.numerator() as f64 / rate.denominator() as f64
            ),
        });

        let _bit_rate = match video.bit_rate() {
            0 => None,
            _ => Some(video.bit_rate() as u64),
        };
        let bit_rate = _bit_rate.map(|r| format!("{:.0} kb/s", r as f64 / 1000f64));

        Ok(VideoMetadata {
            index,
            _codec,
            codec_desc,
            _pixel_fmt,
            pixel_fmt,
            _color_range,
            color_range: color_range.map(str::to_string),
            _color_space,
            color_space: color_space.map(str::to_string),
            _color_primaries,
            color_primaries: color_primaries.map(str::to_string),
            _color_trc,
            color_trc: color_trc.map(str::to_string),
            color_spec_str,
            width,
            height,
            pixel_dimensions,
            _sample_aspect_ratio: _sar,
            sample_aspect_ratio: sar,
            _display_aspect_ratio: _dar,
            display_aspect_ratio: dar,
            _frame_rate,
            frame_rate,
            _bit_rate,
            bit_rate,
        })
    }
}
