use ffmpeg;
use ffmpeg::ffi::AVFieldOrder;
use ffmpeg::format::context::Input;
use std::fmt;
use std::io;

#[derive(Clone, Debug)]
pub enum ScanType {
    Progressive,
    LikelyProgressive,
    Interlaced,
}

impl fmt::Display for ScanType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScanType::Progressive => write!(f, "Progressive scan"),
            ScanType::LikelyProgressive => write!(f, "Progressive scan*"),
            ScanType::Interlaced => write!(f, "Interlaced scan"),
        }
    }
}

const NUM_FRAMES_TO_INSPECT: usize = 30;

// An unknown field order, AV_FIELD_UNKNOWN, is treated as a sign of being
// progressive, unless decode_frames is true, in which case the first 30 frames
// are decoded to look for interlaced frames.
pub fn get_scan_type(input: &mut Input, decode_frames: bool) -> io::Result<Option<ScanType>> {
    let stream_index;
    let mut decoder;
    if let Some(stream) = input.streams().best(ffmpeg::media::Type::Video) {
        stream_index = stream.index();
        decoder = stream.codec().decoder().video()?;
    } else {
        return Ok(None);
    }
    let field_order = unsafe { (*decoder.as_ptr()).field_order };
    debug!("stream #{} field order: {:?}", stream_index, field_order);
    match field_order {
        AVFieldOrder::AV_FIELD_PROGRESSIVE => return Ok(Some(ScanType::Progressive)),
        AVFieldOrder::AV_FIELD_UNKNOWN => {
            if !decode_frames {
                return Ok(Some(ScanType::LikelyProgressive));
            }
        }
        AVFieldOrder::AV_FIELD_TT
        | AVFieldOrder::AV_FIELD_BB
        | AVFieldOrder::AV_FIELD_TB
        | AVFieldOrder::AV_FIELD_BT => return Ok(Some(ScanType::Interlaced)),
    }
    let mut decoded = ffmpeg::frame::Video::empty();
    let mut frame_count: usize = 0;
    for (stream, packet) in input.packets() {
        if stream.index() == stream_index {
            frame_count += 1;
            debug!("decoding frame {}", frame_count);
            if let Ok(true) = decoder.decode(&packet, &mut decoded) {
                if decoded.is_interlaced() {
                    return Ok(Some(ScanType::Interlaced));
                }
            }
            if frame_count >= NUM_FRAMES_TO_INSPECT {
                break;
            }
        }
    }
    Ok(Some(ScanType::Progressive))
}
