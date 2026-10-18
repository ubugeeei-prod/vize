//! Process-only regressions: no GitHub writes, network, Cargo or native toolchain.
use super::*;
use std::time::{Duration, Instant};

fn fixture(script: &str) -> Value {
    json!({"program":"node","args":["-e",script]})
}

#[test]
fn thirteen_independent_reads_share_one_deadline_and_keep_response_order() {
    let root = private_directory().unwrap();
    let clock = Compensation::limited(3000).unwrap();
    let commands: Vec<_> = (0..13)
        .map(|index| {
            fixture(&format!(
                "setTimeout(()=>process.stdout.write(JSON.stringify({{index:{index}}})),400)"
            ))
        })
        .collect();
    let mut retained = Vec::new();
    let started = Instant::now();
    let packets = clock
        .run(&commands, &root, &mut |name, value| {
            retained.push((name.to_owned(), value.clone()));
            Ok(())
        })
        .unwrap();
    assert!(started.elapsed() < Duration::from_secs(3));
    assert_eq!(packets.len(), 13);
    for (index, packet) in packets.iter().enumerate() {
        assert_eq!(
            serde_json::from_str::<Value>(&clock.text(packet).unwrap()).unwrap(),
            json!({"index":index})
        );
        assert_eq!(packet["deadline"], clock.deadline);
        assert!(packet["pid"].as_u64().unwrap() > 0);
        assert!(packet["started"].as_u64().unwrap() <= packet["finished"].as_u64().unwrap());
        assert_eq!(packet["completion"], "closed");
    }
    assert_eq!(
        retained
            .iter()
            .filter(|(name, _)| name == "compensation-process")
            .count(),
        13
    );
    assert_eq!(retained.last().unwrap().0, "compensation-outer");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn outer_group_kill_keeps_exact_binary_prefixes_and_unknown_child_completion() {
    let root = private_directory().unwrap();
    let clock = Compensation::limited(1000).unwrap();
    let child = "require('node:child_process').spawn(process.execPath,['-e','setInterval(()=>{},1000)'],{stdio:'inherit'});process.stdout.write(Buffer.from([0,255,10]));process.stderr.write(Buffer.from([254,1]));process.exit(0)";
    let mut retained = Vec::new();
    let started = Instant::now();
    assert!(
        clock
            .run(&[fixture(child)], &root, &mut |_, value| {
                retained.push(value.clone());
                Ok(())
            })
            .is_err()
    );
    assert!(started.elapsed() < Duration::from_secs(4));
    let packet = &retained[0];
    assert_eq!(packet["stdout"], json!([0, 255, 10]));
    assert_eq!(packet["stderr"], json!([254, 1]));
    assert!(packet["pid"].as_u64().unwrap() > 0);
    assert!(packet["started"].as_u64().is_some());
    assert_eq!(packet["exitObserved"]["status"], 0);
    assert!(packet["exitObserved"]["finished"].as_u64().is_some());
    assert_eq!(packet["finished"], Value::Null);
    assert_eq!(packet["status"], Value::Null);
    assert_eq!(packet["completion"], "unknown-after-outer-exit");
    assert_eq!(packet["outputComplete"], false);
    let outer = retained.last().unwrap();
    assert!(outer["signal"] == 9 || outer["status"] == 137);
    assert!(outer["pid"].as_u64().unwrap() > 0);
    assert_eq!(outer["innerOutput"], "missing-or-partial");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn each_stream_has_its_own_exact_two_mib_prefix_and_overflow_refusal() {
    let root = private_directory().unwrap();
    let clock = Compensation::limited(5000).unwrap();
    let commands = [
        fixture("process.stdout.write(Buffer.alloc(3*1024*1024,255))"),
        fixture("process.stderr.write(Buffer.alloc(3*1024*1024,254))"),
        fixture(
            "process.stdout.write(Buffer.alloc(2*1024*1024,253));process.stderr.write(Buffer.alloc(2*1024*1024,252))",
        ),
    ];
    let packets = clock.run(&commands, &root, &mut |_, _| Ok(())).unwrap();
    for (index, stream, byte) in [(0, "stdout", 255_u8), (1, "stderr", 254)] {
        let bytes: Vec<u8> = serde_json::from_value(packets[index][stream].clone()).unwrap();
        assert_eq!(bytes, vec![byte; STREAM_CAP]);
        assert_eq!(packets[index]["error"]["code"], "ENOBUFS");
        assert_eq!(packets[index]["outputComplete"], false);
        assert!(clock.text(&packets[index]).is_err());
    }
    assert_eq!(packets[2]["status"], 0);
    assert_eq!(packets[2]["outputComplete"], true);
    for (stream, byte) in [("stdout", 253_u8), ("stderr", 252)] {
        let bytes: Vec<u8> = serde_json::from_value(packets[2][stream].clone()).unwrap();
        assert_eq!(bytes, vec![byte; STREAM_CAP]);
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn finite_batch_is_only_explicit_rest_get_and_refuses_option_or_path_escape() {
    let commands = read_commands(
        "owner/repo",
        &["pulls/42", "commits/abc/check-runs?per_page=100"],
    )
    .unwrap();
    assert_eq!(
        commands,
        vec![
            json!({"program":"gh","args":["api","repos/owner/repo/pulls/42","--method","GET"]}),
            json!({"program":"gh","args":["api","repos/owner/repo/commits/abc/check-runs?per_page=100","--method","GET"]}),
        ]
    );
    assert!(read_commands("owner/repo", &[]).is_err());
    assert!(read_commands("owner/repo", &["pulls/42"; 14]).is_err());
    for repository in [
        "owner",
        "owner/repo/extra",
        "../repo",
        "owner/--method POST",
    ] {
        assert!(read_commands(repository, &["pulls/42"]).is_err());
    }
    for endpoint in [
        "",
        "/pulls/42",
        "../graphql",
        "https://host/path",
        "pulls/42\n--method POST",
        "pulls/42#fragment",
        "pulls\\42",
    ] {
        assert!(
            read_commands("owner/repo", &[endpoint]).is_err(),
            "{endpoint}"
        );
    }
}

#[test]
fn failed_read_does_not_drop_other_packets_or_retry_any_child() {
    let root = private_directory().unwrap();
    let commands: Vec<_> = (0..13).map(|index| fixture(&format!(
        "require('node:fs').appendFileSync('calls','{index}\\n');process.stdout.write('{{\"index\":{index}}}');process.exit({})", if index == 5 {17} else {0}
    ))).collect();
    let clock = Compensation::limited(3000).unwrap();
    let packets = clock.run(&commands, &root, &mut |_, _| Ok(())).unwrap();
    assert_eq!(packets.len(), 13);
    assert!(clock.text(&packets[5]).is_err());
    for (index, packet) in packets.iter().enumerate() {
        assert_eq!(packet["status"], if index == 5 { 17 } else { 0 });
        assert_eq!(packet["completion"], "closed");
    }
    let calls = fs::read_to_string(root.join("calls")).unwrap();
    let mut observed: Vec<_> = calls
        .lines()
        .map(|line| line.parse::<usize>().unwrap())
        .collect();
    observed.sort();
    assert_eq!(observed, (0..13).collect::<Vec<_>>());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn subsequent_batch_does_not_restart_absolute_compensation_deadline() {
    let root = private_directory().unwrap();
    let clock = Compensation::limited(1100).unwrap();
    let first = clock
        .run(
            &[fixture("setTimeout(()=>process.stdout.write('first'),250)")],
            &root,
            &mut |_, _| Ok(()),
        )
        .unwrap();
    assert_eq!(clock.text(&first[0]).unwrap(), "first");
    let mut retained = Vec::new();
    assert!(
        clock
            .run(
                &[fixture(
                    "process.stdout.write('second-prefix');setInterval(()=>{},1000)"
                )],
                &root,
                &mut |_, value| {
                    retained.push(value.clone());
                    Ok(())
                }
            )
            .is_err()
    );
    assert!(
        retained
            .iter()
            .all(|packet| packet["deadline"] == clock.deadline)
    );
    assert_eq!(retained[0]["stdout"], json!(b"second-prefix".to_vec()));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn spawn_failure_is_distinct_from_absent_buffered_output() {
    let root = private_directory().unwrap();
    let clock = Compensation::limited(1000).unwrap();
    let packets = clock
        .run(
            &[json!({"program":root.join("missing-program"),"args":[]})],
            &root,
            &mut |_, _| Ok(()),
        )
        .unwrap();
    assert_eq!(packets[0]["pid"], 0);
    assert_eq!(packets[0]["error"]["code"], "ENOENT");
    assert_eq!(packets[0]["stdout"], json!([]));
    assert_eq!(packets[0]["stderr"], json!([]));
    assert_eq!(packets[0]["outputComplete"], false);
    assert!(clock.text(&packets[0]).is_err());
    fs::remove_dir_all(root).unwrap();
}
