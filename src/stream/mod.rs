use crate::ffmpeg::codec::context::Context;
use crate::ffmpeg::media::Type;
use crate::ffmpeg::Stream;
use std::io;

pub mod video;
pub use self::video::VideoMetadata;

pub mod audio;
pub use self::audio::AudioMetadata;

pub mod subtitle;
pub use self::subtitle::SubtitleMetadata;

#[derive(Clone, Debug)]
pub enum StreamMetadata {
    VideoMetadata(VideoMetadata),
    AudioMetadata(AudioMetadata),
    SubtitleMetadata(SubtitleMetadata),
    DataMetadata(DataMetadata),
    AttachmentMetadata(AttachmentMetadata),
    UnknownMetadata(UnknownMetadata),
}

#[derive(Clone, Debug, Serialize)]
pub struct DataMetadata {
    pub index: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct AttachmentMetadata {
    pub index: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct UnknownMetadata {
    pub index: usize,
}

impl StreamMetadata {
    pub fn index(&self) -> usize {
        match self {
            StreamMetadata::VideoMetadata(m) => m.index,
            StreamMetadata::AudioMetadata(m) => m.index,
            StreamMetadata::SubtitleMetadata(m) => m.index,
            StreamMetadata::DataMetadata(m) => m.index,
            StreamMetadata::AttachmentMetadata(m) => m.index,
            StreamMetadata::UnknownMetadata(m) => m.index,
        }
    }

    pub fn video_metadata(&self) -> Option<VideoMetadata> {
        match self {
            StreamMetadata::VideoMetadata(m) => Some(m.clone()),
            _ => None,
        }
    }

    pub fn audio_metadata(&self) -> Option<AudioMetadata> {
        match self {
            StreamMetadata::AudioMetadata(m) => Some(m.clone()),
            _ => None,
        }
    }

    pub fn subtitle_metadata(&self) -> Option<SubtitleMetadata> {
        match self {
            StreamMetadata::SubtitleMetadata(m) => Some(m.clone()),
            _ => None,
        }
    }
}

pub fn parse_stream_meatadata(stream: Stream) -> io::Result<StreamMetadata> {
    let index = stream.index();
    let codec_ctx = Context::from_parameters(stream.parameters())?;
    let codec_par = stream.parameters();
    let tags = stream.metadata();
    Ok(match codec_ctx.medium() {
        Type::Video => {
            StreamMetadata::VideoMetadata(VideoMetadata::new(index, stream, codec_ctx, &codec_par)?)
        }
        Type::Audio => {
            StreamMetadata::AudioMetadata(AudioMetadata::new(index, codec_ctx, &codec_par, &tags)?)
        }
        Type::Subtitle => {
            StreamMetadata::SubtitleMetadata(SubtitleMetadata::new(index, &codec_par, &tags)?)
        }
        Type::Data => StreamMetadata::DataMetadata(DataMetadata { index }),
        Type::Attachment => StreamMetadata::AttachmentMetadata(AttachmentMetadata { index }),
        Type::Unknown => StreamMetadata::UnknownMetadata(UnknownMetadata { index }),
    })
}
