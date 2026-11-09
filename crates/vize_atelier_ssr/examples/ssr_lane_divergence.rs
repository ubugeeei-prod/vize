//! Reducer predicate: exits 0 when the selected and legacy SSR lanes compile
//! an SFC to different production output, so `vize reduce --script` keeps
//! exactly the candidates the SSR corpus counts as divergences.
//!
//! Usage: `ssr_lane_divergence [--name NAME] FILE`. `NAME` is the bundler id
//! passed to the compiler (default: `FILE`), so a reduction can keep the
//! corpus path while candidates are written elsewhere. Exit 1 means the same
//! output or outside the comparable boundary (no template, legacy failure);
//! exit 2 is a usage or read error.

use std::process::ExitCode;

use vize_atelier_sfc::ssr_differential::compare_production;
use vize_atelier_ssr::differential::on_both_lanes;

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (name, file) = match args.as_slice() {
        [file] => (file, file),
        [flag, name, file] if flag == "--name" => (name, file),
        _ => {
            eprintln!("usage: ssr_lane_divergence [--name NAME] FILE");
            return ExitCode::from(2);
        }
    };
    let Ok(source) = std::fs::read_to_string(file) else {
        eprintln!("ssr_lane_divergence: cannot read {file}");
        return ExitCode::from(2);
    };
    let differs = compare_production(name, &source, |compile| on_both_lanes(compile))
        .and_then(|pair| pair.differs());
    if differs == Some(true) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
