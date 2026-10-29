//! Compare the real planner with its source-pinned baseline, including work.
use super::{ShardPlan, VirtualProject};
use std::{cell::RefCell, collections::BTreeSet, fs};
mod baseline;

#[derive(Default, Debug)]
struct Work {
    attempts: usize,
    calls: usize,
    members: Vec<(usize, usize)>,
}
thread_local! { static WORK: RefCell<Option<Work>> = const { RefCell::new(None) }; }
pub(super) fn start_partition() {
    WORK.with(|value| {
        if let Some(work) = value.borrow_mut().as_mut() {
            work.attempts += 1;
        }
    });
}
pub(super) fn record_call() {
    WORK.with(|value| {
        if let Some(work) = value.borrow_mut().as_mut() {
            work.calls += 1;
        }
    });
}
pub(super) fn record_member(index: usize) {
    WORK.with(|value| {
        if let Some(work) = value.borrow_mut().as_mut() {
            work.members.push((work.attempts, index));
        }
    });
}
fn measure<'a>(run: impl FnOnce() -> ShardPlan<'a>) -> (ShardPlan<'a>, Work) {
    WORK.with(|value| *value.borrow_mut() = Some(Work::default()));
    let plan = run();
    let work = WORK.with(|value| value.borrow_mut().take().unwrap());
    (plan, work)
}
fn check(project: &VirtualProject, servers: usize, label: &str) {
    let (expected, before) = measure(|| baseline::partition(project, servers, true));
    let (actual, after) = measure(|| super::partition(project, servers, true));
    assert_eq!(
        actual.shards, expected.shards,
        "{label}/{servers} shard membership/order"
    );
    assert_eq!(
        actual.owners, expected.owners,
        "{label}/{servers} ownership"
    );
    assert_eq!(
        after.attempts, before.attempts,
        "{label}/{servers} fallback control flow"
    );
    if label == "leaf-admitted" && servers > 1 {
        assert_eq!(
            actual.shards.len(),
            servers,
            "positive shared-leaf path skipped"
        );
        assert_eq!(after.attempts, 1, "positive shared-leaf path fell back");
    }
    let unique: BTreeSet<_> = after.members.iter().copied().collect();
    assert_eq!(
        after.members.len(),
        unique.len(),
        "{label}/{servers} repeats a member in one attempt"
    );
    if before.calls == 0 {
        assert_eq!(after.calls, 0, "{label}/{servers} early return gained work");
    } else {
        assert!(before.calls >= after.calls * 2);
        assert!(before.members.len() >= after.members.len() * 2);
    }
    eprintln!(
        "partition-weight-work label={label} servers={servers} old_calls={} old_member_reads={} new_calls={} new_member_reads={} attempts={} shards={}",
        before.calls,
        before.members.len(),
        after.calls,
        after.members.len(),
        after.attempts,
        actual.shards.len()
    );
}
fn build(shape: &str) -> (tempfile::TempDir, VirtualProject) {
    let keep = tempfile::tempdir().unwrap();
    let root = keep.path().canonicalize().unwrap();
    fs::create_dir(root.join("src")).unwrap();
    fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"target":"ES2022","module":"ESNext","moduleResolution":"Bundler","strict":true,"jsx":"preserve","paths":{"@/*":["./src/*"]}},"include":["src"]}"#).unwrap();
    let count = match shape {
        "empty" => 0,
        "singleton" => 1,
        _ => 12,
    };
    let mut paths = Vec::new();
    let pad = "x".repeat(8192);
    for index in 0..count {
        let mut script = format!("const n = {index};\n");
        if shape == "alias" {
            let helper = root.join(format!("src/H{index}.ts"));
            fs::write(&helper, "export const n = 1;\n").unwrap();
            paths.push(helper);
            script = format!("import {{ n }} from '@/H{index}';\n");
        } else if shape == "scc" || shape == "connected" {
            let next = if shape == "connected" {
                (index + 1) % count
            } else {
                (index / 3) * 3 + (index + 1) % 3
            };
            script = format!("import Next from './C{next}.vue';\nvoid Next;\nconst n = {index};\n");
        } else if matches!(shape, "leaf" | "leaf-admitted" | "global-fallback") {
            script = "import { n } from './shared';\n".to_owned();
        }
        let extra = if shape == "heavy" && index == 0 {
            "x".repeat(1024 * 1024)
        } else {
            pad.clone()
        };
        let path = root.join(format!("src/C{index}.vue"));
        fs::write(&path, format!("<script setup lang=\"ts\">\n{script}/* {extra} */\n</script><template>{{{{ n }}}}</template>")).unwrap();
        paths.push(path);
    }
    if matches!(shape, "leaf" | "leaf-admitted" | "global-fallback") {
        let leaf = root.join("src/shared.ts");
        fs::write(&leaf, "export const n = 1;\n").unwrap();
        paths.push(leaf);
    }
    if shape == "global-fallback" {
        let loose = root.join("src/loose.ts");
        fs::write(&loose, "interface VisibleEverywhere { value: number }\n").unwrap();
        paths.push(loose);
    }
    if shape == "global" {
        let global = root.join("src/globals.d.ts");
        fs::write(
            &global,
            "export {}; declare global { interface Window { shared: number } }\n",
        )
        .unwrap();
        paths.push(global);
    }
    let mut project = VirtualProject::new(&root).unwrap();
    project.register_paths(&paths).unwrap();
    if shape == "leaf-admitted" {
        project.materialize().unwrap();
    }
    (keep, project)
}
#[test]
fn cached_weights_preserve_complete_plans_and_read_each_member_once() {
    for shape in [
        "empty",
        "singleton",
        "equal",
        "heavy",
        "alias",
        "scc",
        "connected",
        "leaf",
        "leaf-admitted",
        "global-fallback",
        "global",
    ] {
        let (_keep, project) = build(shape);
        for servers in [1, 2, 8] {
            check(&project, servers, shape);
        }
    }
}
#[test]
fn cached_weight_storage_tradeoff_is_explicit() {
    let extra = std::mem::size_of::<(usize, Vec<usize>)>() - std::mem::size_of::<Vec<usize>>();
    assert_eq!(extra, std::mem::size_of::<usize>());
    // This is temporary group metadata, not an RSS/allocation-byte claim.
    eprintln!("partition-weight-extra-bytes-per-group={extra}");
}
