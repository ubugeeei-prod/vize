//! Read-only source observations before dependency-finalization optimization.
mod fixtures;
mod snapshot;

use serde_json::json;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use vize_armature::Parser;
use vize_atelier_sfc::{
    croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_context},
    parse_sfc,
};
use vize_croquis::{EffectGraphScript, build_effect_graph_from_sfc_scripts};
use vize_croquis_cf::{CrossFileAnalyzer, CrossFileOptions, FileId};
use vize_l0::{Allocator, cstr};

fn register(
    analyzer: &mut CrossFileAnalyzer,
    root: &Path,
    name: &str,
    source: &str,
    raw_script: bool,
) -> FileId {
    let path = root.join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, source).unwrap();
    // Both actual providers observe identical physical file metadata as well
    // as identical bytes. This is input setup, not output normalization.
    let modified =
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_700_000_000);
    fs::File::open(&path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
    if path.extension().is_none_or(|extension| extension != "vue") {
        return analyzer.add_file(path, source);
    }
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let allocator = Allocator::default();
    let template = descriptor
        .template
        .as_ref()
        .map(|block| Parser::new(&allocator, block.content.as_ref()).parse().0);
    let captured = analyze_sfc_descriptor_with_context(
        &descriptor,
        template.as_ref(),
        SfcCroquisOptions::full(),
    );
    let script = descriptor
        .script
        .as_ref()
        .map(|block| EffectGraphScript::new(block.content.as_ref(), block.lang.as_deref()));
    let setup = descriptor
        .script_setup
        .as_ref()
        .map(|block| EffectGraphScript::new(block.content.as_ref(), block.lang.as_deref()));
    let effects = build_effect_graph_from_sfc_scripts(script, setup).summary();
    let script_content = captured.script_content.unwrap_or_default();
    let file_id = analyzer.add_file_with_analysis_and_effect_summary(
        &path,
        if raw_script { &script_content } else { source },
        captured.croquis,
        effects,
    );
    if raw_script {
        analyzer.record_template_complexity(file_id, source);
    }
    file_id
}

fn main() {
    let output = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/vize-tests/cross-file-finalize-facts"));
    fs::create_dir_all(&output).unwrap();
    let output = fs::canonicalize(output).unwrap();
    let mut observations = Vec::new();
    for case in fixtures::cases() {
        for (ordinal, order) in fixtures::orders(case.files.len()).into_iter().enumerate() {
            for raw_script in [false, true] {
                let root =
                    output.join(cstr!("fixtures/{}-{ordinal}-{raw_script}", case.name).as_str());
                fs::create_dir_all(&root).unwrap();
                let mut analyzer =
                    CrossFileAnalyzer::with_project_root(CrossFileOptions::all(), &root);
                for index in &order {
                    let (name, source) = case.files.get(*index).unwrap();
                    register(&mut analyzer, &root, name, source, raw_script);
                }
                analyzer.rebuild_import_edges();
                let first = snapshot::snapshot(&mut analyzer);
                analyzer.rebuild_component_edges();
                let second = snapshot::snapshot(&mut analyzer);
                analyzer.rebuild_import_edges();
                let third = snapshot::snapshot(&mut analyzer);
                observations.push(json!({"case":case.name,"order":order,"rawScriptRegistry":raw_script,"inputs":case.files,"first":first,"second":second,"third":third,"oneVsTwo":snapshot::comparison(&first,&second),"twoVsThree":snapshot::comparison(&second,&third)}));
            }
        }
    }
    let root = output.join("retained-edit");
    fs::create_dir_all(&root).unwrap();
    let mut analyzer = CrossFileAnalyzer::with_project_root(CrossFileOptions::all(), &root);
    let first = "<script setup>import Last from './Last.vue'</script><template><Last /></template>";
    let last =
        "<script setup>import First from './First.vue'</script><template><First /></template>";
    let first_id = register(&mut analyzer, &root, "First.vue", first, false);
    let last_id = register(&mut analyzer, &root, "Last.vue", last, false);
    analyzer.rebuild_component_edges();
    let original = snapshot::snapshot(&mut analyzer);
    register(
        &mut analyzer,
        &root,
        "First.vue",
        &fixtures::edited_source(),
        false,
    );
    analyzer.rebuild_component_edges();
    let edited = snapshot::snapshot(&mut analyzer);
    let stale_import =
        snapshot::stale_import(&analyzer, &root.join("First.vue"), &root.join("Last.vue"));
    analyzer.clear();
    let orphaned_template_facts = [first_id, last_id]
        .into_iter()
        .filter(|id| analyzer.template_complexity_of(*id).is_some())
        .map(|id| id.as_u32())
        .collect::<Vec<_>>();
    let cleared = snapshot::snapshot(&mut analyzer);
    register(
        &mut analyzer,
        &root,
        "First.vue",
        &fixtures::edited_source(),
        false,
    );
    analyzer.rebuild_component_edges();
    let repaired = snapshot::snapshot(&mut analyzer);
    let relation_failures = observations
        .iter()
        .filter(|row| {
            row.get("oneVsTwo")
                .and_then(|comparison| comparison.get("allForwardRelations"))
                != Some(&json!(true))
        })
        .count();
    let raw_changes = observations
        .iter()
        .filter(|row| {
            row.get("oneVsTwo")
                .and_then(|comparison| comparison.get("rawGraph"))
                != Some(&json!(true))
        })
        .count();
    let report = json!({"schema":"vize.cross-file.finalization-facts.observation","qualified":false,"note":"Read-only property observations, not product qualification. Raw graph ordering and complete original analysis are retained. Only documented analysis duration is split into a separate observation for semantic comparison.","observations":observations,"retainedOwner":{"original":original,"edited":edited,"staleImportRetained":stale_import,"orphanTemplateIdsAfterClear":orphaned_template_facts,"cleared":cleared,"repaired":repaired},"summary":{"casesObserved":observations.len(),"relationFailures":relation_failures,"rawGraphChanges":raw_changes}});
    fs::write(
        output.join("complete-observations.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    println!(
        "{}",
        json!({"casesObserved":observations.len(),"relationFailures":relation_failures,"rawGraphChanges":raw_changes,"staleImportRetained":stale_import})
    );
}
