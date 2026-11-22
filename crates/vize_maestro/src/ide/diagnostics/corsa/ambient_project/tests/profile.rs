//! Compare authored results and measure full cold and resident queries.
use super::super::Mutex;
use super::*;
use std::time::Instant;
use vize_canon::{BatchTypeChecker, BatchTypeCheckerTrait, CorsaBridge};

fn batch(p: &Project, host: &Path) -> Vec<Value> {
    let config = p.config();
    let mut checker = BatchTypeChecker::with_options_and_corsa_path(
        p.dir.path(),
        Default::default(),
        config.corsa_path.as_deref(),
    )
    .unwrap();
    checker.scan_project().unwrap();
    let root = vize_l0::path::physical_path(host);
    checker
        .check_project()
        .unwrap()
        .diagnostics
        .into_iter()
        .map(|d| {
            assert_eq!(
                d.file, root,
                "unexpected diagnostic on another fixture file"
            );
            json!([d.code, d.severity, d.line, d.column, d.message])
        })
        .collect()
}

fn starts(wire: &Value) -> Vec<Value> {
    wire.as_array()
        .unwrap()
        .iter()
        .map(|d| {
            json!([
                d["code"],
                d["severity"],
                d["range"]["start"]["line"],
                d["range"]["start"]["character"],
                d["message"]
            ])
        })
        .collect()
}

fn median(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

#[test]
#[ignore = "paired complete native responses with real TypeScript and Vue"]
fn ambient_project_cost() {
    let name = "src/App.vue";
    for declared in [false, true] {
        let p = Project::new();
        p.write(
            "tsconfig.json",
            &fs::read_to_string(pack().join("tsconfig.json.txt")).unwrap(),
        );
        let source = fs::read_to_string(pack().join("missing.App.vue.txt")).unwrap();
        p.write(name, &source);
        if declared {
            for (file, fixture) in [
                ("src/components.d.ts", "components.d.ts.txt"),
                ("src/components/ElBadge.vue", "ElBadge.vue.txt"),
            ] {
                p.write(file, &fs::read_to_string(pack().join(fixture)).unwrap());
            }
        }
        let host = p.open(name, &source, "vue");
        let expected = batch(&p, &p.path(name));
        assert_eq!(expected.len(), usize::from(declared));
        let mut cold_batch = Vec::new();
        let mut cold_editor = Vec::new();
        for trial in 0..9 {
            let run_batch = || {
                let start = Instant::now();
                let got = batch(&p, &p.path(name));
                let spent = start.elapsed();
                assert_eq!(got, expected);
                spent.as_nanos()
            };
            let run_editor = || {
                crate::runtime::block_on(async {
                    let start = Instant::now();
                    let session = p.start().await;
                    let got = wire(&session, &host).await;
                    let spent = start.elapsed();
                    assert_eq!(starts(&got), expected);
                    session.shutdown().await.unwrap();
                    spent.as_nanos()
                })
            };
            if trial % 2 == 0 {
                cold_batch.push(run_batch());
                cold_editor.push(run_editor());
            } else {
                cold_editor.push(run_editor());
                cold_batch.push(run_batch());
            }
        }
        let root = vize_l0::path::physical_path(p.dir.path());
        let (mut ordinary, mut resident) = (Vec::new(), Vec::new());
        crate::runtime::block_on(async {
            let session = p.start().await;
            let reference = wire(&session, &host).await;
            assert_eq!(starts(&reference), expected);
            // With no declaration both constructors select exactly the same
            // native program. Compare the full open, native read and mapping.
            let stock = if declared {
                None
            } else {
                let mut config = p.config();
                config.working_dir = Some(root.clone());
                let bridge = CorsaBridge::with_config(config);
                bridge.spawn().await.unwrap();
                let stock = AmbientComponentProject {
                    state: Arc::clone(&p.state),
                    root,
                    bridge,
                    transaction: Mutex::new(()),
                };
                assert_eq!(wire(&stock, &host).await, reference);
                Some(stock)
            };
            for trial in 0..61 {
                let run = async |session: &AmbientComponentProject| {
                    let start = Instant::now();
                    let got = wire(session, &host).await;
                    let spent = start.elapsed();
                    assert_eq!(got, reference);
                    spent.as_nanos()
                };
                if trial % 2 == 0
                    && let Some(stock) = &stock
                {
                    ordinary.push(run(stock).await);
                }
                resident.push(run(&session).await);
                if trial % 2 == 1
                    && let Some(stock) = &stock
                {
                    ordinary.push(run(stock).await);
                }
            }
            if let Some(stock) = stock {
                stock.shutdown().await.unwrap();
            }
            session.shutdown().await.unwrap();
        });
        let stock = (!ordinary.is_empty()).then(|| median(&mut ordinary));
        eprintln!(
            "ambient_cost declared={declared} cold_samples=9 batch_ns={} editor_ns={} warm_samples=61 ordinary_editor_ns={stock:?} resident_editor_ns={}",
            median(&mut cold_batch),
            median(&mut cold_editor),
            median(&mut resident)
        );
    }
}
