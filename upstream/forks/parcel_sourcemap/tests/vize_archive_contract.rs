//! Complete authored map contracts; JSON cases require the explicitly qualified json feature.
use parcel_sourcemap::{Mapping, OriginalLocation, SourceMap, SourceMapErrorType, SourceMapInner};
use std::{
    io,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

const CONTENTS: [&str; 2] = [
    "<style>\n.x { color: red; }\n</style>\n",
    "/* second */\n.y { color: blue; }\n",
];
const VLQ: &str = "AACAA,K;ACCGC,QDDD";
#[cfg(feature = "json")]
const JSON: &str = r#"{"version":3,"sourceRoot":"/authored","mappings":"AACAA,K;ACCGC,QDDD","sources":["src/input.vue","src/second.vue"],"sourcesContent":["<style>\n.x { color: red; }\n</style>\n","/* second */\n.y { color: blue; }\n"],"names":["x","color"]}"#;
#[cfg(feature = "json")]
const DATA_URL: &str = "data:application/json;charset=utf-8;base64,eyJ2ZXJzaW9uIjozLCJzb3VyY2VSb290IjoiL2F1dGhvcmVkIiwibWFwcGluZ3MiOiJBQUNBQSxLO0FDQ0dDLFFEREQiLCJzb3VyY2VzIjpbInNyYy9pbnB1dC52dWUiLCJzcmMvc2Vjb25kLnZ1ZSJdLCJzb3VyY2VzQ29udGVudCI6WyI8c3R5bGU+XG4ueCB7IGNvbG9yOiByZWQ7IH1cbjwvc3R5bGU+XG4iLCIvKiBzZWNvbmQgKi9cbi55IHsgY29sb3I6IGJsdWU7IH1cbiJdLCJuYW1lcyI6WyJ4IiwiY29sb3IiXX0=";

fn mapping(line: u32, column: u32, original: Option<(u32, u32, u32, Option<u32>)>) -> Mapping {
    Mapping {
        generated_line: line,
        generated_column: column,
        original: original
            .map(|(line, column, source, name)| OriginalLocation::new(line, column, source, name)),
    }
}
fn expected_mappings() -> Vec<Mapping> {
    vec![
        mapping(0, 0, Some((1, 0, 0, Some(0)))),
        mapping(0, 5, None),
        mapping(1, 0, Some((2, 3, 1, Some(1)))),
        mapping(1, 8, Some((1, 2, 0, None))),
    ]
}
fn written(map: &mut SourceMap) -> String {
    let mut bytes = Vec::new();
    map.write_vlq(&mut bytes).unwrap();
    String::from_utf8(bytes).unwrap()
}
fn assert_complete(map: &SourceMap, expected: &[Mapping]) {
    assert_eq!(map.project_root, "/project");
    assert_eq!(
        map.get_sources(),
        &vec!["src/input.vue".to_owned(), "src/second.vue".to_owned()]
    );
    assert_eq!(
        map.get_sources_content(),
        &CONTENTS.iter().map(|s| s.to_string()).collect::<Vec<_>>()
    );
    assert_eq!(map.get_names(), &vec!["x".to_owned(), "color".to_owned()]);
    assert_eq!(map.get_mappings(), expected);
}
fn authored() -> SourceMap {
    let mut map = SourceMap::new("/project");
    assert_eq!(
        map.add_sources(vec!["/project/src/input.vue", "/project/src/second.vue"]),
        vec![0, 1]
    );
    assert_eq!(map.add_names(vec!["x", "color", "x"]), vec![0, 1, 0]);
    for (i, content) in CONTENTS.iter().enumerate() {
        map.set_source_content(i, content).unwrap();
    }
    // Deliberately unordered insertion: the literal VLQ oracle also qualifies sorting.
    for row in [
        mapping(0, 5, None),
        mapping(0, 0, Some((1, 0, 0, Some(0)))),
        mapping(1, 8, Some((1, 2, 0, None))),
        mapping(1, 0, Some((2, 3, 1, Some(1)))),
    ] {
        map.add_mapping(row.generated_line, row.generated_column, row.original);
    }
    assert_eq!(written(&mut map), VLQ);
    assert_complete(&map, &expected_mappings());
    map
}

#[cfg(feature = "json")]
#[test]
fn complete_json_data_url_and_vlq_roundtrip() {
    let mut map = authored();
    assert_eq!(map.to_json(Some("/authored")).unwrap(), JSON);
    assert_eq!(map.to_data_url(Some("/authored")).unwrap(), DATA_URL);
    for mut decoded in [
        SourceMap::from_json("/project", JSON).unwrap(),
        SourceMap::from_data_url("/project", DATA_URL).unwrap(),
    ] {
        assert_complete(&decoded, &expected_mappings());
        assert_eq!(decoded.to_json(Some("/authored")).unwrap(), JSON);
        assert_eq!(written(&mut decoded), VLQ);
    }
    assert!(SourceMap::from_json("/project", "{invalid}").is_err());
    assert!(SourceMap::from_data_url("/project", "data:text/plain;base64,Zm9v").is_err());
}

#[test]
fn complete_offsets_and_source_name_composition() {
    let mut offset = SourceMap::new("/project");
    offset
        .add_vlq_map(
            VLQ.as_bytes(),
            vec!["src/input.vue", "src/second.vue"],
            CONTENTS.to_vec(),
            vec!["x", "color"],
            2,
            4,
        )
        .unwrap();
    let mut expected = expected_mappings();
    for row in &mut expected {
        row.generated_line += 2;
        row.generated_column += 4;
    }
    assert_complete(&offset, &expected);
    assert_eq!(written(&mut offset), ";;IACAA,K;ICCGC,QDDD");
    offset.offset_columns(3, 8, 2).unwrap();
    expected[3].generated_column += 2;
    offset.offset_lines(3, 1).unwrap();
    expected[2].generated_line += 1;
    expected[3].generated_line += 1;
    assert_complete(&offset, &expected);
    assert_eq!(written(&mut offset), ";;IACAA,K;;ICCGC,UDDD");
    let mut merged = SourceMap::new("/project");
    assert_eq!(merged.add_source("pre.css"), 0);
    merged.set_source_content(0, "/* pre */").unwrap();
    assert_eq!(merged.add_name("prior"), 0);
    merged.add_mapping(0, 0, None);
    merged.add_sourcemap(&mut authored(), 2).unwrap();
    assert_eq!(
        merged.get_sources(),
        &vec![
            "pre.css".to_owned(),
            "src/input.vue".to_owned(),
            "src/second.vue".to_owned()
        ]
    );
    assert_eq!(
        merged.get_sources_content(),
        &vec![
            "/* pre */".to_owned(),
            CONTENTS[0].to_owned(),
            CONTENTS[1].to_owned()
        ]
    );
    assert_eq!(
        merged.get_names(),
        &vec!["prior".to_owned(), "x".to_owned(), "color".to_owned()]
    );
    let mut combined = vec![mapping(0, 0, None)];
    for mut row in expected_mappings() {
        row.generated_line += 2;
        if let Some(ref mut original) = row.original {
            original.source += 1;
            original.name = original.name.map(|n| n + 1);
        }
        combined.push(row);
    }
    assert_eq!(merged.get_mappings(), combined);
    assert_eq!(written(&mut merged), "A;;ACCAC,K;ACCGC,QDDD");
    assert!(merged
        .add_mapping_with_offset(mapping(0, 0, None), -1, 0)
        .is_err());
    assert!(merged
        .add_mapping_with_offset(mapping(0, 0, None), 0, -1)
        .is_err());
}

#[test]
fn complete_valid_archive_roundtrip() {
    let map = authored();
    let mut bytes = rkyv::util::AlignedVec::new();
    map.to_buffer(&mut bytes).unwrap();
    assert!(!bytes.is_empty());
    // This checked access also makes the bytecheck feature a compilation requirement.
    let checked =
        rkyv::access::<rkyv::Archived<SourceMapInner>, rkyv::rancor::Error>(&bytes).unwrap();
    assert_eq!(checked.sources.len(), 2);
    let mut restored = SourceMap::from_buffer("/project", &bytes).unwrap();
    assert_complete(&restored, &expected_mappings());
    assert_eq!(written(&mut restored), VLQ);
    #[cfg(feature = "json")]
    assert_eq!(restored.to_json(Some("/authored")).unwrap(), JSON);
    let mut again = rkyv::util::AlignedVec::new();
    restored.to_buffer(&mut again).unwrap();
    let restored_again = SourceMap::from_buffer("/project", &again).unwrap();
    assert_complete(&restored_again, &expected_mappings());
}

fn refusal_checks() {
    let map = authored();
    let mut bytes = rkyv::util::AlignedVec::new();
    map.to_buffer(&mut bytes).unwrap();
    let root = rkyv::access::<rkyv::Archived<SourceMapInner>, rkyv::rancor::Error>(&bytes).unwrap();
    // Pinned rkyv0.8 default32-bit RelPtr is the first repr(C) ArchivedVec field.
    assert_eq!(std::mem::size_of::<rkyv::Archived<isize>>(), 4);
    let pointer_offset = (&root.sources as *const _ as usize) - bytes.as_ptr() as usize;
    assert!(pointer_offset + 4 <= bytes.len());
    let mut corrupted = bytes.to_vec();
    corrupted[pointer_offset..pointer_offset + 4].copy_from_slice(&i32::MAX.to_le_bytes());
    let cases = [
        ("empty", vec![]),
        ("malformed-root", vec![255; bytes.len()]),
        ("truncated-tail", bytes[..bytes.len() - 1].to_vec()),
        ("out-of-bounds-pointer", corrupted),
    ];
    for (name, input) in cases.iter() {
        let mut aligned: rkyv::util::AlignedVec = rkyv::util::AlignedVec::new();
        aligned.extend_from_slice(input);
        let error = SourceMap::from_buffer("/project", &aligned).expect_err(name);
        assert!(
            matches!(error.error_type, SourceMapErrorType::BufferError),
            "{}: {:?}",
            name,
            error
        );
        println!("VIZE_ARCHIVE_REFUSED={}", name);
    }
    println!("VIZE_ARCHIVE_REFUSAL_COMPLETE=4");
}

#[test]
fn archive_validation_is_bounded_process_contract() -> io::Result<()> {
    if std::env::var_os("VIZE_PARCEL_ARCHIVE_CHILD").is_some() {
        refusal_checks();
        return Ok(());
    }
    let mut child = Command::new(std::env::current_exe()?)
        .args([
            "--exact",
            "archive_validation_is_bounded_process_contract",
            "--nocapture",
        ])
        .env("VIZE_PARCEL_ARCHIVE_CHILD", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill()?;
            let output = child.wait_with_output()?;
            panic!(
                "archive child timed out: status={} stdout={} stderr={}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!(
        "archive child status={}\nstdout:\n{}\nstderr:\n{}",
        output.status, stdout, stderr
    );
    assert!(
        output.status.success(),
        "archive child must return normally"
    );
    assert_eq!(stdout.matches("VIZE_ARCHIVE_REFUSED=").count(), 4);
    assert_eq!(stdout.matches("VIZE_ARCHIVE_REFUSAL_COMPLETE=4").count(), 1);
    for name in [
        "empty",
        "malformed-root",
        "truncated-tail",
        "out-of-bounds-pointer",
    ] {
        assert_eq!(
            stdout
                .matches(&format!("VIZE_ARCHIVE_REFUSED={}\n", name))
                .count(),
            1
        );
    }
    Ok(())
}
