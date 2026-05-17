use super::{CrossFileAnalyzer, CrossFileOptions};
use std::path::Path;
use vize_croquis::AnalyzerOptions;

fn script_analysis(script: &str, used_components: &[&str]) -> vize_croquis::Croquis {
    let mut analyzer = vize_croquis::Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    for component in used_components {
        analyzer
            .croquis_mut()
            .used_components
            .insert(vize_carton::CompactString::new(*component));
    }
    analyzer.finish()
}

mod basic;
mod patterns;
mod playground;
mod provider_context;
mod provider_reactivity;
mod tree;
