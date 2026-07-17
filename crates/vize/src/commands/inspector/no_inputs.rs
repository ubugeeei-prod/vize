use super::InspectorOutputFormat;

pub(super) fn handle(format: InspectorOutputFormat) {
    eprintln!("No .vue files found matching the patterns");
    if !matches!(format, InspectorOutputFormat::Json) {
        std::process::exit(1);
    }
}
