//! Drive both implementations over SFC files: extract the template block,
//! lower it with file-absolute spans, run the production pass and the
//! naive evaluator, compare exactly, and keep the run's scope proof and
//! the per-component distribution.

use std::path::{Path, PathBuf};

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_s0::{Allocator, SourceRoot};
use vize_s1::parse;
use vize_s1_to_s2::lower_source_block;
use vize_s1_to_s2::pass::cfg;
use vize_s2::folio::S2Folio;

use super::spec::{Naive, Row, evaluate};

#[derive(Debug, Default)]
pub struct Report {
    pub files: u64,
    pub unreadable: u64,
    pub no_template: u64,
    /// Templates in a non-HTML dialect (`lang="pug"`), counted, not scored.
    pub foreign_lang: u64,
    pub templates: u64,
    pub rows: u64,
    pub unknown: u64,
    pub cyclomatic: Vec<u32>,
    pub cognitive: Vec<u32>,
    pub failures: Vec<String>,
}

impl Report {
    /// The scope proof: a run that scored no template proves nothing.
    pub fn verdict(&self, scope: &str) -> Result<(), String> {
        if self.templates == 0 {
            return Err(format!(
                "complexity {scope}: zero templates were scored — the run proves nothing \
                 ({} files); a degenerated suite must fail, not pass",
                self.files
            ));
        }
        match self.failures.first() {
            None => Ok(()),
            Some(first) => Err(format!(
                "complexity {scope}: {} of {} templates disagree; first:\n{first}",
                self.failures.len(),
                self.templates
            )),
        }
    }

    pub fn scope_line(&self, scope: &str) -> String {
        format!(
            "complexity {scope}: files={} templates={} rows={} unknown={} \
             no-template={} foreign-lang={} unreadable={}",
            self.files,
            self.templates,
            self.rows,
            self.unknown,
            self.no_template,
            self.foreign_lang,
            self.unreadable
        )
    }
}

/// The production facts in the oracle's row shape.
pub fn production(facts: &cfg::ComplexityFacts) -> Naive {
    let mut rows: Vec<Row> = facts
        .contributions
        .iter()
        .map(|row| Row {
            start: row.span.start,
            end: row.span.end,
            kind: row.kind.as_str(),
            op: row.op.map(|id| id.index()),
            nesting: row.nesting,
            cyclomatic: row.cyclomatic,
            cognitive: row.cognitive,
        })
        .collect();
    rows.sort();
    Naive {
        cyclomatic: facts.cyclomatic,
        cognitive: facts.cognitive,
        unknown: facts.unknown,
        max_nesting: facts.max_nesting,
        rows,
    }
}

/// The template block of an SFC as `(content, file offset)`, or why not.
fn template_of(source: &str) -> Result<(&str, u32), &'static str> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).map_err(|_| "no-template")?;
    let template = descriptor.template.as_ref().ok_or("no-template")?;
    if template.lang.as_deref().is_some_and(|lang| lang != "html") {
        return Err("foreign-lang");
    }
    let (start, end) = (template.loc.start, template.loc.end);
    let content = source.get(start..end).ok_or("no-template")?;
    Ok((content, u32::try_from(start).expect("small file")))
}

/// Score one SFC's template with both implementations.
pub fn run_source(source: &str, context: &str, report: &mut Report) {
    report.files += 1;
    let (content, start) = match template_of(source) {
        Ok(found) => found,
        Err("foreign-lang") => {
            report.foreign_lang += 1;
            return;
        }
        Err(_) => {
            report.no_template += 1;
            return;
        }
    };
    let (production, naive) = both(source, content, start);
    report.templates += 1;
    report.rows += production.rows.len() as u64;
    report.unknown += u64::from(production.unknown);
    report.cyclomatic.push(production.cyclomatic);
    report.cognitive.push(production.cognitive);
    if production != naive {
        report.failures.push(format!(
            "{context}\nproduction: {production:#?}\nnaive: {naive:#?}"
        ));
    }
}

/// Both implementations over one template block of `source`.
pub fn both(source: &str, content: &str, start: u32) -> (Naive, Naive) {
    let allocator = Allocator::new();
    let root = SourceRoot::new(source).expect("small source");
    let block = root
        .block(content, start)
        .expect("template is a source slice");
    let (tree, errors) = parse(&allocator, content);
    let lowered = lower_source_block(&allocator, &tree, &errors, block);
    let facts = cfg::run(&lowered);
    let folio = S2Folio::of(&lowered.root.ops);
    (production(&facts), evaluate(&folio))
}

/// Both implementations over a bare template string (offset 0).
pub fn both_template(template: &str) -> (Naive, Naive) {
    both(template, template, 0)
}

/// Every `.vue` file under `root`, sorted, `node_modules` pruned.
pub fn collect_vue_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    children.sort();
    for child in children {
        if child.is_dir() {
            if child.file_name().is_some_and(|name| name == "node_modules") {
                continue;
            }
            collect_vue_files(&child, out);
        } else if child.extension().is_some_and(|ext| ext == "vue") {
            out.push(child);
        }
    }
}

pub fn run_files(files: &[PathBuf]) -> Report {
    let mut report = Report::default();
    for file in files {
        match std::fs::read_to_string(file) {
            Ok(source) => run_source(&source, &file.to_string_lossy(), &mut report),
            Err(_) => report.unreadable += 1,
        }
    }
    report
}

/// Nearest-rank percentile over an unsorted sample.
pub fn percentile(sample: &[u32], q: u32) -> u32 {
    let mut sorted = sample.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    if n == 0 {
        return 0;
    }
    let rank = (usize::try_from(q).expect("q fits") * n)
        .div_ceil(100)
        .max(1);
    sorted[rank - 1]
}

/// The distribution table `complexity-metrics.md` records.
pub fn distribution_table(report: &Report) -> String {
    let mut table = String::from("| metric | n | p50 | p90 | p95 | p99 | max |\n");
    table.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n");
    for (name, sample) in [
        ("cyclomatic", &report.cyclomatic),
        ("cognitive", &report.cognitive),
    ] {
        table.push_str(&format!(
            "| {name} | {} | {} | {} | {} | {} | {} |\n",
            sample.len(),
            percentile(sample, 50),
            percentile(sample, 90),
            percentile(sample, 95),
            percentile(sample, 99),
            sample.iter().max().copied().unwrap_or(0)
        ));
    }
    table
}
