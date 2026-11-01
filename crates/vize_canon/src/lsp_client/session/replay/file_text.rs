//! Compare a current Canon file without allocating another complete text copy.

use std::{
    fs::File,
    io::{ErrorKind, Read},
    path::Path,
};

const CHUNK_BYTES: usize = 8192;

/// The caller still repairs every mismatch or read failure through its original
/// write path. Nothing is memoized: external disk changes are read each time.
pub(super) fn matches(path: &Path, expected: &str) -> bool {
    let Ok(mut file) = File::open(path) else {
        return false;
    };
    reader_matches(&mut file, expected)
}

fn reader_matches(reader: &mut impl Read, expected: &str) -> bool {
    let mut remaining = expected.as_bytes();
    let mut buffer = [0u8; CHUNK_BYTES];
    loop {
        let count = match reader.read(&mut buffer) {
            Ok(count) => count,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(_) => return false,
        };
        if count == 0 {
            return remaining.is_empty();
        }
        let Some(bytes) = buffer.get(..count) else {
            return false;
        };
        let Some(rest) = remaining.strip_prefix(bytes) else {
            return false;
        };
        remaining = rest;
    }
}

#[cfg(test)]
mod tests {
    use super::{CHUNK_BYTES, matches, reader_matches};
    use std::{fs, io, io::Read};

    fn original(path: &std::path::Path, expected: &str) -> bool {
        fs::read_to_string(path).ok().as_deref() == Some(expected)
    }

    #[test]
    fn complete_file_corpus_matches_the_original_text_condition() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("document.vue.ts");
        let large = "雪🌸\r\n".repeat(CHUNK_BYTES);
        let boundary = "x".repeat(CHUNK_BYTES);
        let corpus = [
            "",
            "a",
            "snow 雪🌸\r\nsecond\n",
            "\u{feff}BOM",
            "nul\0byte",
            &boundary,
            &large,
        ];
        let mut comparisons = 0;
        for actual in corpus {
            fs::write(&file, actual).unwrap();
            for expected in corpus {
                assert_eq!(matches(&file, expected), original(&file, expected));
                comparisons += 1;
            }
        }
        assert_eq!(comparisons, 49);
    }

    #[test]
    fn boundary_prefix_suffix_and_invalid_utf8_preserve_the_whole_condition() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("document.vue.ts");
        for size in [
            CHUNK_BYTES - 1,
            CHUNK_BYTES,
            CHUNK_BYTES + 1,
            2 * CHUNK_BYTES + 5,
        ] {
            let expected = "x".repeat(size);
            for actual in [
                expected.as_bytes().to_vec(),
                [expected.as_bytes(), b"extra"].concat(),
                b"short".to_vec(),
                [expected.as_bytes(), &[0xff]].concat(),
            ] {
                fs::write(&file, actual).unwrap();
                assert_eq!(matches(&file, &expected), original(&file, &expected));
            }
        }
        assert_eq!(matches(&file, "valid"), original(&file, "valid"));
        assert!(!matches(&file, "valid"));
    }

    #[test]
    fn external_equal_changed_deleted_and_recreated_files_are_never_cached() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("document.vue.ts");
        fs::write(&file, "export const value = '雪';\r\n").unwrap();
        let expected = "export const value = '雪';\r\n";
        assert!(matches(&file, expected));
        fs::write(&file, "external corruption").unwrap();
        assert!(!matches(&file, expected));
        fs::remove_file(&file).unwrap();
        assert_eq!(matches(&file, expected), original(&file, expected));
        assert!(!matches(&file, expected));
        fs::write(&file, expected).unwrap();
        assert!(matches(&file, expected));
    }

    #[test]
    fn repair_branch_and_final_bytes_match_for_equal_corrupt_missing_and_invalid_files() {
        let root = tempfile::tempdir().unwrap();
        let old = root.path().join("old.vue.ts");
        let current = root.path().join("current.vue.ts");
        let expected = "export const amount = '雪🌸';\r\n";
        let invalid = [0xff, 0xfe];
        for actual in [
            Some(expected.as_bytes()),
            Some(b"changed".as_slice()),
            Some(invalid.as_slice()),
            None,
        ] {
            for path in [&old, &current] {
                if path.exists() {
                    fs::remove_file(path).unwrap();
                }
                if let Some(bytes) = actual {
                    fs::write(path, bytes).unwrap();
                }
            }
            let old_changed = !original(&old, expected);
            let current_changed = !matches(&current, expected);
            assert_eq!(current_changed, old_changed);
            if old_changed {
                fs::write(&old, expected).unwrap();
            }
            if current_changed {
                fs::write(&current, expected).unwrap();
            }
            assert_eq!(fs::read(&current).unwrap(), fs::read(&old).unwrap());
        }
    }

    #[test]
    fn independent_paths_do_not_share_their_previous_answer() {
        let root = tempfile::tempdir().unwrap();
        let first = root.path().join("first.vue.ts");
        let second = root.path().join("second.vue.ts");
        fs::write(&first, "one").unwrap();
        fs::write(&second, "two").unwrap();
        assert!(matches(&first, "one"));
        assert!(!matches(&second, "one"));
        assert!(matches(&second, "two"));
        assert!(!matches(&first, "two"));
    }

    struct Chunks {
        bytes: Vec<u8>,
        cursor: usize,
        chunk: usize,
        interrupt: bool,
        fail_at: Option<usize>,
    }

    impl Read for Chunks {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if std::mem::take(&mut self.interrupt) {
                return Err(io::ErrorKind::Interrupted.into());
            }
            if self.fail_at.is_some_and(|limit| self.cursor >= limit) {
                return Err(io::Error::other("retained test read failure"));
            }
            let count = output
                .len()
                .min(self.chunk)
                .min(self.bytes.len().saturating_sub(self.cursor));
            for (target, source) in output
                .iter_mut()
                .zip(self.bytes.iter().skip(self.cursor))
                .take(count)
            {
                *target = *source;
            }
            self.cursor += count;
            Ok(count)
        }
    }

    #[test]
    fn short_reads_interruption_and_read_failures_keep_exact_eof_semantics() {
        let text = "雪🌸\r\nlast";
        for chunk in [1, 2, 3, CHUNK_BYTES] {
            let mut reader = Chunks {
                bytes: text.as_bytes().to_vec(),
                cursor: 0,
                chunk,
                interrupt: true,
                fail_at: None,
            };
            assert!(reader_matches(&mut reader, text));
        }
        for fail_at in [0, 2, text.len()] {
            let mut reader = Chunks {
                bytes: text.as_bytes().to_vec(),
                cursor: 0,
                chunk: 1,
                interrupt: false,
                fail_at: Some(fail_at),
            };
            assert!(!reader_matches(&mut reader, text));
        }
    }
}
