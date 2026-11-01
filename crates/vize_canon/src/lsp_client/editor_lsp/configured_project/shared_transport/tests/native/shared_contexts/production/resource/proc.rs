//! Complete owned Linux process observations for the finite discriminator.
use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn tick() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros()
}

pub(super) fn child_pids(pid: u32) -> Vec<u32> {
    let mut result = Vec::new();
    for task in fs::read_dir(cstr!("/proc/{pid}/task").as_str()).unwrap() {
        let bytes = match fs::read_to_string(task.unwrap().path().join("children")) {
            Ok(value) => value,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!("cannot read children: {error}"),
        };
        result.extend(
            bytes
                .split_whitespace()
                .map(|value| value.parse::<u32>().unwrap()),
        );
    }
    result.sort_unstable();
    result.dedup();
    result
}

fn process_life(pid: u32) -> control::ProcessLife {
    let base = std::path::PathBuf::from(cstr!("/proc/{pid}").as_str());
    let stat = fs::read_to_string(base.join("stat")).unwrap();
    let fields = stat
        .rsplit_once(") ")
        .unwrap()
        .1
        .split_whitespace()
        .collect::<Vec<_>>();
    assert_ne!(fields[0], "Z");
    control::ProcessLife {
        pid,
        birth: fields[19].parse().unwrap(),
        executable: fs::read_link(base.join("exe")).unwrap(),
    }
}

pub(super) fn native_owners(root: &Path, native: &Path, count: usize) -> Vec<control::ProcessLife> {
    let pids = child_pids(std::process::id());
    assert_eq!(
        pids.len(),
        count,
        "all direct children must be real native owners"
    );
    pids.into_iter()
        .map(|pid| {
            let life = process_life(pid);
            assert_eq!(life.executable, native);
            assert_eq!(
                fs::read_link(cstr!("/proc/{pid}/cwd").as_str()).unwrap(),
                root
            );
            assert!(
                fs::read(cstr!("/proc/{pid}/cmdline").as_str())
                    .unwrap()
                    .split(|byte| *byte == 0)
                    .any(|arg| arg == b"--lsp")
            );
            assert!(child_pids(pid).is_empty(), "unexpected native descendants");
            life
        })
        .collect()
}

fn kib(raw: &str, key: &str) -> u64 {
    let line = raw.lines().find(|line| line.starts_with(key)).unwrap();
    let fields = line.split_whitespace().collect::<Vec<_>>();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[2], "kB");
    fields[1].parse().unwrap()
}

pub(super) fn sample(
    name: &str,
    root: &Path,
    native: &Path,
    count: usize,
) -> Vec<control::ProcessLife> {
    let started = tick();
    let owners = native_owners(root, native, count);
    let mut lives = vec![process_life(std::process::id())];
    lives.extend(owners.iter().cloned());
    let mut rows = Vec::new();
    let mut rss = 0;
    let mut pss = 0;
    for life in lives {
        let before = tick();
        let base = std::path::PathBuf::from(cstr!("/proc/{}", life.pid).as_str());
        let stat_before = fs::read_to_string(base.join("stat")).unwrap();
        let status = fs::read_to_string(base.join("status")).unwrap();
        let rollup = fs::read_to_string(base.join("smaps_rollup")).unwrap();
        let exe = fs::read_link(base.join("exe")).unwrap();
        let cwd = fs::read_link(base.join("cwd")).unwrap();
        let cmdline = fs::read(base.join("cmdline")).unwrap();
        let stat_after = fs::read_to_string(base.join("stat")).unwrap();
        assert_eq!(process_life(life.pid), life);
        let rss_kib = kib(&rollup, "Rss:");
        let pss_kib = kib(&rollup, "Pss:");
        rss += rss_kib;
        pss += pss_kib;
        let image_sha = if life.pid == std::process::id() {
            std::env::var("VIZE_RESOURCE_TEST_IMAGE_SHA256").unwrap()
        } else {
            "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70".into()
        };
        rows.push(
            json!({"pid":life.pid,"birth":life.birth,"executable":exe,"executableSHA256":image_sha,
            "cwd":cwd,"cmdlineBytes":cmdline,"statBefore":stat_before,"statAfter":stat_after,
            "status":status,"smapsRollup":rollup,"rssKiB":rss_kib,"pssKiB":pss_kib,
            "collectionStartUnixUs":before,"collectionEndUnixUs":tick()}),
        );
    }
    assert_eq!(native_owners(root, native, count), owners);
    record(
        cstr!("resource-memory-{name}").as_str(),
        &json!({"milestone":name,
        "collectionStartUnixUs":started,"collectionEndUnixUs":tick(),"rows":rows,
        "treeRssKiB":rss,"treePssKiB":pss,"processCount":count+1,
        "scope":"quiescent synchronous common collection interval; not simultaneous atomic sample or peak"}),
    );
    owners
}
