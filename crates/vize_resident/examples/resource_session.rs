//! Synthetic resident database session for P5-4b. Alpha pages are fixture
//! inputs, not a claim that a production Croquis exporter has been wired.
//! The Linux runner samples this process with the shared TS-44 sampler.

use std::{io::Write as _, process::ExitCode, time::Duration};
use vize_davinci::summary::{AlphaEntry, AlphaPages, Facet, Fingerprint, SfcSummary, Signature};
use vize_l0::{String, cstr};
use vize_resident::{
    DeclarationName, ResidentDatabase, SourceFile, StageConfig, SummaryInput,
    compute_file_artifacts, declaration_fingerprint, sfc_summary,
};

#[salsa::tracked(returns(copy))]
fn dependent_use<'db>(
    db: &'db dyn salsa::Database,
    dependent: SourceFile,
    provider: SummaryInput,
    facet: Facet,
    name: DeclarationName<'db>,
) -> Option<Fingerprint> {
    let _ = dependent.path(db);
    declaration_fingerprint(db, provider, facet, name)
}

struct File {
    source: SourceFile,
    summary: SummaryInput,
    index: usize,
    number_prop: bool,
}

fn pages(index: usize, number_prop: bool) -> AlphaPages {
    let entry = |name, contract| AlphaEntry {
        name,
        contract: String::from(contract),
    };
    AlphaPages {
        signature: Signature {
            name: cstr!("Component{index}"),
            params: String::default(),
        },
        props: vec![entry(
            cstr!("label{index}"),
            if number_prop { "number" } else { "string" },
        )],
        emits: vec![entry(cstr!("select{index}"), "MouseEvent")],
        slots: vec![entry(String::from("default"), "{ value: string }")],
        reactivity: vec![entry(cstr!("value{index}"), "ref")],
        components: vec![entry(cstr!("Component{}", index + 1), "resolved")],
    }
}

fn text(index: usize, revision: usize) -> String {
    cstr!(
        "<script setup lang=\"ts\">\nconst value{index} = {revision};\n</script>\n<template><section class=\"panel\"><h2>Component {index}</h2><button @click=\"select{index}\">{{{{ value{index} }}}}</button><slot :value=\"value{index}\"/></section></template>\n<style scoped>.panel {{ display: flex; color: red; }}</style>"
    )
}

fn verify(db: &ResidentDatabase, file: &File) -> Result<(), String> {
    let source = file.source.text(db);
    if db.file_artifacts(file.source) != compute_file_artifacts(source, StageConfig::default()) {
        return Err(cstr!(
            "incremental stage artifact differs from clean for file {}",
            file.index
        ));
    }
    let clean = SfcSummary::from_alpha(pages(file.index, file.number_prop))
        .map_err(|error| cstr!("invalid fixture alpha: {error:?}"))?;
    if sfc_summary(db, file.summary)
        .as_ref()
        .map_err(|error| cstr!("{error:?}"))?
        != &clean
    {
        return Err(cstr!(
            "incremental summary differs from clean for file {}",
            file.index
        ));
    }
    for (facet, name) in [
        (Facet::Prop, cstr!("label{}", file.index)),
        (Facet::Emit, cstr!("select{}", file.index)),
    ] {
        let name = db.declaration_name(&name);
        if declaration_fingerprint(db, file.summary, facet, name)
            != clean.fingerprint(facet, name.name(db))
        {
            return Err(cstr!(
                "incremental declaration differs from clean for file {}",
                file.index
            ));
        }
    }
    Ok(())
}

fn read_users(
    db: &ResidentDatabase,
    file: &File,
    dependent: SourceFile,
) -> [Option<Fingerprint>; 2] {
    [
        (Facet::Prop, cstr!("label{}", file.index)),
        (Facet::Emit, cstr!("select{}", file.index)),
    ]
    .map(|(facet, name)| {
        dependent_use(
            db,
            dependent,
            file.summary,
            facet,
            db.declaration_name(&name),
        )
    })
}

fn argument(args: &[String], name: &str, default: usize) -> Result<usize, String> {
    match args.iter().position(|arg| arg == name) {
        Some(at) => args
            .get(at + 1)
            .and_then(|value| value.parse().ok())
            .filter(|value| *value > 0)
            .ok_or_else(|| cstr!("{name} needs a positive integer")),
        None => Ok(default),
    }
}

fn run() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).map(String::from).collect();
    let count = argument(&args, "--files", 10_000)?;
    let edits = argument(&args, "--edits", 32)?;
    let hold = argument(&args, "--hold-seconds", 10)?;
    let mut db = ResidentDatabase::default();
    let mut files = Vec::with_capacity(count);
    for index in 0..count {
        let source = db.open(&cstr!("/synthetic/Component{index}.vue"), &text(index, 0));
        let summary = db.publish_alpha(source, pages(index, false));
        files.push(File {
            source,
            summary,
            index,
            number_prop: false,
        });
    }
    for file in &files {
        verify(&db, file)?;
    }
    let initial = db.take_accounting();
    if initial.get("sfc_summary").map(|row| row.executed) != Some(count as u32) {
        return Err(String::from(
            "the warm session did not execute every summary",
        ));
    }
    for revision in 1..=edits {
        // Keep the edited provider within the hot tail so this tests backdating
        // rather than an intentionally evicted memo. All cold files remain live.
        let index = count - 1 - (revision % count.min(32));
        let dependent = files
            .get((index + 1) % count)
            .ok_or_else(|| String::from("missing dependent fixture"))?
            .source;
        let file = files
            .get_mut(index)
            .ok_or_else(|| String::from("missing provider fixture"))?;
        let before = read_users(&db, file, dependent);
        let _ = db.take_accounting();
        db.edit_with_alpha(
            file.source,
            file.summary,
            &text(index, revision),
            pages(index, file.number_prop),
        );
        if read_users(&db, file, dependent) != before {
            return Err(String::from("a body edit changed an interface fingerprint"));
        }
        let counts = db.take_accounting();
        if counts
            .get("dependent_use")
            .is_none_or(|row| row.executed != 0 || row.reused != 2)
        {
            return Err(cstr!("a body edit executed a dependent query: {counts:?}"));
        }
        verify(&db, file)?;
        let _ = db.take_accounting();
        let before = read_users(&db, file, dependent);
        let _ = db.take_accounting();
        file.number_prop = !file.number_prop;
        db.revise_alpha(file.summary, pages(index, file.number_prop));
        let after = read_users(&db, file, dependent);
        let counts = db.take_accounting();
        if before[0] == after[0]
            || before[1] != after[1]
            || counts
                .get("dependent_use")
                .is_none_or(|row| row.executed != 1 || row.reused != 1)
        {
            return Err(cstr!(
                "a signature edit invalidated the wrong consumers: {counts:?}"
            ));
        }
        verify(&db, file)?;
    }
    db.configure_tsconfig("{\"strict\":true}");
    for file in &files {
        if sfc_summary(&db, file.summary) != &Err(vize_resident::ResidentSummaryError::StaleAlpha) {
            return Err(String::from(
                "changed tsconfig served an unrefreshed alpha summary",
            ));
        }
    }
    // Refresh all fixture exports after the high-durability config change,
    // then revisit every cold memo and reclaimed declaration name.
    for file in &files {
        db.revise_alpha(file.summary, pages(file.index, file.number_prop));
    }
    for file in &files {
        verify(&db, file)?;
        let _ = db.take_accounting();
    }
    println!(
        "{{\"files\":{count},\"edits\":{edits},\"stage_equivalence\":true,\"summary_equivalence\":true,\"body_edit_dependent_executions\":0,\"signature_edit_affected_consumers\":1,\"stale_config_rejected\":true}}"
    );
    std::io::stdout()
        .flush()
        .map_err(|error| cstr!("{error}"))?;
    // The external sampler also records current RSS while the populated
    // database is idle, rather than after its storage has been dropped.
    std::thread::sleep(Duration::from_secs(hold as u64));
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("resident resource session: {error}");
            ExitCode::FAILURE
        }
    }
}
