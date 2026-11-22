use super::*;
use std::path::{Path, PathBuf};
use vize::plugin_template::register;
use vize_atelier_sfc::{
    SfcParseOptions,
    croquis::{SfcCroquisOptions, analyze_sfc_descriptor_with_occurrences},
    parse_sfc,
};
use vize_croquis::{
    Croquis, EffectGraphScript, EffectGraphSummary, build_effect_graph_from_sfc_scripts,
};
use vize_croquis_cf::{ComponentComplexity, CrossFileAnalyzer, CrossFileOptions, FileId};
use vize_doctor::{DoctorReport, application_analysis::report_from_application_graph};
use vize_patina::plugin_batch::error::HostError;

struct Input {
    path: PathBuf,
    source: String,
}

impl Input {
    fn new(path: &str, source: impl Into<String>) -> Self {
        Self {
            path: Path::new("/vize-project").join(path),
            source: source.into(),
        }
    }

    // The same semantic and effect producers that populate the CLI Doctor graph.
    fn analysis(&self) -> (Croquis, EffectGraphSummary) {
        let descriptor = parse_sfc(&self.source, SfcParseOptions::default()).unwrap();
        let arena = vize_l0::Allocator::new();
        let root = descriptor
            .template
            .as_ref()
            .map(|t| vize_armature::parse(&arena, &t.content).0);
        let analysis = analyze_sfc_descriptor_with_occurrences(
            &descriptor,
            root.as_ref(),
            SfcCroquisOptions::full(),
        )
        .0;
        let effects = build_effect_graph_from_sfc_scripts(
            descriptor
                .script
                .as_ref()
                .map(|s| EffectGraphScript::new(&s.content, s.lang.as_deref())),
            descriptor
                .script_setup
                .as_ref()
                .map(|s| EffectGraphScript::new(&s.content, s.lang.as_deref())),
        )
        .summary();
        (analysis.croquis, effects)
    }
}

fn graph() -> CrossFileAnalyzer {
    CrossFileAnalyzer::with_project_root(CrossFileOptions::minimal(), "/vize-project")
}

type Packet = Result<PluginTemplateBatch, HostError>;

fn add(
    graph: &mut CrossFileAnalyzer,
    input: &Input,
    request: &PluginSpec<'_>,
    joined: bool,
) -> (FileId, Packet) {
    let (analysis, effects) = input.analysis();
    if joined {
        let (id, packet) = register(
            graph,
            &input.path,
            &input.source,
            analysis,
            effects,
            request,
        );
        (id, packet.unwrap())
    } else {
        let id = graph.add_file_with_analysis_and_effect_summary(
            &input.path,
            &input.source,
            analysis,
            effects,
        );
        (
            id,
            original_plugin(&input.source, &input.path.to_string_lossy(), request),
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Report {
    components: Vec<ComponentComplexity>,
    doctor: DoctorReport,
}

fn report(graph: &mut CrossFileAnalyzer) -> Report {
    graph.rebuild_import_edges();
    graph.rebuild_component_edges();
    let result = graph.analyze();
    let doctor = report_from_application_graph(".", graph, &result).unwrap();
    Report {
        components: result.template_complexity,
        doctor,
    }
}

fn project(
    inputs: &[Input],
    request: &PluginSpec<'_>,
    joined: bool,
) -> (Vec<(FileId, Packet)>, Report) {
    let mut graph = graph();
    let packets = inputs
        .iter()
        .map(|input| add(&mut graph, input, request, joined))
        .collect();
    (packets, report(&mut graph))
}

fn equal(inputs: &[Input], request: &PluginSpec<'_>) -> Report {
    let (old, old_report) = project(inputs, request, false);
    let (new, new_report) = project(inputs, request, true);
    assert_eq!(new_report, old_report);
    assert_eq!(
        serde_json::to_vec(&new_report.doctor).unwrap(),
        serde_json::to_vec(&old_report.doctor).unwrap()
    );
    assert_eq!(new.len(), inputs.len());
    for ((id, plugin), (expected_id, original)) in new.into_iter().zip(old) {
        assert_eq!(id, expected_id);
        compare(
            &PluginTemplateProducts {
                plugin,
                complexity: None,
            },
            &PluginTemplateProducts {
                plugin: original,
                complexity: None,
            },
            "graph plugin",
        );
    }
    new_report
}

const CHILD: &str = r#"<template><div v-if="a"><div v-if="b"><div v-if="c"><p v-for="x in xs">{{ x ? 1 : 2 }}</p></div></div></div></template>"#;

fn board(count: usize) -> Vec<Input> {
    let mut imports = String::new();
    let mut tags = String::new();
    let mut files = Vec::new();
    for index in 1..=count {
        imports.push_str(&format!("import Card{index} from './Card{index}.vue';\n"));
        tags.push_str(&format!("<Card{index} />"));
        files.push(Input::new(&format!("Card{index}.vue"), CHILD));
    }
    files.insert(
        0,
        Input::new(
            "Board.vue",
            format!(
                "<script setup>\n{imports}</script>\r\n<template><main>{tags}</main></template>"
            ),
        ),
    );
    files
}

fn examples() -> Vec<(&'static str, Vec<Input>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    ["cli", "html-conformance", "vite-musea"]
        .into_iter()
        .map(|name| {
            let mut files = Vec::new();
            let source = root.join(name).join("src");
            for directory in [source.clone(), source.join("components")] {
                if !directory.exists() {
                    continue;
                }
                for file in std::fs::read_dir(directory).unwrap() {
                    let path = file.unwrap().path();
                    if path.extension().is_some_and(|ext| ext == "vue") {
                        let relative = path.strip_prefix(&source).unwrap().to_str().unwrap();
                        files.push(Input::new(
                            relative,
                            std::fs::read_to_string(&path).unwrap(),
                        ));
                    }
                }
            }
            files.sort_by(|a, b| a.path.cmp(&b.path));
            (name, files)
        })
        .collect()
}

#[test]
fn full_reports_packets_and_render_trees_match_for_original_projects() {
    let demands = ["templateScopes".into()];
    let request = spec(&demands, None);
    let board = board(12);
    let actual = equal(&board, &request);
    let root = actual
        .components
        .iter()
        .find(|row| row.file_name.ends_with("Board.vue"))
        .unwrap();
    assert_eq!(
        (
            root.rendered.cyclomatic,
            root.rendered.cognitive,
            root.rendered_components
        ),
        (73, 180, 12)
    );
    assert_eq!(actual.doctor.findings().len(), 1);
    assert_eq!(
        actual.doctor.findings()[0]
            .provenance
            .invalidation_inputs
            .len(),
        13
    );
    for (_, files) in examples() {
        equal(&files, &request);
    }
    let invalid_demands = ["unknown".into()];
    equal(&board, &spec(&invalid_demands, None));
}

#[test]
fn updates_remove_obsolete_records_and_clear_resets_file_ids() {
    let mut actual = graph();
    let mut original = graph();
    let request = spec(&[], None);
    let root = Input::new(
        "App.vue",
        "<script setup>import Child from './Child.vue'</script><template><Child/></template>",
    );
    let nested =
        "<script setup>import App from './App.vue'</script><template><App v-if='雪'/></template>";
    for source in [
        nested,
        "<script setup>const value=1</script>",
        "<template lang='pug'>p 外</template>",
        CHILD,
        nested,
    ] {
        for (graph, joined) in [(&mut actual, true), (&mut original, false)] {
            let _ = add(graph, &root, &request, joined);
            let input = Input::new("Child.vue", source);
            let (id, _) = add(graph, &input, &request, joined);
            assert_eq!(
                graph.template_complexity_of(id),
                TemplateComplexity::from_sfc(source).as_ref()
            );
        }
        assert_eq!(report(&mut actual), report(&mut original));
    }
    actual.clear();
    original.clear();
    let (id, packet) = register(
        &mut actual,
        Path::new("/vize-project/entry.ts"),
        "export const n=1",
        Croquis::new(),
        EffectGraphSummary::default(),
        &request,
    );
    let old = original.add_file_with_analysis_and_effect_summary(
        "/vize-project/entry.ts",
        "export const n=1",
        Croquis::new(),
        EffectGraphSummary::default(),
    );
    assert_eq!(id, old);
    assert!(packet.is_none());
    assert_eq!(report(&mut actual), report(&mut original));
}

#[test]
fn interrupted_template_pass_preserves_the_previous_revision_and_next_file_id() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let mut actual = graph();
    let mut original = graph();
    let request = spec(&[], None);
    let root = Input::new("App.vue", CHILD);
    let (id, _) = add(&mut actual, &root, &request, true);
    let _ = add(&mut original, &root, &request, false);
    for name in ["App.vue", "New.VUE"] {
        let input = Input::new(name, "<template><other/></template>");
        let (analysis, effects) = input.analysis();
        let failure = catch_unwind(AssertUnwindSafe(|| {
            actual.add_file_with_template_pass(
                &input.path,
                &input.source,
                analysis,
                effects,
                |_| -> (Option<TemplateComplexity>, ()) { panic!("template consumer interrupted") },
            );
        }));
        assert!(failure.is_err());
        assert_eq!(actual.registry().len(), 1);
        assert_eq!(actual.graph().len(), 1);
        assert_eq!(
            actual.template_complexity_of(id),
            original.template_complexity_of(id)
        );
        assert_eq!(report(&mut actual), report(&mut original));
    }
    let next = Input::new("New.VUE", "<template><main/></template>");
    assert_eq!(
        add(&mut actual, &next, &request, true).0,
        add(&mut original, &next, &request, false).0
    );
    assert_eq!(report(&mut actual), report(&mut original));
}

mod profile;
