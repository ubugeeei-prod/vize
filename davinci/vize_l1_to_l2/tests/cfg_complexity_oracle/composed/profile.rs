use std::{hint::black_box, time::Instant};
use vize_l0::{Allocator, SourceRoot, pass::NoRemarks};
use vize_l1::markup::parse;
use vize_l1_to_l2::lower_source_block;
use vize_l2::{
    analysis::{static_analysis, template, template_complexity},
    provenance::NoProvenance,
};

fn samples(label: &str, ops: u32, reps: u32, original: impl Fn(), shared: impl Fn()) {
    let time = |call: &dyn Fn()| {
        let start = Instant::now();
        for _ in 0..reps {
            call();
        }
        start.elapsed().as_nanos() / u128::from(reps)
    };
    original();
    shared();
    let mut pairs = Vec::new();
    for pair in 0..31 {
        let (old, new) = if pair % 2 == 0 {
            (time(&original), time(&shared))
        } else {
            let new = time(&shared);
            (time(&original), new)
        };
        pairs.push((old, new));
    }
    for (pair, (old, new)) in pairs.iter().enumerate() {
        eprintln!(
            "sample {label} pair={pair} ops={ops} reps={reps} independent_ns={old} shared_ns={new}"
        );
    }
}

#[test]
#[ignore = "paired time over an already lowered original artifact, ci-opt"]
fn paired_template_observations() {
    for (label, source, reps) in [
        ("empty", String::new(), 512), ("text", "hello".to_owned(), 512),
        ("small", "<div class='x'>{{ message }}</div>".to_owned(), 256),
        ("static", "<span title='x'>text</span>".repeat(128), 16),
        ("flow", "<template v-for='x in xs'><i v-if='x.ok && flag'>{{ x.name }}</i><b v-else>no</b></template>".repeat(64), 16),
        ("slots", "<Card :a='a' @click='act'><template #default='{row}'><i v-if='row.ok'>{{row.name}}</i></template></Card>".repeat(64),16),
    ] {
        let arena = Allocator::new();
        let root = SourceRoot::new(&source).expect("bounded profile fixture");
        let block = root.block(&source,0).expect("full source block");
        let (tree, errors) = parse(&arena,&source);
        let lowered = lower_source_block(&arena,&tree,&errors,block);
        let region = &lowered.artifact.root;
        let count = lowered.op_count;
        let expected = template_complexity::run(region,count);
        super::check(&lowered, &expected);
        let quiet = || {
            black_box((static_analysis::run(black_box(region),count,&mut NoProvenance),template_complexity::run(black_box(region),count)));
        };
        let jointly = || {
            black_box(template::run(black_box(region),count,&mut NoProvenance,&mut NoRemarks));
        };
        samples(label,count,reps,quiet,jointly);
        let observed = || {
            let (mut trace,mut rows)=(Vec::new(),super::Rows::default());
            black_box((static_analysis::run_remarked(black_box(region),count,&mut trace,&mut rows),template_complexity::run(black_box(region),count),trace,rows));
        };
        let jointly_observed = || {
            let (mut trace,mut rows)=(Vec::new(),super::Rows::default());
            black_box((template::run(black_box(region),count,&mut trace,&mut rows),trace,rows));
        };
        samples(&format!("{label}-observed"),count,reps,observed,jointly_observed);
    }
}

#[test]
#[ignore = "paired lookup of published owner facts, ci-opt"]
fn paired_owner_queries() {
    for (label, owners, repeats) in [
        ("one-owner", 1, 8192),
        ("32-owners", 32, 512),
        ("1024-owners", 1024, 16),
    ] {
        let source = "<i>ok</i>".repeat(owners);
        let arena = Allocator::new();
        let root = SourceRoot::new(&source).expect("bounded source");
        let (tree, errors) = parse(&arena, &source);
        let lowered = lower_source_block(
            &arena,
            &tree,
            &errors,
            root.block(&source, 0).expect("full block"),
        );
        let region = &lowered.artifact.root;
        let count = lowered.op_count;
        let old = static_analysis::run(region, count, &mut NoProvenance);
        let new = template::run(region, count, &mut NoProvenance, &mut NoRemarks).static_facts;
        let ids: Vec<_> = (0..count)
            .filter_map(vize_l0::id::NodeId::from_index)
            .collect();
        assert_eq!(old.sorted_entries(), new.sorted_entries());
        for id in &ids {
            assert_eq!(old.get(*id), new.get(*id));
        }
        let query = |get: &dyn Fn(vize_l0::id::NodeId) -> bool| {
            let found = ids.iter().filter(|id| get(**id)).count();
            black_box(found);
        };
        samples(
            label,
            count,
            repeats,
            || {
                query(&|id| {
                    old.get(black_box(id))
                        .is_some_and(|fact| fact.native_descendants)
                });
            },
            || {
                query(&|id| {
                    new.get(black_box(id))
                        .is_some_and(|fact| fact.native_descendants)
                });
            },
        );
    }
}
