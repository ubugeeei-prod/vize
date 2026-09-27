use std::{
    io::Write,
    process::{Command, Stdio},
};

use super::{
    InspectorArgs, OFFICIAL_COMPILER_NODE_SCRIPT, OfficialCompareInput, OfficialCompareInputFile,
    OfficialCompareOutput, VizeCompilerRun, compare_error, curator_inspector, dev_module_root,
};

pub(super) fn run_official_compiler_for_compare(
    files: &[curator_inspector::InspectorSourceFile],
    vize_runs: &[VizeCompilerRun],
    args: &InspectorArgs,
) -> OfficialCompareOutput {
    let input = OfficialCompareInput {
        target: args.target.as_str(),
        module_root: dev_module_root(),
        files: files
            .iter()
            .zip(vize_runs)
            .map(|(file, vize)| OfficialCompareInputFile {
                path: file.path.clone(),
                source: file.source.clone(),
                vize_code: vize.code.clone(),
            })
            .collect(),
    };
    let input_json = serde_json::to_vec(&input).unwrap_or_else(|error| {
        eprintln!("Failed to serialize official compiler input: {error}");
        std::process::exit(1);
    });
    let node = std::env::var_os("VIZE_INSPECTOR_NODE").unwrap_or_else(|| "node".into());
    let mut child = Command::new(&node)
        .arg("--input-type=module")
        .arg("-e")
        .arg(OFFICIAL_COMPILER_NODE_SCRIPT)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| {
            let node = node.to_string_lossy();
            eprintln!(
                "--format compare requires a local Node.js runtime and @vue/compiler-sfc. Failed to start {node}: {error}"
            );
            std::process::exit(1);
        });

    let write_result = child
        .stdin
        .take()
        .ok_or_else(|| std::io::Error::other("the Node.js process has no piped stdin"))
        .and_then(|mut stdin| stdin.write_all(&input_json));

    let output = child.wait_with_output().unwrap_or_else(|error| {
        eprintln!("Failed to wait for official compiler process: {error}");
        std::process::exit(1);
    });

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<stderr is not valid UTF-8>");
        eprintln!("{}", compare_error::official_compiler_error_message(stderr));
        std::process::exit(output.status.code().unwrap_or(1));
    }

    write_result.unwrap_or_else(|error| {
        eprintln!("Failed to write inspector compare input to Node.js: {error}");
        std::process::exit(1);
    });

    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<stdout is not valid UTF-8>");
        eprintln!("Failed to parse official compiler output: {error}\n{stdout}");
        std::process::exit(1);
    })
}
