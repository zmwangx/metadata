// Prejudiced, or slightly more consistent, names of common format and
// codecs. "Common" is based on my personal experience. Some common
// formats and codecs are not listed here because the stock names are
// good enough.

use crate::ffmpeg;
use crate::ffmpeg::codec::parameters::Parameters;
use crate::ffmpeg::codec::profile::{Profile, AAC, H264, HEVC, VP9};
use crate::ffmpeg::codec::Id;
use crate::ffmpeg::format::Input;
use std::ffi::CStr;
use std::path::Path;
use std::str::from_utf8_unchecked;

// Assumes ASCII!
fn capitalize(s: String) -> String {
    if s == "" {
        return "".to_string();
    }
    s.chars().next().unwrap().to_ascii_uppercase().to_string() + &(s.clone())[1..s.len()]
}

pub fn format_name(format: &Input, path: &Path) -> String {
    let extension = path
        .extension()
        .map_or("", |s| s.to_str().unwrap())
        .to_ascii_lowercase();
    let uppercase_extension = extension.to_ascii_uppercase();
    // The original AVInputFormat.long_name is shown in the comment
    // above each entry. (ffmpeg -formats)
    match format.name() {
        // "raw ADTS AAC (Advanced Audio Coding)"
        "aac" => "Raw ADTS AAC",
        // "Audio IFF"
        "aiff" => "Audio Interchange File Format (AIFF)",
        // "ASF (Advanced / Active Streaming Format)"
        "asf" => "Advanced Systems Format (ASF)",
        // "SSA (SubStation Alpha) subtitle"
        "ass" => "Advanced SubStation Alpha (ASS)",
        // "FLV (Flash Video)""
        "flv" => "Flash Video (FLV)",
        // "piped jpeg sequence"
        "jpeg_pipe" => "JPEG",
        // "Matroska / WebM"
        "matroska,webm" => match &extension as &str {
            ".webm" => "WebM",
            _ => "Matroska (MKV)",
        },
        // "MP2/3 (MPEG audio layer 2/3)"
        "mp3" => "MP3",
        // "MPEG-PS (MPEG-2 Program Stream)"
        "mpeg" => "MPEG-2 Program Stream (MPEG-PS)",
        // "MPEG-TS (MPEG-2 Transport Stream)"
        "mpegts" => "MPEG-2 Transport Stream (MPEG-TS)",
        // "QuickTime / MOV"
        "mov,mp4,m4a,3gp,3g2,mj2" => match &extension as &str {
            "mov" | "qt" => "QuickTime File Format",
            "3gp" => "3GPP",
            "3g2" => "3GPP2",
            "mj2" | "mjp2" => "Motion JPEG 2000",
            _ => return format!("MPEG-4 Part 14 ({})", uppercase_extension),
        },
        // "piped png sequence"
        "png_pipe" => "PNG",
        // "RealText subtitle format"
        "realtext" => "RealText",
        // "SAMI subtitle format"
        "sami" => "Synchronized Accessible Media Interchange (SAMI)",
        // "SubRip subtitle"
        "srt" => "SubRip",
        // "SubViewer subtitle format"
        "subviewer" => "SubViewer",
        // "SubViewer v1 subtitle format"
        "subviewer1" => "SubViewer v1",
        // "WAV / WAVE (Waveform Audio)"
        "wav" => "Waveform Audio (WAV)",
        // "WebVTT subtitle"
        "webvtt" => "WebVTT",
        _ => return capitalize(format.description().to_string()),
    }
    .to_string()
}

pub fn codec_name(codec_id: Id) -> String {
    let codec_descriptor = unsafe { ffmpeg::ffi::avcodec_descriptor_get(codec_id.into()) };
    let name = unsafe { from_utf8_unchecked(CStr::from_ptr((*codec_descriptor).name).to_bytes()) };
    let long_name =
        unsafe { from_utf8_unchecked(CStr::from_ptr((*codec_descriptor).long_name).to_bytes()) };
    // The original AVCodec.long_name is shown in the comment above each
    // entry. (ffmpeg -codecs)
    match name {
        // Video codecs
        // "H.264 / AVC / MPEG-4 AVC / MPEG-4 part 10"
        "h264" => "H.264",
        // "H.265 / HEVC (High Efficiency Video Coding)"
        "hevc" => "HEVC",
        // "MPEG-4 part 2"
        "mpeg4" => "MPEG-4 Part 2",
        // "PNG (Portable Network Graphics) image"
        "png" => "PNG",
        // "On2 VP8"
        "vp8" => "VP8",
        // "Google VP9"
        "vp9" => "VP9",

        // Audio codecs
        // "AAC (Advanced Audio Coding)"
        "aac" => "AAC",
        // "ATSC A/52A (AC-3)"
        "ac3" => "Dolby AC-3",
        // "Cook / Cooker / Gecko (RealAudio G2)"
        "cook" => "Cook (RealAudio G2)",
        // "FLAC (Free Lossless Audio Codec)"
        "flac" => "FLAC",
        // "MP3 (MPEG audio layer 3)"
        "mp3" => "MP3",
        // "Opus (Opus Interactive Audio Codec)"
        "opus" => "Opus",
        // "RealAudio 1.0 (14.4K)"
        "ra_144" => "RealAudio 1.0",
        // "RealAudio 2.0 (28.8K)"
        "ra_288" => "RealAudio 2.0",

        // Subtitle codecs
        // "ASS (Advanced SSA) subtitle"
        "ass" => "Advanced SubStation Alpha (ASS)",
        // "RealText subtitle"
        "realtext" => "RealText",
        // "SAMI subtitle"
        "sami" => "Synchronized Accessible Media Interchange (SAMI)",
        // "SubRip subtitle with embedded timing"
        "srt" => "SubRip",
        // "SSA (SubStation Alpha) subtitle"
        "ssa" => "SubStation Alpha (SSA)",
        // "SubRip subtitle"
        "subrip" => "SubRip",
        // "SubViewer subtitle format"
        "subviewer" => "SubViewer",
        // "SubViewer v1 subtitle format"
        "subviewer1" => "SubViewer v1",
        // "WebVTT subtitle"
        "webvtt" => "WebVTT",

        _ => return capitalize(long_name.to_string()),
    }
    .to_string()
}

pub fn codec_description(codec_par: &Parameters) -> String {
    let codec_id = codec_par.id();
    let name = codec_name(codec_id);
    let profile = Profile::from((codec_id, unsafe { (*codec_par.as_ptr()).profile }));
    let level = unsafe { (*codec_par.as_ptr()).level };
    match codec_id {
        Id::H264 => {
            let profile_name = if let Profile::H264(p) = profile {
                match p {
                    H264::Constrained => "Constrained Profile",
                    H264::Intra => "Intra Profile",
                    H264::Baseline => "Baseline Profile",
                    H264::ConstrainedBaseline => "Constrained Baseline Profile",
                    H264::Main => "Main Profile",
                    H264::Extended => "Extended Profile",
                    H264::High => "High Profile",
                    H264::High10 => "High 10 Profile",
                    H264::High10Intra => "High 10 Intra Profile",
                    H264::High422 => "High 4:2:2 Profile",
                    H264::High422Intra => "High 4:2:2 Intra Profile",
                    H264::High444 => "High 4:4:4 Profile",
                    H264::High444Predictive => "High 4:4:4 Predictive Profile",
                    H264::High444Intra => "High 4:4:4 Intra Profile",
                    H264::CAVLC444 => "CAVLC 4:4:4 Profile",
                }
            } else {
                "Unknown Profile"
            };
            let level_name = if level % 10 == 0 {
                format!("{}", level / 10)
            } else {
                format!("{:.1}", level as f64 / 10f64)
            };
            format!("{} ({} level {})", name, profile_name, level_name)
        }

        Id::HEVC => {
            let profile_name = if let Profile::HEVC(p) = profile {
                match p {
                    HEVC::Main => "Main Profile",
                    HEVC::Main10 => "Main 10 Profile",
                    HEVC::MainStillPicture => "Main Still Picture Profile",
                    HEVC::Rext => "Range Extension (RExt)",
                }
            } else {
                "Unknown Profile"
            };
            let level_name = if level % 30 == 0 {
                format!("{}", level / 30)
            } else {
                format!("{:.1}", level as f64 / 30f64)
            };
            format!("{} ({} level {})", name, profile_name, level_name)
        }

        Id::VP9 => {
            let profile_name = if let Profile::VP9(p) = profile {
                match p {
                    VP9::_0 => "Profile 0",
                    VP9::_1 => "Profile 1",
                    VP9::_2 => "Profile 2",
                    VP9::_3 => "Profile 3",
                }
            } else {
                "Unknown Profile"
            };
            // libavcodec does not seem able to detect VP9 level (gives
            // -99 on my samples).
            format!("{} ({})", name, profile_name)
        }

        Id::AAC => {
            let profile_name = if let Profile::AAC(p) = profile {
                match p {
                    AAC::Main => "Main Profile",
                    AAC::Low => "LC",  // Low Complexity
                    AAC::SSR => "SSR", // Scalable Sample Rate
                    AAC::LTP => "LTP", // Long Term Prediction
                    AAC::HE => "HE-AAC",
                    AAC::HEv2 => "HE-AAC v2",
                    AAC::LD => "LD",   // Low Decay
                    AAC::ELD => "ELD", // Enhanced Low Decay
                    AAC::MPEG2Low => "MPEG-2 LC",
                    AAC::MPEG2HE => "MPEG-2 HE-AAC",
                }
            } else {
                "Unknown Profile"
            };
            format!("{} ({})", name, profile_name)
        }

        _ => name,
    }
}
