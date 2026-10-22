use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use super::{TypeAwareFailure, with_corsa_session};
use crate::{LintPreset, Linter};
use vize_l0::{String, ToCompactString, cstr};

#[test]
fn two_tsconfigs_keep_two_sessions() {
    let Some(corsa_path) = corsa_path() else {
        return;
    };
    let root = case_dir("two-tsconfigs");
    let _ = std::fs::remove_dir_all(&root);
    let alpha = root.join("alpha");
    let beta = root.join("beta");
    write_package(&alpha);
    write_package(&beta);

    let linter = Linter::with_preset(LintPreset::Opinionated).with_corsa_path(Some(corsa_path));
    let gate = (Mutex::new(0usize), Condvar::new());
    let alpha_file = alpha.join("src").join("A.vue");
    let beta_file = beta.join("src").join("B.vue");
    let overlapped = std::thread::scope(|scope| {
        let left = scope.spawn(|| enter_session(&linter, &alpha_file, &gate));
        let right = scope.spawn(|| enter_session(&linter, &beta_file, &gate));
        (left.join(), right.join())
    });
    overlapped
        .0
        .expect("alpha thread")
        .expect("alpha session stayed open beside beta");
    overlapped
        .1
        .expect("beta thread")
        .expect("beta session stayed open beside alpha");
    assert_eq!(
        linter.native_corsa.len(),
        2,
        "the second tsconfig must not close the first session"
    );

    Linter::finish_type_aware_lint(std::slice::from_ref(&linter), &[]);
    let _ = std::fs::remove_dir_all(&root);
}

fn enter_session(
    linter: &Linter,
    file: &Path,
    gate: &(Mutex<usize>, Condvar),
) -> Result<(), String> {
    let filename = file
        .to_str()
        .ok_or_else(|| "utf-8 path".to_compact_string())?;
    with_corsa_session(linter, filename, |_| {
        if !wait_for_peer(gate) {
            return Err("sessions did not overlap".to_compact_string());
        }
        Ok(())
    })
    .map_err(failure_message)
}

fn wait_for_peer(gate: &(Mutex<usize>, Condvar)) -> bool {
    let (lock, condvar) = gate;
    let Ok(mut count) = lock.lock() else {
        return false;
    };
    *count += 1;
    condvar.notify_all();
    let start = Instant::now();
    while *count < 2 {
        let (guard, timeout) = match condvar.wait_timeout(count, Duration::from_secs(30)) {
            Ok(waited) => waited,
            Err(_) => return false,
        };
        count = guard;
        if timeout.timed_out() && *count < 2 {
            return false;
        }
        if start.elapsed() > Duration::from_secs(45) {
            return false;
        }
    }
    true
}

fn failure_message(failure: TypeAwareFailure) -> String {
    match failure {
        TypeAwareFailure::Runtime(message)
        | TypeAwareFailure::ProjectConfiguration { message, .. } => message,
    }
}

fn write_package(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("package.json"), "{}\n").unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        "{\n  \"compilerOptions\": { \"strict\": true, \"target\": \"ES2022\" },\n  \"include\": [\"src\"]\n}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src").join("marker.ts"),
        "export const marker = 1;\n",
    )
    .unwrap();
}

fn case_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("vize-tests")
        .join(cstr!("patina-corsa-pool-{name}-{}", std::process::id()).as_str())
}

fn corsa_path() -> Option<PathBuf> {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .to_path_buf();
    let stable = repo
        .join("node_modules")
        .join("@typescript")
        .join(
            cstr!(
                "typescript-{}",
                vize_carton::corsa_resolver::platform_suffix()
            )
            .as_str(),
        )
        .join("lib")
        .join(cstr!("tsc{}", std::env::consts::EXE_SUFFIX).as_str());
    stable.is_file().then_some(stable)
}
