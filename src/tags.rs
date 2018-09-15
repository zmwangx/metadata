use ffmpeg::DictionaryRef;
use regex::Regex;

pub trait Tags {
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
