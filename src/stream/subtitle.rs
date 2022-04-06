use crate::ffmpeg::codec::{self, Parameters};
use crate::ffmpeg::DictionaryRef;
use std::io;

use crate::prejudice;

#[derive(Clone, Debug, Serialize)]
pub struct SubtitleMetadata {
    pub index: usize,

    pub language: Option<String>,

    #[serde(skip_serializing)]
    pub _codec: codec::Id,
    pub codec_desc: String,
}

impl SubtitleMetadata {
    pub fn new(
        index: usize,
        codec_par: &Parameters,
        tags: &DictionaryRef,
    ) -> io::Result<SubtitleMetadata> {
        let language = tags
            .get("language")
            .or_else(|| tags.get("LANGUAGE"))
            .map(str::to_string);

        let _codec = codec_par.id();
        let codec_desc = prejudice::codec_description(&codec_par);

        Ok(SubtitleMetadata {
            index,
            language,
            _codec,
            codec_desc,
        })
    }
}
