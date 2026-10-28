use criterion::{BatchSize, Criterion};
use std::{hint::black_box, path::PathBuf, time::Duration};
use vize_canon::batch::{CompositeSourceMap, VirtualFile};
use vize_carton::{FxHashMap, String};
#[path = "partition_weight/baseline.rs"]
mod baseline;
#[path = "partition_weight/candidate.rs"]
mod candidate;
type Bins = Vec<(usize, Vec<usize>)>;
type Kernel = fn(&[&VirtualFile], FxHashMap<usize, Vec<usize>>, usize) -> Option<Bins>;
fn inputs(
    groups: usize,
    members: usize,
    heavy: usize,
) -> (Vec<VirtualFile>, FxHashMap<usize, Vec<usize>>) {
    let mut files = Vec::new();
    let mut components = FxHashMap::default();
    for group in 0..groups {
        let mut indices = Vec::new();
        for _ in 0..if group == 0 { members + heavy } else { members } {
            let index = files.len();
            let path = PathBuf::from(format!("/partition-weight/{index}.vue"));
            indices.push(index);
            files.push(VirtualFile {
                content: String::from("x".repeat(32 + index % 11)),
                source_map: CompositeSourceMap::empty(),
                original_path: path.clone(),
                virtual_path: path,
            });
        }
        components.insert(group, indices);
    }
    (files, components)
}
fn main() {
    let pass = std::env::var("PARTITION_WEIGHT_ORDER").unwrap_or_else(|_| "baseline-first".into());
    assert!(pass == "baseline-first" || pass == "candidate-first");
    let mut criterion = Criterion::default()
        .sample_size(30)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2));
    if let Some(out) = std::env::var_os("PARTITION_WEIGHT_CRITERION_DIR") {
        criterion = criterion.output_directory(&PathBuf::from(out));
    }
    criterion = criterion.configure_from_args();
    for (label, groups, members, heavy) in [
        ("eight", 8, 1, 0),
        ("medium", 128, 8, 0),
        ("many", 2500, 1, 0),
        ("skewed", 128, 1, 10_000),
    ] {
        let (files, components) = inputs(groups, members, heavy);
        let refs: Vec<_> = files.iter().collect();
        assert_eq!(
            baseline::pack(&refs, components.clone(), 8),
            candidate::pack(&refs, components.clone(), 8)
        );
        eprintln!(
            "partition-weight-normal fixture={label} groups={groups} members={} source_bytes={} pass={pass} setup=excluded output_drop=excluded counters=none hardware=not_measured",
            files.len(),
            files.iter().map(|file| file.content.len()).sum::<usize>()
        );
        let mut group = criterion.benchmark_group(format!("partition_weight/{pass}/{label}"));
        let mut kernels: [(&str, Kernel); 2] =
            [("baseline", baseline::pack), ("candidate", candidate::pack)];
        if pass == "candidate-first" {
            kernels.reverse();
        }
        for (name, kernel) in kernels {
            group.bench_function(name, |b| {
                b.iter_batched(
                    || components.clone(),
                    |input| black_box(kernel(black_box(&refs), black_box(input), black_box(8))),
                    BatchSize::PerIteration,
                )
            });
        }
        group.finish();
    }
    criterion.final_summary();
}
