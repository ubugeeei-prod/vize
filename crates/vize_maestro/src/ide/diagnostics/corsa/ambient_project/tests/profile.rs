//! Compare authored results and measure full cold and resident queries.
use super::super::super::collect_virtual::finished_from_phased_lsp;
use super::super::{CorsaDocument, assemble_corsa_diagnostics, borrowed};
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

fn ordinary(p: &Project) -> CorsaBridge {
    let mut config = p.config();
    config.working_dir = Some(vize_l0::path::physical_path(p.dir.path()));
    CorsaBridge::with_config(config)
}

async fn ordinary_wire(p: &Project, bridge: &CorsaBridge, host: &SourceSnapshot) -> Value {
    let path = vize_l0::path::physical_path(&host.uri().to_file_path().unwrap());
    let options = p.state.editor_virtual_ts_options().await;
    let overlays = p.state.corsa_overlays();
    let document = bridge
        .open_vue_virtual_document_with_borrowed_overlays_and_options(
            &path,
            host.source(),
            p.state.canonical_vue_document_options(),
            &borrowed(&overlays),
            &options,
        )
        .await
        .unwrap();
    let diagnostics = bridge
        .get_diagnostics_with_provenance(&document.request_uri)
        .await
        .unwrap();
    let projection = CorsaDocument {
        code: document.code,
        mapping: document.mapping,
        import_source_map: document.import_source_map,
    };
    let finished = diagnostics
        .into_iter()
        .filter_map(|row| finished_from_phased_lsp(&projection.code, row, 0))
        .collect();
    let response: Vec<_> = assemble_corsa_diagnostics(host.source(), &[projection], finished)
        .into_iter()
        .map(|row| row.diagnostic)
        .collect();
    serde_json::to_value(response).unwrap()
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
        let mut cold_stock = Vec::new();
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
                    let session = p.start();
                    let got = wire(&session, &host).await;
                    let spent = start.elapsed();
                    assert_eq!(starts(&got), expected);
                    session.shutdown().await.unwrap();
                    spent.as_nanos()
                })
            };
            let run_stock = || {
                crate::runtime::block_on(async {
                    let start = Instant::now();
                    let bridge = ordinary(&p);
                    bridge.spawn().await.unwrap();
                    let got = ordinary_wire(&p, &bridge, &host).await;
                    let spent = start.elapsed();
                    assert_eq!(starts(&got), expected);
                    bridge.shutdown().await.unwrap();
                    spent.as_nanos()
                })
            };
            if trial % 2 == 0 {
                if !declared {
                    cold_stock.push(run_stock());
                }
                cold_batch.push(run_batch());
                cold_editor.push(run_editor());
            } else {
                cold_editor.push(run_editor());
                cold_batch.push(run_batch());
                if !declared {
                    cold_stock.push(run_stock());
                }
            }
        }
        let (mut stock_times, mut resident) = (Vec::new(), Vec::new());
        crate::runtime::block_on(async {
            let session = p.start();
            let reference = wire(&session, &host).await;
            assert_eq!(starts(&reference), expected);
            // With no declaration both constructors select exactly the same
            // native program. Compare the full open, native read and mapping.
            let stock = if declared {
                None
            } else {
                let stock = ordinary(&p);
                stock.spawn().await.unwrap();
                assert_eq!(ordinary_wire(&p, &stock, &host).await, reference);
                Some(stock)
            };
            for trial in 0..61 {
                let run_stock = async |bridge: &CorsaBridge| {
                    let start = Instant::now();
                    let got = ordinary_wire(&p, bridge, &host).await;
                    let spent = start.elapsed();
                    assert_eq!(got, reference);
                    spent.as_nanos()
                };
                if trial % 2 == 0
                    && let Some(stock) = &stock
                {
                    stock_times.push(run_stock(stock).await);
                }
                let start = Instant::now();
                let got = wire(&session, &host).await;
                resident.push(start.elapsed().as_nanos());
                assert_eq!(got, reference);
                if trial % 2 == 1
                    && let Some(stock) = &stock
                {
                    stock_times.push(run_stock(stock).await);
                }
            }
            if let Some(stock) = stock {
                stock.shutdown().await.unwrap();
            }
            session.shutdown().await.unwrap();
        });
        let stock = (!stock_times.is_empty()).then(|| median(&mut stock_times));
        let stock_cold = (!cold_stock.is_empty()).then(|| median(&mut cold_stock));
        eprintln!(
            "ambient_cost declared={declared} cold_samples=9 batch_ns={} ordinary_cold_editor_ns={stock_cold:?} editor_ns={} warm_samples=61 ordinary_editor_ns={stock:?} resident_editor_ns={}",
            median(&mut cold_batch),
            median(&mut cold_editor),
            median(&mut resident)
        );
    }
}
