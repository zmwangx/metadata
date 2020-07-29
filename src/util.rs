use sha2::{Digest, Sha256};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub enum Base {
    Base2,  // IEC
    Base10, // SI
}

fn round_up(value: f64, precision: usize) -> f64 {
    let multiplier = 10u64.pow(precision as u32) as f64;
    (value * multiplier).ceil() / multiplier
}

pub fn human_size(bytes: u64, base: Base) -> String {
    let multiplier = match base {
        Base::Base2 => 1024f64,
        Base::Base10 => 1000f64,
    };
    let units = match base {
        Base::Base2 => ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB"],
        Base::Base10 => ["KB", "MB", "GB", "TB", "PB", "EB", "ZB"],
    };
    let mut size = bytes as f64;
    if size < multiplier {
        return format!("{:.0}B", size);
    };
    for unit in &units {
        size /= multiplier;
        let precision: usize;
        if size < multiplier {
            if size < 10f64 {
                precision = 2;
            } else if size < 100f64 {
                precision = 1;
            } else {
                precision = 0;
            }
            return format!("{:.*}{}", precision, round_up(size, precision), unit);
        }
    }
    format!("{:.0}{}", size.ceil(), units[units.len() - 1])
}

pub fn format_seconds(secs: f64) -> String {
    let seconds = secs % 60f64;
    let minutes = (secs / 60f64).floor() % 60f64;
    let hours = (secs / 3600f64).floor();
    format!("{:02.0}:{:02.0}:{:05.2}", hours, minutes, seconds)
}

pub fn sha256_hash(path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};
    use regex::{Captures, Regex};
    use std;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn human_size_gives_valid_range(bytes: u64, base: Base) -> (String, bool) {
        let hs = human_size(bytes, base);
        lazy_static! {
            static ref BASE2_PATTERN: Regex =
                Regex::new(r"^(?P<num>\d+(\.(?P<decimal>\d+))?)(?P<unit>([KMGTPEZ]i)?B)$").unwrap();
            static ref BASE10_PATTERN: Regex =
                Regex::new(r"^(?P<num>\d+(\.(?P<decimal>\d+))?)(?P<unit>[KMGTPEZ]?B)$").unwrap();
        }
        let caps_or_not: Option<Captures>;
        match base {
            Base::Base2 => {
                caps_or_not = BASE2_PATTERN.captures(&hs);
            }
            Base::Base10 => {
                caps_or_not = BASE10_PATTERN.captures(&hs);
            }
        };
        if let Some(caps) = caps_or_not {
            let num: f64 = caps.name("num").map_or("", |m| m.as_str()).parse().unwrap();
            let precision = caps.name("decimal").map_or("", |m| m.as_str()).len();
            let unit = caps.name("unit").map_or("", |m| m.as_str());
            let multiplier = match base {
                Base::Base2 => 1024u64.pow(match unit {
                    "B" => 0,
                    "KiB" => 1,
                    "MiB" => 2,
                    "GiB" => 3,
                    "TiB" => 4,
                    "PiB" => 5,
                    "EiB" => 6,
                    "ZiB" => 7,
                    _ => panic!(),
                }),
                Base::Base10 => 1000u64.pow(match unit {
                    "B" => 0,
                    "KB" => 1,
                    "MB" => 2,
                    "GB" => 3,
                    "TB" => 4,
                    "PB" => 5,
                    "EB" => 6,
                    "ZB" => 7,
                    _ => panic!(),
                }),
            } as f64;
            let range_upper = (multiplier * num) as u64;
            let range_lower = (multiplier * (num * 10f64.powi(precision as i32) - 1f64)
                / 10f64.powi(precision as i32)) as u64;
            // In theory bytes should be strictly greater than range_lower, but
            // we relax the restrictions a bit to allow some floating point
            // imprecision, e.g. for 1100B => 1.11KB.
            (hs, bytes >= range_lower && bytes <= range_upper)
        } else {
            (hs, false)
        }
    }

    #[test]
    fn human_size_gives_valid_range_fixed() {
        // Test some known problematic inputs previously caught by quickcheck.
        for &(bytes, base) in [(1100, Base::Base10), (4730, Base::Base10)].iter() {
            let (hs, valid) = human_size_gives_valid_range(bytes, base);
            assert!(valid, "invalid human size {:?} for {}B in {:?}", hs, bytes, base);
        }
    }

    // Wrap u64 with a custom, smarter generator, since the default
    // generator tend to generate small values and does not suit our
    // needs.
    #[derive(Clone, Debug)]
    struct U64 {
        value: u64,
    }

    impl Arbitrary for U64 {
        fn arbitrary<G: Gen>(g: &mut G) -> U64 {
            U64 {
                value: 2f64.powf(8f64 + 55f64 * g.next_u32() as f64 / (std::u32::MAX as f64 + 1f64))
                    as u64,
            }
        }
    }

    impl Arbitrary for Base {
        fn arbitrary<G: Gen>(g: &mut G) -> Base {
            match g.next_u32() % 2 {
                0 => Base::Base2,
                1 => Base::Base10,
                _ => panic!(),
            }
        }
    }

    quickcheck! {
        fn human_size_gives_valid_range_arbitrary(bytes: U64, base: Base) -> bool {
            human_size_gives_valid_range(bytes.value, base).1
        }
    }

    #[test]
    fn sha256_hash_returns_correct_hash() {
        let file = NamedTempFile::new().unwrap();
        file.as_file().write_all(&[0u8; 65536]).unwrap();
        assert_eq!(
            // head -c 65536 /dev/zero | sha256sum
            "de2f256064a0af797747c2b97505dc0b9f3df0de4f489eac731c23ae9ca9cc31",
            sha256_hash(file.path()).unwrap()
        );
    }
}
