//! Independent complete refusal oracles for inert CLI subprocess fixtures.
#![expect(clippy::disallowed_types, reason = "owned process transport text")]
#![expect(clippy::disallowed_macros, reason = "owned process transport text")]
use std::{
    path::Path,
    process::{Command, Output},
    sync::LazyLock,
};

pub const NODE_OPTIONS: &str = "--stack-trace-limit=0";
const TOKEN: &str = "[TypeError: owned-refusal-probe]";
static NODE: LazyLock<(Output, String)> = LazyLock::new(|| {
    let output = Command::new("node").env("NODE_OPTIONS", NODE_OPTIONS)
        .args(["--input-type=module", "-e", "process.stdout.write(process.version); await Promise.resolve(); throw new TypeError('owned-refusal-probe');"])
        .output().expect("independent Node refusal probe");
    probe(output)
});

fn probe(output: Output) -> (Output, String) {
    eprintln!("independent Node refusal probe packet: {output:?}");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let version = std::str::from_utf8(&output.stdout).expect("Node version");
    let components = version
        .strip_prefix('v')
        .expect("Node version prefix")
        .split('.')
        .collect::<Vec<_>>();
    assert_eq!(components.len(), 3, "{output:?}");
    assert!(
        components
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())),
        "{output:?}"
    );
    let stderr = std::str::from_utf8(&output.stderr).expect("Node refusal text");
    assert_eq!(stderr.matches(TOKEN).count(), 1, "{output:?}");
    let version = version.to_owned();
    (output, version)
}

fn assert_packet(output: &Output, code: i32, expected: &[u8]) {
    eprintln!("native config refusal process packet: {output:?}");
    assert_eq!(
        (
            output.status.code(),
            output.stdout.as_slice(),
            output.stderr.as_slice()
        ),
        (Some(code), b"".as_slice(), expected)
    );
}

pub fn assert_error(
    output: &Output,
    selected: &Path,
    directory: &Path,
    exception: &str,
    code: i32,
) {
    assert_error_with_suffix(output, selected, directory, exception, code, "");
}

/// Preserve the complete independently observed Node error before owned CLI guidance.
pub fn assert_error_with_suffix(
    output: &Output,
    selected: &Path,
    directory: &Path,
    exception: &str,
    code: i32,
    suffix: &str,
) {
    // The independent probe owns Node's uncaught-error renderer, without loading
    // any configuration. Substitute its unique exception token on the expected
    // side only; every actual process byte remains intact.
    let body = if exception == "SyntaxError: Unexpected token ';'" {
        let source = url::Url::from_file_path(directory.join("vite.config.mjs")).unwrap();
        format!(
            "{source}:1\nexport default {{ vue: {{ }};\n                         ^\n\n[{exception}]\n\nNode.js {}\n",
            NODE.1
        )
    } else {
        std::str::from_utf8(&NODE.0.stderr)
            .unwrap()
            .replace(TOKEN, &format!("[{exception}]"))
    };
    let expected = format!(
        "\x1b[31mError:\x1b[0m failed to parse {}: node failed to load config: {body}{suffix}",
        selected.display()
    );
    assert_packet(output, code, expected.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    fn output(code: i32, stdout: &[u8], stderr: &[u8]) -> Output {
        #[cfg(unix)]
        use std::os::unix::process::ExitStatusExt;
        #[cfg(windows)]
        use std::os::windows::process::ExitStatusExt;
        #[cfg(unix)]
        let status = std::process::ExitStatus::from_raw(code << 8);
        #[cfg(windows)]
        let status = std::process::ExitStatus::from_raw(code as u32);
        Output {
            status,
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        }
    }
    #[test]
    fn whole_packets_reject_foreign_messages_trailing_bytes_and_extra_errors() {
        assert_packet(
            &output(2, b"", b"owned complete error\n"),
            2,
            b"owned complete error\n",
        );
        for altered in [
            output(1, b"", b"owned complete error\n"),
            output(2, b"foreign stdout", b"owned complete error\n"),
            output(2, b"", b"foreign message\nowned complete error\n"),
            output(2, b"", b"owned complete error\ntrailing bytes"),
            output(2, b"", b"owned complete error\n[TypeError: extra error]\n"),
        ] {
            assert!(
                std::panic::catch_unwind(|| assert_packet(&altered, 2, b"owned complete error\n"))
                    .is_err()
            );
        }
    }
    #[test]
    fn failed_or_ambiguous_probes_cannot_supply_an_expected_envelope() {
        let token = TOKEN.as_bytes();
        let valid = probe(output(1, b"v24.14.0", token));
        assert_eq!(
            (
                valid.0.status.code(),
                valid.0.stdout.as_slice(),
                valid.0.stderr.as_slice(),
                valid.1.as_str()
            ),
            (Some(1), b"v24.14.0".as_slice(), token, "v24.14.0")
        );
        for altered in [
            output(0, b"v24.14.0", token),
            output(1, b"foreign version", token),
            output(1, b"v24.14.0", b""),
            output(1, b"v24.14.0", format!("{TOKEN}{TOKEN}").as_bytes()),
        ] {
            assert!(std::panic::catch_unwind(|| probe(altered)).is_err());
        }
    }
}
