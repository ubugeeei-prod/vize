#![expect(
    clippy::disallowed_types,
    reason = "The checker SDK owns its profiling callback across request threads"
)]

use super::retained_document_corsa_path;
use crate::linter::corsa_session::{CorsaTypeAwareSession, TypeProbe};
use corsa::api::{ApiProfileEvent, ApiProfilePhase, ApiProfiler};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
struct Snapshots(AtomicUsize);

impl ApiProfiler for Snapshots {
    fn on_profile(&self, event: &ApiProfileEvent) {
        if event.method == "updateSnapshot" && event.phase == ApiProfilePhase::Transport {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl Snapshots {
    fn count(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}

#[test]
fn project_batches_refresh_once_and_preserve_every_document_probe() {
    let Some(corsa_path) = retained_document_corsa_path() else {
        return;
    };
    for disk_only in [false, true] {
        for newline in ["\n", "\r\n"] {
            let root = tempfile::Builder::new()
                .prefix("patina-batch-'😀-")
                .tempdir()
                .unwrap();
            let root = root.path().canonicalize().unwrap();
            std::fs::write(root.join("package.json"), "{}").unwrap();
            std::fs::write(root.join("Child.vue"), "<script setup lang=\"ts\">defineProps<{ label: string }>()</script><template><span /></template>").unwrap();
            let filenames: Vec<_> = ["A.vue", "B.ts", "C.tsx", "D.mts"]
                .map(|name| root.join(name).to_str().unwrap().to_owned())
                .into();
            let sources: Vec<_> = ["unknown", "null", "undefined", "never"].into_iter().enumerate().map(|(index, ty)| {
                let import = if index == 0 { "import Child from './Child.vue';\n" } else { "" };
                vize_l0::cstr!("{import}export {{}};\n// 😀 retained coordinates\ndeclare const query: {ty};\nquery;\n").replace('\n', newline)
            }).collect();
            let counter = Arc::new(Snapshots::default());
            let mut session = CorsaTypeAwareSession::new_with_configure(
                filenames.first().unwrap(),
                Some(&corsa_path),
                None,
                |config| config.with_profiler(counter.clone()),
            )
            .unwrap();
            assert_eq!(counter.count(), 1, "initial checker snapshot");
            if disk_only {
                session.supports_overlay_updates = false;
            }
            let inputs: Vec<_> = sources
                .iter()
                .zip(&filenames)
                .map(|(source, name)| (source.as_str(), name.as_str()))
                .collect();
            session.open_virtual_projects(&inputs).unwrap();
            assert_eq!(
                counter.count(),
                2,
                "one real updateSnapshot for four documents"
            );
            check_all(
                &mut session,
                &sources,
                &filenames,
                &["unknown", "null", "undefined", "never"],
            );
            assert_eq!(counter.count(), 2, "cached retargets must not refresh");
            assert!(
                session
                    .opened_documents
                    .get(filenames.first().unwrap().as_str())
                    .unwrap()
                    .rewritten_source
                    .is_some()
            );

            let mut edited = sources.clone();
            let first = edited.first_mut().unwrap();
            *first = first.replace("query: unknown", "query: never");
            let third = edited.get_mut(2).unwrap();
            *third = third.replace("query: undefined", "query: unknown");
            let changed: Vec<_> = edited
                .iter()
                .zip(&filenames)
                .map(|(source, name)| (source.as_str(), name.as_str()))
                .collect();
            session.open_virtual_projects(&changed).unwrap();
            assert_eq!(counter.count(), 3, "two edits share one snapshot refresh");
            check_all(
                &mut session,
                &edited,
                &filenames,
                &["never", "null", "unknown", "never"],
            );
            session.open_virtual_projects(&changed).unwrap();
            assert_eq!(counter.count(), 3, "unchanged project batch");
            let duplicate = [
                ("invalid source", filenames.first().unwrap().as_str()),
                ("other invalid source", filenames.first().unwrap().as_str()),
            ];
            assert_eq!(
                session.open_virtual_projects(&duplicate).unwrap_err(),
                vize_l0::cstr!(
                    "Duplicate authored document in patina snapshot batch: {}",
                    filenames.first().unwrap()
                )
            );
            assert_eq!(
                counter.count(),
                3,
                "duplicate identities are rejected before publishing"
            );
            check_all(
                &mut session,
                &edited,
                &filenames,
                &["never", "null", "unknown", "never"],
            );
            session.open_virtual_projects(&[]).unwrap();
            assert_eq!(counter.count(), 3, "empty batch keeps the active snapshot");
            assert_eq!(
                session.active_filename.as_deref(),
                Some(filenames.last().unwrap().as_str())
            );
            session.close();
        }
    }
}

fn check_all(
    session: &mut CorsaTypeAwareSession,
    sources: &[impl AsRef<str>],
    filenames: &[impl AsRef<str>],
    types: &[&str],
) {
    for ((source, filename), ty) in sources.iter().zip(filenames).zip(types) {
        let source = source.as_ref();
        let filename = filename.as_ref();
        session.open_virtual_project(source, filename).unwrap();
        let at = source.rfind("query;").unwrap() as u32;
        let probes = session
            .probe_types_at_offsets(source, &[(at, false, false)])
            .unwrap();
        assert_eq!(
            probes,
            vec![Some(TypeProbe {
                type_texts: vec![(*ty).into()],
                ..TypeProbe::default()
            })],
            "{filename}"
        );
    }
}
