//! Independent report-only phase work; linked to each measured role's actual rlibs.
use std::{hint::black_box, time::Instant};
use vize_atelier_sfc::{CssCompileOptions, compile_css};
use vize_carton::pool;
use vize_croquis::sfc::__internal::extract_and_transform_v_bind_with_scope;

fn main() {
    let inputs = std::env::args().nth(1).expect("pinned phase inputs");
    let inputs: serde_json::Value =
        serde_json::from_slice(&std::fs::read(inputs).unwrap()).unwrap();
    let filename = inputs.get("filename").unwrap().as_str().unwrap();
    let options = CssCompileOptions {
        filename: Some(filename.into()),
        ..Default::default()
    };
    let mut vectors = Vec::new();
    for vector in inputs.get("vectors").unwrap().as_array().unwrap() {
        let source = vector.get("source").unwrap().as_str().unwrap();
        let batch = vector.get("batch").unwrap().as_u64().unwrap();
        let (processed, vars) = {
            let arena = pool::acquire();
            let (code, vars) = extract_and_transform_v_bind_with_scope(&arena, source, None);
            (code.to_owned(), vars)
        };
        let full = compile_css(source, &options);
        let backend = compile_css(&processed, &options);
        assert_eq!(full.code, backend.code);
        assert_eq!(full.errors, backend.errors);
        assert_eq!(full.warnings, backend.warnings);
        assert_eq!(full.css_vars, vars);
        assert_eq!(backend.css_vars.len(), 0);
        assert_eq!(full.map, None);
        assert_eq!(backend.map, None);
        let phases = ["scanner", "processed-input-compile", "full-compile"];
        let work = |phase: &str| match phase {
            "scanner" => {
                let bump = pool::acquire();
                let result =
                    extract_and_transform_v_bind_with_scope(&bump, black_box(source), None);
                black_box(result);
            }
            "processed-input-compile" => {
                black_box(compile_css(
                    black_box(processed.as_str()),
                    black_box(&options),
                ));
            }
            "full-compile" => {
                black_box(compile_css(black_box(source), black_box(&options)));
            }
            _ => unreachable!("closed phase"),
        };
        for _ in 0..100 {
            for phase in phases {
                work(phase);
            }
        }
        let mut samples = Vec::new();
        for sample in 0..20 {
            for offset in 0..phases.len() {
                let phase = phases.get((sample + offset) % phases.len()).unwrap();
                let start = Instant::now();
                for _ in 0..batch {
                    work(phase);
                }
                samples.push(serde_json::json!({
                    "phase": phase, "calls": batch, "elapsedNs": start.elapsed().as_nanos()
                }));
            }
        }
        vectors.push(serde_json::json!({
            "id":vector.get("id").unwrap(), "source":source,
            "options":{"filename":filename,"minify":false,"sourceMap":false},
            "logicalWork":{"authoredBytes":source.len(),"processedBytes":processed.len(),"variables":vars.len()},
            "outputs":{"scanner":{"code":processed,"variables":vars.iter().map(|v|v.as_str()).collect::<Vec<_>>()},"processedInputCompile":backend,"fullCompile":full},
            "samples":samples,
        }));
    }
    println!(
        "{}",
        serde_json::json!({
            "schema":"vize.css.source-map.source-phases", "version":1, "vectors":vectors,
            "note":"Processed-input compilation includes the existing public guard and no-binding prepass; these timings are not disjoint additive internal phases or instruction qualification."
        })
    );
}
