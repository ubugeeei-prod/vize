//! One artifact through production and both specs.

use std::fs;
use std::path::{Path, PathBuf};

use vize_armature::Parser;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_carton::{Allocator, CompactString, cstr};
use vize_croquis::facts::spec::agreement::Agreement;
use vize_croquis::facts::spec::{
    bindings, bindings_extract, provide_inject, reactivity, trace, undefined_refs,
};
use vize_croquis::facts::{
    Bindings, CroquisFacts, Demand, FactConsumer, FactGroup, FactTable, UndefinedRefs,
};

/// The TS-34 harness reads both groups through a declared demand.
struct FactSpecHarness;
impl FactConsumer for FactSpecHarness {
    const NAME: &'static str = "ts-34/fact-spec";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID).with(UndefinedRefs::ID);
}

/// One run's tally per group.
#[derive(Default)]
pub struct Planes {
    pub bindings: Agreement,
    pub undefined: Agreement,
    pub reactivity: Agreement,
    pub provide: Agreement,
    pub race: Agreement,
}

impl Planes {
    /// `((artifacts, compared, facts) per group)` — the pinned census.
    pub fn census(&self) -> ((u64, u64, u64), (u64, u64, u64)) {
        let row = |run: &Agreement| (run.artifacts, run.compared, run.facts);
        (row(&self.bindings), row(&self.undefined))
    }

    pub fn scope_lines(&self, label: &str) -> CompactString {
        cstr!(
            "{}\n{}\n{}\n{}\n{}",
            self.bindings.scope_line("bindings", label),
            self.undefined.scope_line("undefined-refs", label),
            self.reactivity.scope_line("reactivity", label),
            self.provide.scope_line("provide-inject", label),
            self.race.scope_line("race-conditions", label)
        )
    }

    pub fn assert_verdicts(&self, label: &str) {
        self.bindings
            .verdict("bindings", label)
            .unwrap_or_else(|message| panic!("{message}"));
        if cfg!(debug_assertions) {
            self.undefined
                .verdict("undefined-refs", label)
                .unwrap_or_else(|message| panic!("{message}"));
        }
    }
}

pub fn collect_vue_files(root: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(root)? {
        entries.push(entry?.path());
    }
    entries.sort();
    for path in entries {
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == "node_modules" || name == ".git")
            {
                continue;
            }
            collect_vue_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "vue") {
            out.push(path);
        }
    }
    Ok(())
}

pub fn run_source(name: &str, source: &str, planes: &mut Planes) {
    let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
        planes.bindings.skip("sfc-parse-error");
        planes.undefined.skip("sfc-parse-error");
        planes.reactivity.skip("sfc-parse-error");
        planes.provide.skip("sfc-parse-error");
        planes.race.skip("sfc-parse-error");
        return;
    };
    let allocator = Allocator::new();
    let template_ast = descriptor.template.as_ref().map(|block| {
        let (root, _errors) = Parser::new(&allocator, block.content.as_ref()).parse();
        root
    });
    let (croquis, checked) = trace::record(|| {
        analyze_sfc_descriptor(
            &descriptor,
            template_ast.as_ref(),
            SfcCroquisOptions::full(),
        )
    });
    let mut facts = CroquisFacts::new(&croquis);
    let view = facts.prepare::<FactSpecHarness>();
    let production_bindings = view.get::<Bindings>().expect("declared");
    let production_undefined = view.get::<UndefinedRefs>().expect("declared");

    // -- Bindings: <script setup>-only artifacts --------------------------
    let setup = descriptor.script_setup.as_ref();
    if descriptor.script.is_some() {
        planes.bindings.skip("plain-script");
    } else if setup.is_some_and(|block| matches!(block.lang.as_deref(), Some("tsx" | "jsx"))) {
        planes.bindings.skip("jsx-script");
    } else {
        let decls = match setup {
            Some(block) => bindings_extract::extract(&block.content, false),
            None => Ok(Vec::new()),
        };
        match decls {
            Err(reason) => planes.bindings.skip(reason),
            Ok(decls) => {
                let macro_props: Vec<CompactString> = croquis
                    .macros
                    .props()
                    .iter()
                    .map(|prop| prop.name.clone())
                    .collect();
                let spec = bindings::evaluate(&decls, &macro_props, setup.is_some());
                let divergence = (spec != *production_bindings)
                    .then(|| describe_bindings(name, production_bindings, &spec));
                planes
                    .bindings
                    .compare(production_bindings.len(), divergence);
            }
        }
    }

    // -- UndefinedRefs: every artifact, debug builds -----------------------
    match checked {
        None => planes.undefined.skip("release-build"),
        Some(checked) => {
            let spec = undefined_refs::evaluate(&checked, &croquis.scopes, production_bindings);
            let production: Vec<_> = production_undefined
                .iter()
                .map(|(_, fact)| fact.clone())
                .collect();
            let (production, spec) = (
                undefined_refs::canonical(production),
                undefined_refs::canonical(spec),
            );
            let divergence = (production != spec)
                .then(|| cstr!("{name}: production {production:?} != spec {spec:?}"));
            planes.undefined.compare(production.len(), divergence);
        }
    }

    reactivity::compare_croquis(name, &croquis, &mut planes.reactivity);
    provide_inject::compare_provide(name, &croquis, &mut planes.provide);
    provide_inject::compare_race(name, &croquis, &mut planes.race);
}

fn describe_bindings(
    name: &str,
    production: &FactTable<Bindings>,
    spec: &FactTable<Bindings>,
) -> CompactString {
    let production: Vec<_> = production.iter().collect();
    let spec: Vec<_> = spec.iter().collect();
    let only_production: Vec<_> = production
        .iter()
        .filter(|row| !spec.contains(row))
        .collect();
    let only_spec: Vec<_> = spec
        .iter()
        .filter(|row| !production.contains(row))
        .collect();
    cstr!("{name}: production-only {only_production:?} spec-only {only_spec:?}")
}
