use ffmpeg;
use ffmpeg::codec::context::Context;
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

// An unknown field order, AV_FIELD_UNKNOWN, is treated as a sign of being
// progressive.
pub fn get_scan_type(input: &mut Input) -> io::Result<Option<ScanType>> {
    let stream_index;
    let decoder;
    if let Some(stream) = input.streams().best(ffmpeg::media::Type::Video) {
        stream_index = stream.index();
        let context = Context::from_parameters(stream.parameters())?;
        decoder = context.decoder().video()?;
    } else {
        return Ok(None);
    }
    let field_order = unsafe { (*decoder.as_ptr()).field_order };
    debug!("stream #{} field order: {:?}", stream_index, field_order);
    match field_order {
        AVFieldOrder::AV_FIELD_PROGRESSIVE => Ok(Some(ScanType::Progressive)),
        AVFieldOrder::AV_FIELD_UNKNOWN => Ok(Some(ScanType::LikelyProgressive)),
        AVFieldOrder::AV_FIELD_TT
        | AVFieldOrder::AV_FIELD_BB
        | AVFieldOrder::AV_FIELD_TB
        | AVFieldOrder::AV_FIELD_BT => Ok(Some(ScanType::Interlaced)),
    }
}
