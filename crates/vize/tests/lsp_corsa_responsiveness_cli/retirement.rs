//! Observe the held API owner before any test cleanup can kill it.
use super::lsp_process::LspProcess;
use std::path::Path;

#[cfg(target_os = "linux")]
pub(super) fn capture(root: &Path, lsp: &LspProcess) {
    use serde_json::json;
    use std::time::{Duration, Instant};
    let started = Instant::now();
    loop {
        let mut identities = Vec::new();
        for pid in std::fs::read_to_string(root.join("backend.pids"))
            .unwrap()
            .lines()
        {
            let pid: u32 = pid.parse().unwrap();
            let process = Path::new("/proc").join(vize_l0::cstr!("{pid}").as_str());
            let stat = std::fs::read_to_string(process.join("stat")).unwrap();
            let fields: Vec<_> = stat
                .rsplit_once(") ")
                .unwrap()
                .1
                .split_whitespace()
                .collect();
            assert_eq!(fields[1].parse::<u32>().unwrap(), lsp.process_id());
            assert!(fields[19].parse::<u64>().is_ok());
            let image = std::fs::read_link(process.join("exe")).unwrap();
            if image.file_name().is_some_and(|name| name == "cat") {
                identities.push(json!({"pid":pid,"birthTicks":fields[19],"parent":lsp.process_id(),"image":image,"wholeStat":stat}));
            }
        }
        let pids: Vec<u32> = std::fs::read_to_string(root.join("backend.pids"))
            .unwrap()
            .lines()
            .map(|pid| pid.parse().unwrap())
            .collect();
        if !identities.is_empty() && identities.len() == pids.len() {
            std::fs::write(
                root.join("backend-identity.json"),
                serde_json::to_vec_pretty(&json!({"identities":identities,"pids":pids})).unwrap(),
            )
            .unwrap();
            return;
        }
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "held backend never exec'd its actual owned cat image"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(target_os = "linux")]
pub(super) fn assert_reaped(root: &Path, lsp: &LspProcess) {
    use serde_json::{Value, json};
    let before: Value =
        serde_json::from_slice(&std::fs::read(root.join("backend-identity.json")).unwrap())
            .unwrap();
    let pids: Vec<u32> = std::fs::read_to_string(root.join("backend.pids"))
        .unwrap()
        .lines()
        .map(|pid| pid.parse().unwrap())
        .collect();
    let mut probes = Vec::new();
    for identity in before["identities"].as_array().unwrap() {
        let pid = identity["pid"].as_u64().unwrap();
        let probe = match std::fs::read_to_string(
            Path::new("/proc")
                .join(vize_l0::cstr!("{pid}").as_str())
                .join("stat"),
        ) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                json!({"pid":pid,"reaped":true,"observation":"absent"})
            }
            Err(error) => json!({"pid":pid,"reaped":false,"error":vize_l0::cstr!("{error}")}),
            Ok(stat) => {
                let fields: Vec<_> = stat
                    .rsplit_once(") ")
                    .unwrap()
                    .1
                    .split_whitespace()
                    .collect();
                let birth = fields.get(19).copied();
                let reaped = birth.is_some_and(|birth| {
                    birth.parse::<u64>().is_ok() && Some(birth) != identity["birthTicks"].as_str()
                });
                json!({"pid":pid,"reaped":reaped,"birthTicks":birth,"state":fields.first(),"wholeStat":stat})
            }
        };
        probes.push(probe);
    }
    let stderr = lsp.stderr_bytes();
    let journal: Vec<Value> = String::from_utf8_lossy(&stderr)
        .lines()
        .filter_map(|line| {
            if !line.contains("controlled native startup termination") {
                return None;
            }
            let (_, receipt) = line.split_once("receipt=").unwrap();
            Some(serde_json::from_str(receipt).unwrap())
        })
        .collect();
    let proof = json!({"before":before,"pidsAfter":pids,"afterServerExitBeforeTestCleanup":probes,"controlledStartupJournal":journal});
    if let Some(capture) = lsp.trace_dir() {
        std::fs::write(
            capture.join("held-startup-retirement.json"),
            serde_json::to_vec_pretty(&proof).unwrap(),
        )
        .unwrap();
    }
    println!("held startup retirement before test cleanup: {proof}");
    assert_eq!(
        json!(pids),
        before["pids"],
        "a late callback must not spawn another backend: {proof}"
    );
    for identity in before["identities"].as_array().unwrap() {
        let terminal = journal
            .iter()
            .find(|entry| entry["terminal"]["pid"] == identity["pid"])
            .expect("actual controlled-abort terminal acknowledgment");
        assert_eq!(terminal["controlledStartupAbort"], true);
        assert_eq!(terminal["role"], "api");
        assert_eq!(terminal["terminal"]["reaped"], true);
        assert_eq!(terminal["terminal"]["killRequested"], true);
        assert_eq!(terminal["terminal"]["timedOut"], false);
        assert_eq!(
            terminal["terminal"]["status"],
            "ExitStatus(unix_wait_status(9))"
        );
        assert_eq!(terminal["stderr"]["custody"], "inherited-server-stderr");
    }
    assert!(
        probes.iter().all(|probe| probe["reaped"] == true),
        "same-birth backend remains before BackendCleanup: {proof}"
    );
}

#[cfg(not(target_os = "linux"))]
pub(super) fn capture(_: &Path, _: &LspProcess) {}
#[cfg(not(target_os = "linux"))]
pub(super) fn assert_reaped(_: &Path, _: &LspProcess) {}
