use crate::ffmpeg::DictionaryRef;
use regex::Regex;

pub type Tags = Vec<(String, String)>;

pub trait ToTags {
    fn to_tags(&self) -> Tags;

    fn to_filtered_tags(&self) -> Tags {
        self.to_tags()
            .iter()
            .filter(|(k, _)| !Self::tag_is_boring(&k))
            .cloned()
            .collect()
    }

    fn tag_is_boring(key: &str) -> bool {
        lazy_static! {
            // Some fixed names, plus tags beginning with an underscore (e.g.,
            // _STATISTICS_* tags by mkvmerge), or in reversed domain name notation
            // (e.g., com.apple.quicktime.player.* tags).
            static ref BORING_PATTERN: Regex = Regex::new(r"(?i)^((major_brand|minor_version|compatible_brands|creation_time|handler_name|encoder)$|_|com\.)").unwrap();
        }
        BORING_PATTERN.is_match(key)
    }
}

impl<'a> ToTags for DictionaryRef<'a> {
    fn to_tags(&self) -> Vec<(String, String)> {
        self.iter()
            // Filter out empty tags.
            // With FFmpeg 3.4.4, empty tags could be exposed, but this was
            // fixed in 4.0. We filter out empty tags for consistency.
            // Example: tests/data/realvideo1_realaudio1_rm/realvideo1.realaudio1.rm
            .filter_map(|(k, v)| {
                if v.is_empty() {
                    None
                } else {
                    Some((k.to_string(), v.to_string()))
                }
            })
            .collect()
    }
}
