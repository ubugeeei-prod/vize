"""Compile one public guard-type snippet against normal, old-control and new libraries."""

import hashlib
import json
import os
from pathlib import Path
import subprocess


def invoke(command, stdout_path, stderr_path):
    with stdout_path.open("w") as output, stderr_path.open("w") as error:
        result = subprocess.run(command, stdout=output, stderr=error, check=False)
    return result.returncode


def library_artifact(log):
    matches = []
    for line in log.read_text().splitlines():
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        if (
            row.get("reason") == "compiler-artifact"
            and row["target"]["name"] == "vize_l0"
            and row["target"]["kind"] == ["lib"]
        ):
            matches.append(row)
    assert len(matches) == 1, (log, len(matches))
    artifact = matches[0]
    libraries = [Path(name) for name in artifact["filenames"] if name.endswith(".rlib")]
    assert len(libraries) == 1 and libraries[0].is_file()
    return artifact, libraries[0]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compile_probe(role, artifact, library, snippet, evidence):
    dependency_dirs = {Path(name).parent for name in artifact["filenames"] if Path(name).parent.name == "deps"}
    assert len(dependency_dirs) == 1, artifact
    dependencies = next(iter(dependency_dirs))
    assert dependencies.is_dir()
    destination = evidence / ("send-" + role)
    destination.mkdir()
    stdout = evidence / ("send-" + role + ".stdout")
    stderr = evidence / ("send-" + role + ".jsonl")
    code = invoke(
        [
            "rustc", "+1.99.1", "--edition=2024", "--crate-name", "guard_send_probe",
            "--emit=metadata", "--error-format=json", "--out-dir", str(destination),
            "--extern", "vize_l0=" + str(library), "-L", "dependency=" + str(dependencies),
            str(snippet),
        ],
        stdout,
        stderr,
    )
    diagnostics = []
    for line in stderr.read_text().splitlines():
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        if row.get("level") == "error" and row.get("code") is not None:
            diagnostics.append({"code": row["code"]["code"], "message": row["message"]})
    return {
        "returnCode": code, "diagnostics": diagnostics, "rlib": str(library),
        "rlibSha256": digest(library), "dependencyDirectory": str(dependencies), "artifact": artifact,
    }


def main():
    root = Path.cwd()
    evidence = root / "evidence"
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    normal, normal_library = library_artifact(evidence / "normal-law.log")
    instrumented, instrumented_library = library_artifact(evidence / "instrumented-law.log")
    assert "micro-profiling" not in normal["features"]
    assert "micro-profiling" in instrumented["features"]
    assert normal_library != instrumented_library
    for artifact in (normal, instrumented):
        assert Path(artifact["manifest_path"]) == root / "davinci/vize_l0/Cargo.toml"
        assert Path(artifact["target"]["src_path"]) == root / "davinci/vize_l0/src/lib.rs"

    control = Path(os.environ["RUNNER_TEMP"]) / (
        "old-guard-" + os.environ["GITHUB_RUN_ID"] + "-" + os.environ["GITHUB_RUN_ATTEMPT"]
    )
    assert not control.exists()
    subprocess.run(["git", "worktree", "add", "--detach", str(control), head], check=True)
    core_relative = Path("davinci/vize_l0/src/profiler/core.rs")
    core = control / core_relative
    original = core.read_text()
    field = (
        '    #[cfg(feature = "micro-profiling")]\n'
        "    _same_thread: std::marker::PhantomData<std::rc::Rc<()>>,\n"
    )
    initializer = (
        '            #[cfg(feature = "micro-profiling")]\n'
        "            _same_thread: std::marker::PhantomData,\n"
    )
    assert original.count(field) == original.count(initializer) == 1
    previous = original.replace(field, "").replace(initializer, "")
    core.write_text(previous)
    with (evidence / "old-guard-mutation.patch").open("w") as output:
        subprocess.run(["git", "-C", str(control), "diff", "--", str(core_relative)], stdout=output, check=True)
    old_log = evidence / "old-guard-build.jsonl"
    old_error = evidence / "old-guard-build.stderr"
    old_code = invoke(
        [
            "cargo", "+1.99.1", "build", "--locked", "--profile", "ci", "-p", "vize_l0",
            "--lib", "--no-default-features", "--features", ",".join(instrumented["features"]),
            "--manifest-path", str(control / "Cargo.toml"),
            "--target-dir", str(root / "target/guard-old-control"), "--message-format", "json",
        ],
        old_log,
        old_error,
    )
    assert old_code == 0
    old, old_library = library_artifact(old_log)
    assert old["features"] == instrumented["features"]
    assert Path(old["manifest_path"]) == control / "davinci/vize_l0/Cargo.toml"

    snippet = evidence / "guard-send-snippet.rs"
    snippet.write_text(
        "use vize_l0::profiler::ProfileGuard;\n"
        "fn require_send<T: Send>() {}\n"
        "fn main() { require_send::<ProfileGuard>(); }\n"
    )
    roles = {
        "normal": compile_probe("normal", normal, normal_library, snippet, evidence),
        "old-profiling-control": compile_probe("old", old, old_library, snippet, evidence),
        "instrumented": compile_probe("instrumented", instrumented, instrumented_library, snippet, evidence),
    }
    receipt = {
        "source": head, "snippetSha256": digest(snippet),
        "originalCoreSha256": hashlib.sha256(original.encode()).hexdigest(),
        "mutatedCoreSha256": hashlib.sha256(previous.encode()).hexdigest(),
        "mutation": "Remove only the added zero-sized field and initializer; keep profiling features equal",
        "roles": roles,
    }
    (evidence / "guard-send-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    for role in ("normal", "old-profiling-control"):
        assert roles[role]["returnCode"] == 0 and roles[role]["diagnostics"] == [], roles[role]
    rejected = roles["instrumented"]
    assert rejected["returnCode"] != 0
    assert [row["code"] for row in rejected["diagnostics"]] == ["E0277"], rejected
    assert "Rc" in rejected["diagnostics"][0]["message"]
    assert "cannot be sent" in rejected["diagnostics"][0]["message"]
    print("guard-send-mutation=" + json.dumps(receipt, separators=(",", ":")))


if __name__ == "__main__":
    main()
