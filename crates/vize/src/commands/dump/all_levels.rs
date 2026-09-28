//! JSON export of the native L1 → L2 → L3 stage ladder.

use std::path::Path;

use vize_curator::inspector::{StageFeed, ladder_run};

pub(super) fn run(path: &Path) {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("vue") => {
            eprintln!(
                "dump: {}: Vue SFC capture awaits the native L1 container; pass a raw template file",
                path.display()
            );
            std::process::exit(1);
        }
        Some("pug" | "jade") => {
            eprintln!(
                "dump: {}: Pug stage capture is not wired to the native ladder yet",
                path.display()
            );
            std::process::exit(1);
        }
        _ => {}
    }
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("dump: cannot read {}: {error}", path.display());
            std::process::exit(1);
        }
    };
    let result = ladder_run(&path.display().to_string(), &source, &|| 0);
    let feed = StageFeed {
        command: "vize-dump".into(),
        pages: result.pages,
        remarks: result.remarks,
    };
    print!("{}", feed.to_json());
}
