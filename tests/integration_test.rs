extern crate metadata;

extern crate ffmpeg_next as ffmpeg;
extern crate tempfile;

use metadata::{MediaFileMetadata, Render};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

const LATEST_LAVC_VERSION: u32 = 3815012; // LIBAVCODEC_VERSION_INT for lavc 58.54.100 (FFmpeg 4.2)

macro_rules! media_file_tests {
    (
        $(
            $name:ident: {
                input: $input:expr,
                output: $output:expr,
                $( output_with_checksum: $output_with_checksum:expr, )*
                $( output_with_tags: $output_with_tags:expr, )*
                $( output_with_all_tags: $output_with_all_tags:expr, )*
                $( output_with_frame_decoding: $output_with_frame_decoding:expr, )*
            }
        )*
    ) => {
        $(
            #[test]
            fn $name() {
                ffmpeg::init().unwrap();
                unsafe { ffmpeg::ffi::av_log_set_level(ffmpeg::ffi::AV_LOG_FATAL); }

                let filename = Path::new($input).file_name().unwrap().to_str().unwrap();
                let input = include_bytes!($input);
                let output = include_str!($output);
                let _output_with_checksum: Option<&str> = None;
                let _output_with_tags: Option<&str> = None;
                // let _output_with_all_tags: Option<&str> = None;
                let _output_with_frame_decoding: Option<&str> = None;
                $( let _output_with_checksum = Some(include_str!($output_with_checksum)); )*
                $( let _output_with_tags = Some(include_str!($output_with_tags)); )*
                $( let _output_with_all_tags = Some(include_str!($output_with_all_tags)); )*
                $( let _output_with_frame_decoding = Some(include_str!($output_with_frame_decoding)); )*

                let tmpdir = TempDir::new().unwrap();
                let input_path = tmpdir.path().join(filename);
                let mut infile = File::create(&input_path).unwrap();
                infile.write_all(input).unwrap();
                infile.flush().unwrap();

                let mut meta = MediaFileMetadata::new(&input_path).unwrap();
                assert_eq!(output, meta.render_default().unwrap() + "\n");

                if let Some(output) = _output_with_checksum {
                    meta.include_checksum(true).unwrap();
                    assert_eq!(output, meta.render_default().unwrap() + "\n");
                    meta.include_checksum(false).unwrap();
                }

                if ffmpeg::codec::version() == LATEST_LAVC_VERSION {
                    if let Some(output) = _output_with_tags {
                        meta.include_tags(true);
                        assert_eq!(output, meta.render_default().unwrap() + "\n");
                        meta.include_tags(false);
                    }

                    if let Some(output) = _output_with_all_tags {
                        meta.include_all_tags(true);
                        assert_eq!(output, meta.render_default().unwrap() + "\n");
                        meta.include_all_tags(false);
                        meta.include_tags(false);
                    }
                }

                if let Some(output) = _output_with_frame_decoding {
                    meta.decode_frames(true).unwrap();
                    assert_eq!(output, meta.render_default().unwrap() + "\n");
                    meta.decode_frames(false).unwrap();
                }
            }
        )*
    }
}

media_file_tests! {
    _5_1_side_wav: {
        input: "data/_5_1_side_wav/5.1-side.wav",
        output: "data/_5_1_side_wav/5.1-side.wav.txt",
        output_with_checksum: "data/_5_1_side_wav/5.1-side.wav.with_checksum.txt",
        output_with_all_tags: "data/_5_1_side_wav/5.1-side.wav.with_all_tags.txt",
    }

    _5_1_wav: {
        input: "data/_5_1_wav/5.1.wav",
        output: "data/_5_1_wav/5.1.wav.txt",
        output_with_all_tags: "data/_5_1_wav/5.1.wav.with_all_tags.txt",
    }

    aac_aac: {
        input: "data/aac_aac/aac.aac",
        output: "data/aac_aac/aac.aac.txt",
        output_with_all_tags: "data/aac_aac/aac.aac.with_all_tags.txt",
    }

    aac_he_aac: {
        input: "data/aac_he_aac/aac_he.aac",
        output: "data/aac_he_aac/aac_he.aac.txt",
        output_with_all_tags: "data/aac_he_aac/aac_he.aac.with_all_tags.txt",
    }

    ac3_ac3: {
        input: "data/ac3_ac3/ac3.ac3",
        output: "data/ac3_ac3/ac3.ac3.txt",
        output_with_all_tags: "data/ac3_ac3/ac3.ac3.with_all_tags.txt",
    }

    aiff_aiff: {
        input: "data/aiff_aiff/aiff.aiff",
        output: "data/aiff_aiff/aiff.aiff.txt",
        output_with_all_tags: "data/aiff_aiff/aiff.aiff.with_all_tags.txt",
    }

    flac_flac: {
        input: "data/flac_flac/flac.flac",
        output: "data/flac_flac/flac.flac.txt",
        output_with_all_tags: "data/flac_flac/flac.flac.with_all_tags.txt",
    }

    h264_3g2: {
        input: "data/h264_3g2/h264.3g2",
        output: "data/h264_3g2/h264.3g2.txt",
        output_with_all_tags: "data/h264_3g2/h264.3g2.with_all_tags.txt",
    }

    h264_3gp: {
        input: "data/h264_3gp/h264.3gp",
        output: "data/h264_3gp/h264.3gp.txt",
        output_with_all_tags: "data/h264_3gp/h264.3gp.with_all_tags.txt",
    }

    h264_aac_mp4: {
        input: "data/h264_aac_mp4/h264.aac.mp4",
        output: "data/h264_aac_mp4/h264.aac.mp4.txt",
        output_with_all_tags: "data/h264_aac_mp4/h264.aac.mp4.with_all_tags.txt",
    }

    h264_aac_srt_mkv: {
        input: "data/h264_aac_srt_mkv/h264.aac.srt.mkv",
        output: "data/h264_aac_srt_mkv/h264.aac.srt.mkv.txt",
        output_with_tags: "data/h264_aac_srt_mkv/h264.aac.srt.mkv.with_tags.txt",
        output_with_all_tags: "data/h264_aac_srt_mkv/h264.aac.srt.mkv.with_all_tags.txt",
    }

    h264_ass_mkv: {
        input: "data/h264_ass_mkv/h264.ass.mkv",
        output: "data/h264_ass_mkv/h264.ass.mkv.txt",
        output_with_all_tags: "data/h264_ass_mkv/h264.ass.mkv.with_all_tags.txt",
    }

    h264_flv: {
        input: "data/h264_flv/h264.flv",
        output: "data/h264_flv/h264.flv.txt",
        output_with_all_tags: "data/h264_flv/h264.flv.with_all_tags.txt",
    }

    h264_mov: {
        input: "data/h264_mov/h264.mov",
        output: "data/h264_mov/h264.mov.txt",
        output_with_all_tags: "data/h264_mov/h264.mov.with_all_tags.txt",
    }

    h264_mp4: {
        input: "data/h264_mp4/h264.mp4",
        output: "data/h264_mp4/h264.mp4.txt",
        output_with_all_tags: "data/h264_mp4/h264.mp4.with_all_tags.txt",
    }

    h264_srt_mkv: {
        input: "data/h264_srt_mkv/h264.srt.mkv",
        output: "data/h264_srt_mkv/h264.srt.mkv.txt",
        output_with_all_tags: "data/h264_srt_mkv/h264.srt.mkv.with_all_tags.txt",
    }

    h264_ts: {
        input: "data/h264_ts/h264.ts",
        output: "data/h264_ts/h264.ts.txt",
        output_with_all_tags: "data/h264_ts/h264.ts.with_all_tags.txt",
    }

    h264_high4_0_mp4: {
        input: "data/h264_high4_0_mp4/h264_high4.0.mp4",
        output: "data/h264_high4_0_mp4/h264_high4.0.mp4.txt",
        output_with_all_tags: "data/h264_high4_0_mp4/h264_high4.0.mp4.with_all_tags.txt",
    }

    h264_interlaced_mp4: {
        input: "data/h264_interlaced_mp4/h264_interlaced.mp4",
        output: "data/h264_interlaced_mp4/h264_interlaced.mp4.txt",
        output_with_all_tags: "data/h264_interlaced_mp4/h264_interlaced.mp4.with_all_tags.txt",
        output_with_frame_decoding: "data/h264_interlaced_mp4/h264_interlaced.mp4.with_frame_decoding.txt",
    }

    hevc_mp4: {
        input: "data/hevc_mp4/hevc.mp4",
        output: "data/hevc_mp4/hevc.mp4.txt",
        output_with_all_tags: "data/hevc_mp4/hevc.mp4.with_all_tags.txt",
    }

    jpeg_jpeg: {
        input: "data/jpeg_jpeg/jpeg.jpeg",
        output: "data/jpeg_jpeg/jpeg.jpeg.txt",
        output_with_all_tags: "data/jpeg_jpeg/jpeg.jpeg.with_all_tags.txt",
    }

    mjpeg_mp4: {
        input: "data/mjpeg_mp4/mjpeg.mp4",
        output: "data/mjpeg_mp4/mjpeg.mp4.txt",
        output_with_all_tags: "data/mjpeg_mp4/mjpeg.mp4.with_all_tags.txt",
    }

    mono_wav: {
        input: "data/mono_wav/mono.wav",
        output: "data/mono_wav/mono.wav.txt",
        output_with_all_tags: "data/mono_wav/mono.wav.with_all_tags.txt",
    }

    mp3_jpeg_mp3: {
        input: "data/mp3_jpeg_mp3/mp3.jpeg.mp3",
        output: "data/mp3_jpeg_mp3/mp3.jpeg.mp3.txt",
        output_with_all_tags: "data/mp3_jpeg_mp3/mp3.jpeg.mp3.with_all_tags.txt",
    }

    mp3_mp3: {
        input: "data/mp3_mp3/mp3.mp3",
        output: "data/mp3_mp3/mp3.mp3.txt",
        output_with_all_tags: "data/mp3_mp3/mp3.mp3.with_all_tags.txt",
    }

    mp3_png_mp3: {
        input: "data/mp3_png_mp3/mp3.png.mp3",
        output: "data/mp3_png_mp3/mp3.png.mp3.txt",
        output_with_all_tags: "data/mp3_png_mp3/mp3.png.mp3.with_all_tags.txt",
    }

    mpeg1video_mp4: {
        input: "data/mpeg1video_mp4/mpeg1video.mp4",
        output: "data/mpeg1video_mp4/mpeg1video.mp4.txt",
        output_with_all_tags: "data/mpeg1video_mp4/mpeg1video.mp4.with_all_tags.txt",
    }

    mpeg2video_m2v: {
        input: "data/mpeg2video_m2v/mpeg2video.m2v",
        output: "data/mpeg2video_m2v/mpeg2video.m2v.txt",
        output_with_all_tags: "data/mpeg2video_m2v/mpeg2video.m2v.with_all_tags.txt",
    }

    mpeg2video_mp4: {
        input: "data/mpeg2video_mp4/mpeg2video.mp4",
        output: "data/mpeg2video_mp4/mpeg2video.mp4.txt",
        output_with_all_tags: "data/mpeg2video_mp4/mpeg2video.mp4.with_all_tags.txt",
    }

    mpeg2video_mpg: {
        input: "data/mpeg2video_mpg/mpeg2video.mpg",
        output: "data/mpeg2video_mpg/mpeg2video.mpg.txt",
        output_with_all_tags: "data/mpeg2video_mpg/mpeg2video.mpg.with_all_tags.txt",
    }

    mpeg4_mp4: {
        input: "data/mpeg4_mp4/mpeg4.mp4",
        output: "data/mpeg4_mp4/mpeg4.mp4.txt",
        output_with_all_tags: "data/mpeg4_mp4/mpeg4.mp4.with_all_tags.txt",
    }

    png_png: {
        input: "data/png_png/png.png",
        output: "data/png_png/png.png.txt",
        output_with_all_tags: "data/png_png/png.png.with_all_tags.txt",
    }

    realvideo1_realaudio1_rm: {
        input: "data/realvideo1_realaudio1_rm/realvideo1.realaudio1.rm",
        output: "data/realvideo1_realaudio1_rm/realvideo1.realaudio1.rm.txt",
        output_with_all_tags: "data/realvideo1_realaudio1_rm/realvideo1.realaudio1.rm.with_all_tags.txt",
    }

    stereo_mp3: {
        input: "data/stereo_mp3/stereo.mp3",
        output: "data/stereo_mp3/stereo.mp3.txt",
        output_with_all_tags: "data/stereo_mp3/stereo.mp3.with_all_tags.txt",
    }

    stereo_wav: {
        input: "data/stereo_wav/stereo.wav",
        output: "data/stereo_wav/stereo.wav.txt",
        output_with_all_tags: "data/stereo_wav/stereo.wav.with_all_tags.txt",
    }

    theora_ogv: {
        input: "data/theora_ogv/theora.ogv",
        output: "data/theora_ogv/theora.ogv.txt",
        output_with_all_tags: "data/theora_ogv/theora.ogv.with_all_tags.txt",
    }

    vorbis_oga: {
        input: "data/vorbis_oga/vorbis.oga",
        output: "data/vorbis_oga/vorbis.oga.txt",
        output_with_all_tags: "data/vorbis_oga/vorbis.oga.with_all_tags.txt",
    }

    vp8_webm: {
        input: "data/vp8_webm/vp8.webm",
        output: "data/vp8_webm/vp8.webm.txt",
        output_with_all_tags: "data/vp8_webm/vp8.webm.with_all_tags.txt",
    }

    vp9_webm: {
        input: "data/vp9_webm/vp9.webm",
        output: "data/vp9_webm/vp9.webm.txt",
        output_with_all_tags: "data/vp9_webm/vp9.webm.with_all_tags.txt",
    }
}
