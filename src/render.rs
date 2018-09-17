use handlebars::{self, Handlebars};
use serde::Serialize;

use stream::{self, StreamMetadata};

pub trait Render: Serialize {
    fn render(&self, template: &str) -> Result<String, handlebars::TemplateRenderError> {
        Handlebars::new().render_template(template, &self)
    }

    fn default_template() -> String;

    fn render_default(&self) -> Result<String, handlebars::TemplateRenderError> {
        self.render(&Self::default_template())
    }
}

// StreamMetadata renders to a one-line string similar to avcodec_string
// (libavcodec/utils.c), which is used by ffmpeg/ffprobe's to display stream
// info.
// https://ffmpeg.org/doxygen/4.0/group__lavc__misc.html#ga6d4056568b5ab73d2e55800d9a5caa66
// https://github.com/FFmpeg/FFmpeg/blob/n4.0.2/libavcodec/utils.c#L1167

impl StreamMetadata {
    pub fn render_default(&self) -> Result<String, handlebars::TemplateRenderError> {
        match self {
            StreamMetadata::VideoMetadata(m) => m.render_default(),
            StreamMetadata::AudioMetadata(m) => m.render_default(),
            StreamMetadata::SubtitleMetadata(m) => m.render_default(),
            StreamMetadata::DataMetadata(m) => m.render_default(),
            StreamMetadata::AttachmentMetadata(m) => m.render_default(),
            StreamMetadata::UnknownMetadata(m) => m.render_default(),
        }
    }
}

impl Render for stream::VideoMetadata {
    fn default_template() -> String {
        "#{{{index}}}: Video\
         , {{{codec_desc}}}\
         {{#if pixel_fmt}}\
         , {{{pixel_fmt}}}{{#if color_spec_str}} ({{{color_spec_str}}}){{/if}}\
         {{/if}}\
         , {{{pixel_dimensions}}} \
         (SAR {{{sample_aspect_ratio}}}, DAR {{{display_aspect_ratio}}})\
         {{#if frame_rate}}\
         , {{{frame_rate}}}\
         {{/if}}\
         {{#if bit_rate}}\
         , {{{bit_rate}}}\
         {{/if}}\
         "
            .to_string()
    }
}

// TODO: render unknown language as und

impl Render for stream::AudioMetadata {
    fn default_template() -> String {
        "#{{{index}}}: Audio \
         ({{#if language}}{{{language}}}{{else}}und{{/if}})\
         , {{{codec_desc}}}\
         , {{{sample_rate}}}\
         , {{{channel_layout}}}\
         {{#if bit_rate}}\
         , {{{bit_rate}}}\
         {{/if}}\
         "
            .to_string()
    }
}

impl Render for stream::SubtitleMetadata {
    fn default_template() -> String {
        "#{{{index}}}: Subtitle \
         ({{#if language}}{{{language}}}{{else}}und{{/if}})\
         , {{{codec_desc}}}\
         "
            .to_string()
    }
}

impl Render for stream::DataMetadata {
    fn default_template() -> String {
        "#{{{index}}}: Data".to_string()
    }
}

impl Render for stream::AttachmentMetadata {
    fn default_template() -> String {
        "#{{{index}}}: Attachment".to_string()
    }
}

impl Render for stream::UnknownMetadata {
    fn default_template() -> String {
        "#{{{index}}}: Unknown".to_string()
    }
}
