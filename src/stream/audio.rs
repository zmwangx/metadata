use crate::ffmpeg;
use crate::ffmpeg::codec::decoder::audio::Audio;
use crate::ffmpeg::codec::{self, Context, Parameters};
use crate::ffmpeg::util::channel_layout::ChannelLayout;
use crate::ffmpeg::DictionaryRef;
use libc;
use std::io;
use std::str::from_utf8_unchecked;

use crate::prejudice;

#[derive(Clone, Debug, Serialize)]
pub struct AudioMetadata {
    pub index: usize,

    pub language: Option<String>,

    #[serde(skip_serializing)]
    pub _codec: codec::Id,
    pub codec_desc: String,

    #[serde(skip_serializing)]
    pub _sample_rate: u32,
    pub sample_rate: String,

    #[serde(skip_serializing)]
    pub _channel_layout: ChannelLayout,
    pub channel_layout: String,

    #[serde(skip_serializing)]
    pub _bit_rate: Option<u64>,
    pub bit_rate: Option<String>,
}

impl AudioMetadata {
    pub fn new(
        index: usize,
        codec_ctx: Context,
        codec_par: &Parameters,
        tags: &DictionaryRef,
    ) -> io::Result<AudioMetadata> {
        let audio = codec_ctx.decoder().audio()?;

        let language = tags
            .get("language")
            .or_else(|| tags.get("LANGUAGE"))
            .map(str::to_string);

        let _codec = codec_par.id();
        let codec_desc = prejudice::codec_description(&codec_par);

        let _sample_rate = audio.rate();
        let sample_rate = format!("{} Hz", _sample_rate);

        let (_channel_layout, channel_layout) = Self::get_channel_layout(&audio);

        let _bit_rate = match audio.bit_rate() {
            0 => None,
            _ => Some(audio.bit_rate() as u64),
        };
        let bit_rate = _bit_rate.map(|r| format!("{:.0} kb/s", r as f64 / 1000f64));

        Ok(AudioMetadata {
            index,
            language,
            _codec,
            codec_desc,
            _sample_rate,
            sample_rate,
            _channel_layout,
            channel_layout,
            _bit_rate,
            bit_rate,
        })
    }

    fn get_channel_layout(audio: &Audio) -> (ChannelLayout, String) {
        let layout = audio.channel_layout();
        let layout_string: String;
        let mut buf = [0u8; 128];
        unsafe {
            ffmpeg::ffi::av_channel_layout_describe(
                &layout.0 as *const ffmpeg::ffi::AVChannelLayout,
                buf.as_mut_ptr() as *mut libc::c_char,
                128,
            );
            layout_string = from_utf8_unchecked(&buf)
                .trim_end_matches(char::from(0))
                .to_string();
        }
        (layout, layout_string)
    }
}
