//! Refuse owned support diagnostics before authored mapping can drop them.

use std::{path::Path, process::Output};

use super::{CliCheckFailure, CliPathResolver, CorsaError, VirtualProject, output_message};

pub(super) fn refuse(output: &Output, project: &VirtualProject) -> Result<(), CliCheckFailure> {
    let Some(target) = project.owned_cli_support_path() else {
        return Ok(());
    };
    refuse_target(output, project.virtual_root(), target)
}

fn refuse_target(output: &Output, root: &Path, target: &Path) -> Result<(), CliCheckFailure> {
    if [&output.stdout, &output.stderr]
        .iter()
        .any(|bytes| std::str::from_utf8(bytes).is_err())
    {
        return Err(CliCheckFailure::Runtime(CorsaError::CorsaExecution {
            exit_code: output.status.code().unwrap_or(1).max(1),
            message: vize_carton::cstr!(
                "undecodable native output in witnessed CLI program:\n{}",
                output_message(output)
            ),
        }));
    }
    if diagnosed(output, root, target) {
        return Err(CliCheckFailure::OwnedSupport(CorsaError::CorsaExecution {
            exit_code: output.status.code().unwrap_or(1).max(1),
            message: vize_carton::cstr!(
                "native diagnostics in witnessed compiler-owned support {}:\n{}",
                target.display(),
                output_message(output)
            ),
        }));
    }
    Ok(())
}

fn diagnosed(output: &Output, root: &Path, target: &Path) -> bool {
    let mut paths = CliPathResolver::new(root);
    for bytes in [&output.stdout, &output.stderr] {
        let Ok(text) = std::str::from_utf8(bytes) else {
            // refuse_target handles undecodable active packets as Runtime,
            // without misattributing them to the support file.
            return false;
        };
        for line in text.lines() {
            let Some((prefix, _)) = line.split_once("): ") else {
                continue;
            };
            let Some((path, position)) = prefix.rsplit_once('(') else {
                continue;
            };
            let Some((line, column)) = position.split_once(',') else {
                continue;
            };
            if line.parse::<u32>().is_ok()
                && column.parse::<u32>().is_ok()
                && paths.resolve(path) == target
            {
                return true;
            }
        }
    }
    false
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    fn output(stdout: &str, stderr: &str) -> Output {
        Output {
            status: std::process::ExitStatus::from_raw(2 << 8),
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn exact_owned_target_is_not_an_authored_filter_or_filename_prefix() {
        let root = Path::new("/private-test/virtual");
        let target = root.join("private-return.d.ts");
        let whole = output(
            "source.vue.ts(1,2): error TS2322: old error\nprivate-return.d.ts(2,3): error TS2304: Cannot find name 'poison'.\n",
            "",
        );
        let before = whole.stdout.clone();
        assert!(diagnosed(&whole, root, &target));
        assert_eq!(whole.stdout, before);
        let error = refuse_target(&whole, root, &target).unwrap_err();
        let CliCheckFailure::OwnedSupport(CorsaError::CorsaExecution { exit_code, message }) =
            error
        else {
            panic!("owned support was not terminal");
        };
        assert_eq!(exit_code, 2);
        assert_eq!(
            message,
            "native diagnostics in witnessed compiler-owned support /private-test/virtual/private-return.d.ts:\nsource.vue.ts(1,2): error TS2322: old error\nprivate-return.d.ts(2,3): error TS2304: Cannot find name 'poison'."
        );
        let mut false_clean = output(
            "private-return.d.ts(1,1): error TS1005: malformed helper\n",
            "",
        );
        false_clean.status = std::process::ExitStatus::from_raw(0);
        let error = refuse_target(&false_clean, root, &target).unwrap_err();
        assert!(matches!(
            error,
            CliCheckFailure::OwnedSupport(CorsaError::CorsaExecution { exit_code: 1, .. })
        ));
        assert!(diagnosed(
            &output(
                "",
                "/private-test/virtual/private-return.d.ts(1,1): error TS1005: malformed helper\n"
            ),
            root,
            &target
        ));
        for other in [
            "other/private-return.d.ts",
            "private-return.d.ts.extra",
            "/authored/private-return.d.ts",
            "source.vue.ts",
        ] {
            assert!(!diagnosed(
                &output(
                    &vize_carton::cstr!("{other}(1,1): error TS2322: preserved authored error\n"),
                    ""
                ),
                root,
                &target
            ));
        }
    }

    #[test]
    fn unmappable_or_undecodable_support_cannot_claim_clean() {
        let root = Path::new("/private-test/virtual");
        let target = root.join("private-return.d.ts");
        let mut raw = output(
            "",
            "private-return.d.ts(1,1): warning TS9999: unsupported helper\n",
        );
        assert!(diagnosed(&raw, root, &target));
        raw.stderr = vec![0xff];
        assert!(matches!(
            refuse_target(&raw, root, &target),
            Err(CliCheckFailure::Runtime(_))
        ));
        assert!(!diagnosed(&raw, root, &target));
        assert_eq!(raw.stderr, [0xff]);
    }
}
