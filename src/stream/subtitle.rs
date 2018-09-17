use ffmpeg::codec::{self, Parameters};
use ffmpeg::DictionaryRef;
use std::io;

use prejudice;

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
        metadata: &DictionaryRef,
    ) -> io::Result<SubtitleMetadata> {
        let language = metadata
            .get("language")
            .or(metadata.get("LANGUAGE"))
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
