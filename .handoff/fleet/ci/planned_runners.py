"""Give skip-only job paths a hosted runner via a planner-conditional runs-on."""
import sys

root = sys.argv[1]
busy = "blacksmith-32vcpu-ubuntu-2404"
jobs = {
    ".github/workflows/check.yml": {
        "security-audit": "needs.iteration-plan.outputs.dependencies == 'true'",
        "node-engine-compat": "needs.iteration-plan.outputs.engines == 'true'",
        "check-vize-apps": "needs.iteration-plan.outputs.apps == 'true'",
    },
    ".github/workflows/pr-source-checks.yml": {
        "wasm-release-check": "contains(fromJSON(needs.pr-source-plan.outputs.rust-plan).packages, 'vize_vitrine')",
        "ci-selection-tests": "needs.pr-source-plan.outputs.tooling == 'true'",
        "pr-native-runtime": "matrix.lane != 'none'",
        "pr-playground-test": "needs.pr-source-plan.outputs.playground == 'true'",
    },
}
for file, selected in jobs.items():
    path = f"{root}/{file}"
    text = open(path).read()
    for job, condition in selected.items():
        start = text.index(f"\n  {job}:\n")
        end = text.index("\n  ", start + 3)
        # runs-on must be inside this job's header block
        at = text.index(f"    runs-on: {busy}\n", start)
        nxt = text.find("\n  ", start + 3)
        # find next job start (two-space indent key)
        import re
        m = re.compile(r"\n  [a-z][a-z0-9-]*:\n").search(text, start + 3)
        assert m is None or at < m.start(), (file, job)
        text = (
            text[:at]
            + f"    runs-on: ${{{{ {condition} && '{busy}' || 'ubuntu-24.04' }}}}\n"
            + text[at + len(f"    runs-on: {busy}\n"):]
        )
        print("planned runner", file, job)
    open(path, "w").write(text)
