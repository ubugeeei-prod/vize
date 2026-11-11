use super::super::aggregate::should_render_details;
use vize_patina::OutputFormat;

#[test]
fn quiet_mode_still_renders_error_details() {
    for format in [
        OutputFormat::Text,
        OutputFormat::Ansi,
        OutputFormat::Plain,
        OutputFormat::Stylish,
        OutputFormat::Json,
        OutputFormat::Markdown,
        OutputFormat::Html,
        OutputFormat::Agent,
    ] {
        assert!(should_render_details(format, true));
        assert!(should_render_details(format, false));
    }
}
