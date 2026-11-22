//! Whole-file source and graph qualification over an explicit NUL-separated
//! corpus. Every queried result is checked against the original lowered rows.
#![cfg(feature = "graph-queries")]
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l3::query::{Direction, EdgeKinds, OpGraph, ReachQuery, check_reachability};

#[test]
#[ignore = "supply the tracked or real-project corpus via VIZE_IR_GRAPH_INPUTS"]
fn original_graphs_and_lossless_recovery() {
    let manifest = std::env::var_os("VIZE_IR_GRAPH_INPUTS").unwrap();
    let list = std::fs::read(manifest).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut total = 0;
    let mut invalid_utf8 = 0;
    let mut admitted = 0;
    let mut refused = 0;
    let mut graphs = 0;
    let mut queries = 0;
    let mut reached = 0;
    let mut bytes = 0;
    let options = DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    };
    for file in list
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        let path = std::str::from_utf8(file).unwrap();
        let data = std::fs::read(root.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"));
        total += 1;
        let Ok(source) = std::str::from_utf8(&data) else {
            invalid_utf8 += 1;
            continue;
        };
        bytes += data.len();
        let arena = Allocator::default();
        let original = lower_sfc_native(&arena, source, options);
        let l1 = original.l1ir();
        let mut cursor = 0usize;
        for seg in l1.container().segments() {
            let span = seg.span();
            assert_eq!(span.start as usize, cursor, "{path}");
            let end = span.end as usize;
            assert_eq!(
                l1.source().get(cursor..end).unwrap().as_bytes(),
                data.get(cursor..end).unwrap(),
                "{path}"
            );
            cursor = end;
        }
        assert_eq!(cursor, data.len(), "{path}");
        let Some(native) = original.admitted() else {
            refused += 1;
            continue;
        };
        admitted += 1;
        let lowered = vize_l2_to_l3::lower_ir(&arena, native.l2ir());
        let ir = lowered.l3ir();
        let index = OpGraph::new(ir).unwrap();
        if ir.ops.is_empty() {
            continue;
        }
        graphs += 1;
        for op in &ir.ops {
            for direction in [Direction::Forward, Direction::Reverse] {
                let q = ReachQuery {
                    start: op.id,
                    direction,
                    kinds: EdgeKinds::ALL,
                };
                let steps = index
                    .reach_with(q, |op| {
                        let span = op.span;
                        assert!(
                            source.get(span.start as usize..span.end as usize).is_some(),
                            "{path} {span:?}"
                        );
                    })
                    .unwrap();
                check_reachability(ir, q, &steps).unwrap_or_else(|e| panic!("{path}: {e:?}"));
                reached += steps.len();
                queries += 1;
            }
        }
    }
    assert!(graphs > 0 && refused > 0 && queries > 0);
    assert_eq!(total, invalid_utf8 + admitted + refused);
    println!(
        "graph corpus files={total} bytes={bytes} invalid_utf8={invalid_utf8} admitted={admitted} rejected={refused} nonempty={graphs} queries={queries} witnessed_ops={reached}"
    );
}
