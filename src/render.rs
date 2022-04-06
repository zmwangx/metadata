use handlebars::{self, Handlebars};
use serde::Serialize;

use crate::media_file::MediaFileMetadata;
use crate::stream::{self, StreamMetadata};

pub trait Render: Serialize {
    fn render(&self, template: &str) -> Result<String, handlebars::TemplateRenderError> {
        Handlebars::new().render_template(template, &self)
    }

    fn default_template() -> String;

    fn render_default(&self) -> Result<String, handlebars::TemplateRenderError> {
        self.render(&Self::default_template())
    }
}

// The width (20) here really should not be hard-coded, but the
// handlebars_helper! macro does not seem to support inline helpers with
// additional arguments (a statement I basically pulled out of my ass).
handlebars_helper!(padkey: |key: str| format!("{:<20}", &[key, ": "].join("")));

impl Render for MediaFileMetadata {
    fn render(&self, template: &str) -> Result<String, handlebars::TemplateRenderError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_helper("padkey", Box::new(padkey));
        handlebars.render_template(template, &self)
    }

    fn default_template() -> String {
        "\
         {{#if title}}\
         Title:                  {{{title}}}\n\
         {{/if}}\
         Filename:               {{{file_name}}}\n\
         File size:              {{{file_size}}} ({{{file_size_base10}}}, {{{file_size_base2}}})\n\
         {{#if options.include_checksum and hash }}\
         SHA-256 digest:         {{{hash}}}\n\
         {{/if}}\
         Container format:       {{{container_format}}}\n\
         Duration:               {{#if duration}}{{{duration}}}{{else}}Not available{{/if}}\n\
         {{#if pixel_dimensions}}\
         Pixel dimensions:       {{{pixel_dimensions}}}\n\
         {{/if}}\
         {{#if sample_aspect_ratio}}\
         Sample aspect ratio:    {{{sample_aspect_ratio}}}\n\
         {{/if}}\
         {{#if display_aspect_ratio}}\
         Display aspect ratio:   {{{display_aspect_ratio}}}\n\
         {{/if}}\
         {{#if scan_type}}\
         Scan type:              {{{scan_type}}}\n\
         {{/if}}\
         {{#if frame_rate}}\
         Frame rate:             {{{frame_rate}}}\n\
         {{/if}}\
         Bit rate:               {{{bit_rate}}}\n\
         Streams:\n\
         {{#each streams_metadata_rendered as |stream_metadata|}}    {{{stream_metadata}}}\n{{/each}}\
         \
         {{#if options.include_all_tags}}\
           {{#if tags}}\
             Tags:\n\
             {{#each tags as |kv|}}    {{padkey kv.0}}{{{kv.1}}}\n{{/each}}\
           {{/if}}\
           {{#each streams_tags as |s|}}\
             {{#if s.tags}}  #{{{s.index}}}\n\
             {{#each s.tags as |kv|}}    {{padkey kv.0}}{{{kv.1}}}\n{{/each}}\
             {{/if}}\
           {{/each}}\
         {{else}}{{#if options.include_tags}}\
           {{#if filtered_tags}}\
             Tags:\n\
             {{#each filtered_tags as |kv|}}    {{padkey kv.0}}{{{kv.1}}}\n{{/each}}\
           {{/if}}\
           {{#each streams_filtered_tags as |s|}}\
             {{#if s.tags}}  #{{{s.index}}}\n\
             {{#each s.tags as |kv|}}    {{padkey kv.0}}{{{kv.1}}}\n{{/each}}\
             {{/if}}\
           {{/each}}\
         {{/if}}{{/if}}\
         "
            .to_string()
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
