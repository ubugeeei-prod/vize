use std::path::{Path, PathBuf};

use vize_carton::hash::hash_str;
use vize_relief::TemplateSyntaxMode;

/// Aggregate counters for the native batch stats surface.
#[derive(Default)]
pub(super) struct BatchStats {
    pub(super) success: usize,
    pub(super) failed: usize,
    pub(super) input_bytes: usize,
    pub(super) output_bytes: usize,
}

impl BatchStats {
    pub(super) fn failed() -> Self {
        Self {
            failed: 1,
            ..Default::default()
        }
    }

    pub(super) fn add(mut self, other: Self) -> Self {
        self.success += other.success;
        self.failed += other.failed;
        self.input_bytes += other.input_bytes;
        self.output_bytes += other.output_bytes;
        self
    }
}

/// Fingerprint used to collapse repeated batch inputs before compiling.
/// Parent identity prevents grouping files whose relative imports differ.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct BatchCompileKey {
    pub(super) source_hash: u64,
    pub(super) source_len: usize,
    pub(super) parent_hash: u64,
    pub(super) parent_len: usize,
    pub(super) component_name_len: usize,
    pub(super) options: u16,
}

/// One physical compile job, possibly standing in for many logical files.
pub(super) struct BatchCompileJob {
    pub(super) path: PathBuf,
    pub(super) source: String,
    pub(super) repeats: usize,
    pub(super) input_bytes: usize,
}

impl BatchCompileJob {
    pub(super) fn single(path: PathBuf, source: String) -> Self {
        let input_bytes = source.len();
        Self {
            path,
            source,
            repeats: 1,
            input_bytes,
        }
    }
}

pub(super) fn batch_options_bits(
    ssr: bool,
    vapor: bool,
    is_ts: bool,
    template_syntax: TemplateSyntaxMode,
    standalone: bool,
    experimental_bits: u16,
) -> u16 {
    u16::from(ssr)
        | (u16::from(vapor) << 1)
        | (u16::from(is_ts) << 2)
        | (u16::from(template_syntax_bits(template_syntax)) << 3)
        | (u16::from(standalone) << 5)
        | experimental_bits
}

fn template_syntax_bits(template_syntax: TemplateSyntaxMode) -> u8 {
    match template_syntax {
        TemplateSyntaxMode::Standard => 0,
        TemplateSyntaxMode::Strict => 1,
        TemplateSyntaxMode::Quirks => 2,
        _ => 3,
    }
}

/// Reject grouping when the representative filename can affect self-resolution.
pub(super) fn should_cache_batch_compile(source: &str, component_name: &str) -> bool {
    component_name.is_empty()
        || (!source.contains(component_name)
            && !source.contains(component_name_to_kebab_case(component_name).as_str()))
}

fn component_name_to_kebab_case(component_name: &str) -> String {
    let mut out = String::with_capacity(component_name.len());
    for (index, ch) in component_name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index != 0 {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// Include the directory because script type imports resolve relative to it.
pub(super) fn parent_cache_parts(path: &Path) -> (u64, usize) {
    let Some(parent) = path.parent() else {
        return (hash_str(""), 0);
    };
    let parent = parent.to_string_lossy();
    (hash_str(parent.as_ref()), parent.len())
}
