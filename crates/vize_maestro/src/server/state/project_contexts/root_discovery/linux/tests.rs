use std::{fs, os::unix::ffi::OsStrExt};
use vize_l0::cstr;

use super::{DirectoryMarkers, directory_markers, observe_records};
use crate::server::state::project_contexts::root_discovery::standard_directory_markers;

fn record(name: &[u8], file_type: u8) -> Vec<u8> {
    let length = (19 + name.len() + 1).next_multiple_of(8);
    let mut bytes = vec![0_u8; length];
    bytes[16..18].copy_from_slice(&(length as u16).to_ne_bytes());
    bytes[18] = file_type;
    bytes[19..19 + name.len()].copy_from_slice(name);
    bytes
}

#[test]
fn records_preserve_names_dot_filtering_and_type_independence() {
    let mut bytes = Vec::new();
    for (name, file_type) in [
        (b".".as_slice(), libc::DT_DIR),
        (b"..".as_slice(), libc::DT_DIR),
        (b"source.vue".as_slice(), libc::DT_REG),
        (b"vite.config.\xff".as_slice(), libc::DT_UNKNOWN),
        (b"tsconfig.\xff.json".as_slice(), libc::DT_UNKNOWN),
        (b"package.json".as_slice(), libc::DT_DIR),
        ("vite.config.🌋".as_bytes(), libc::DT_LNK),
    ] {
        bytes.extend(record(name, file_type));
    }
    let mut markers = DirectoryMarkers::default();
    assert!(!observe_records(&bytes, &mut markers).unwrap());
    assert_eq!(
        markers,
        DirectoryMarkers {
            package: true,
            vite: true,
            typescript: false,
        }
    );
    bytes.extend(record("tsconfig.𐐀.json".as_bytes(), libc::DT_UNKNOWN));
    assert!(observe_records(&bytes, &mut markers).unwrap());
    assert_eq!(
        markers,
        DirectoryMarkers {
            typescript: true,
            ..Default::default()
        }
    );
}

#[test]
fn truncated_and_malformed_records_fail_without_out_of_bounds_reads() {
    let complete = record(b"source.vue", libc::DT_REG);
    for length in 1..complete.len() {
        assert!(observe_records(&complete[..length], &mut DirectoryMarkers::default()).is_err());
    }
    assert!(!observe_records(&[], &mut DirectoryMarkers::default()).unwrap());
    for length in [0_u16, 1, 18, 19, u16::MAX] {
        let mut bytes = complete.clone();
        bytes[16..18].copy_from_slice(&length.to_ne_bytes());
        assert!(observe_records(&bytes, &mut DirectoryMarkers::default()).is_err());
    }
    let mut no_terminator = complete.clone();
    no_terminator[19..].fill(b'x');
    assert!(observe_records(&no_terminator, &mut DirectoryMarkers::default()).is_err());
    let empty_name = record(b"", libc::DT_REG);
    assert!(observe_records(&empty_name, &mut DirectoryMarkers::default()).is_err());
}

#[test]
fn native_scans_match_standard_enumeration_across_multiple_kernel_batches() {
    let fixture = tempfile::tempdir().unwrap();
    for index in 0..3000 {
        fs::write(
            fixture
                .path()
                .join(cstr!("component-{index:04}.vue").as_str()),
            "",
        )
        .unwrap();
    }
    fs::write(
        fixture
            .path()
            .join(std::ffi::OsStr::from_bytes(b"vite.config.\xff")),
        "",
    )
    .unwrap();
    fs::write(fixture.path().join("package.json"), "{}").unwrap();
    for marker in [
        "vite.config.𐐀",
        "tsconfig.arbitrary.🌋.json",
        "jsconfig.json",
    ] {
        let path = fixture.path().join(marker);
        fs::write(&path, "{}").unwrap();
        assert_eq!(
            directory_markers(fixture.path()).unwrap(),
            standard_directory_markers(fixture.path())
        );
        fs::remove_file(path).unwrap();
        assert_eq!(
            directory_markers(fixture.path()).unwrap(),
            standard_directory_markers(fixture.path())
        );
    }
}

#[test]
fn each_lookup_observes_renames_symlinks_and_replaced_directories() {
    use std::os::unix::fs::symlink;
    let fixture = tempfile::tempdir().unwrap();
    let directory = fixture.path().join("src");
    fs::create_dir(&directory).unwrap();
    let alias = fixture.path().join("alias");
    symlink(&directory, &alias).unwrap();
    assert_eq!(
        directory_markers(&alias).unwrap(),
        DirectoryMarkers::default()
    );
    fs::write(directory.join("incoming"), "{}").unwrap();
    fs::rename(
        directory.join("incoming"),
        directory.join("tsconfig.external.json"),
    )
    .unwrap();
    assert!(directory_markers(&alias).unwrap().typescript);
    fs::rename(&directory, fixture.path().join("old-src")).unwrap();
    fs::create_dir(&directory).unwrap();
    assert_eq!(
        directory_markers(&alias).unwrap(),
        DirectoryMarkers::default()
    );
    assert_eq!(
        directory_markers(&directory).unwrap(),
        standard_directory_markers(&directory)
    );
    fs::write(directory.join("vite.config."), "{}").unwrap();
    assert!(directory_markers(&alias).unwrap().vite);
}

#[test]
fn missing_and_non_directory_paths_use_the_existing_fallback() {
    let fixture = tempfile::tempdir().unwrap();
    let file = fixture.path().join("ordinary-file");
    fs::write(&file, "").unwrap();
    for path in [file, fixture.path().join("absent")] {
        assert!(directory_markers(&path).is_err());
        assert_eq!(
            super::super::directory_markers(&path),
            standard_directory_markers(&path)
        );
    }
}
