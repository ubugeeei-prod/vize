#!/usr/bin/env python3
"""Read-only complete-tree audit of an official release candidate and its source cut.

Python 3.12+. Never invokes Cargo, Moon, vp, installs, a provider, or a release.
The output is an attestation of a specific tree relation, not publication proof.
"""
import argparse
import copy
import hashlib
import json
import re
import subprocess
import tomllib
from pathlib import Path

SCHEMA = "vize.release.source-cut-version-bridge"
RECIPE_PATHS = [
    "tools/moon/cmd/release/apply_versions.mbt",
    "tools/moon/cmd/release/preparation.mbt",
    "tools/moon/cmd/release/readmes.mbt",
    "tools/moon/cmd/release/versions.mbt",
    "tools/moon/cmd/release/native_catalog.mbt",
    "tools/support/release/pr_start.rs",
]
GUEST_LOCKS = [
    "davinci/vize_extension_host/tests/guests/expression-echo/Cargo.lock",
    "davinci/vize_extension_host/tests/guests/output-echo/Cargo.lock",
    "davinci/vize_extension_host/tests/guests/typed-expression-echo/Cargo.lock",
    "examples/volt-target/Cargo.lock",
]


def sha(data):
    return hashlib.sha256(data).hexdigest()


def compact(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":")).encode()


def first_version(content, old, new, json_field=False):
    lines = content.split("\n")
    for i, line in enumerate(lines):
        text = line.strip()
        match = text.startswith('"version"') if json_field else (
            text.startswith('version = "') and text.endswith('"'))
        if match and old in line:
            lines[i] = line.replace(old, new, 1)
            break
    return "\n".join(lines)


def workspace_manifest(content, old, new, own_names):
    section = ""
    out = []
    for line in content.split("\n"):
        text = line.strip()
        if text.startswith("[") and text.endswith("]"):
            section = text
        if section == "[workspace.package]" and text == f'version = "{old}"':
            line = line.replace(old, new, 1)
        elif section == "[workspace.dependencies]" and (
            text.startswith("vize_") or 'package = "vize_' in line
        ) and f'version = "={old}"' in line:
            entry = tomllib.loads(line)
            dep_key, dep = next(iter(entry.items()))
            name = dep.get("package", dep_key)
            assert name in own_names and "path" in dep, ("not an owned path pin", name)
            line = line.replace(f'version = "={old}"', f'version = "={new}"')
        out.append(line)
    return "\n".join(out)


def native_catalog(content, old, new, workspace):
    out, in_block = [], False
    for line in content.split("\n"):
        if workspace and line.startswith('  - "@vizejs/native-') and old in line:
            if new not in line:
                line = line.replace(old, old + " || " + new, 1)
            out.append(line)
            continue
        if line == "  native-binaries:":
            in_block = True
            out.append(line)
            continue
        if in_block:
            leaves = line != "" and not line.startswith("    ")
            if workspace:
                leaves = leaves and not line.startswith("  #")
            if leaves:
                in_block = False
            elif workspace and line.startswith('    "@vizejs/native-') and old in line:
                line = line.replace(old, new, 1)
            elif not workspace and (
                line.startswith("      specifier: ") or line.startswith("      version: ")
            ) and old in line:
                line = line.replace(old, new, 1)
        out.append(line)
    return "\n".join(out)


def readme(content, old, new):
    out, benchmark = [], False
    for line in content.split("\n"):
        if "<!-- benchmark:readme:start -->" in line:
            benchmark = True
        out.append(line if benchmark else line.replace(old, new))
        if "<!-- benchmark:readme:end -->" in line:
            benchmark = False
    return "\n".join(out)


def cargo_lock(content, old, new, names):
    """Only version fields of source-less in-tree packages; all foreign bytes exact."""
    blocks = re.split(r"(?=^\[\[package\]\]$)", content, flags=re.M)
    out = []
    for block in blocks:
        if not block.startswith("[[package]]"):
            out.append(block)
            continue
        package = tomllib.loads(block)["package"][0]
        lines = block.split("\n")
        if package["name"] in names and "source" not in package and package["version"] == old:
            index = lines.index(f'version = "{old}"')
            lines[index] = f'version = "{new}"'
        for i, line in enumerate(lines):
            match = re.fullmatch(r'(?P<start>\s*\")(?P<name>[^" ]+) ' + re.escape(old) + r'(?P<end>\",?)', line)
            if match and match["name"] in names:
                lines[i] = match["start"] + match["name"] + " " + new + match["end"]
        out.append("\n".join(lines))
    return "".join(out)


def audit(repo, source_cut, release_head, old, new, recipe_path, pr=None, offline=False):
    def git(*args):
        return subprocess.check_output(["git", *args], cwd=repo)

    def blob(rev, path):
        return git("show", rev + ":" + path)

    def tree(rev):
        rows = {}
        for raw in git("ls-tree", "-r", "-z", "--full-tree", rev).split(b"\0"):
            if raw:
                meta, path = raw.split(b"\t", 1)
                mode, kind, oid = meta.decode().split(" ")
                rows[path.decode()] = {"mode": mode, "type": kind, "oid": oid}
        return rows

    assert re.fullmatch(r"[0-9a-f]{40}", source_cut)
    assert re.fullmatch(r"[0-9a-f]{40}", release_head)
    old_parts = tuple(map(int, old.split(".")))
    assert tuple(map(int, new.split("."))) == (old_parts[0], old_parts[1] + 1, 0), "not the selected minor bump"
    parent_line = git("rev-list", "--parents", "-n", "1", release_head).decode().split()
    assert parent_line == [release_head, source_cut], "candidate is not the sole child of the source cut"
    assert git("show", "-s", "--format=%B", release_head).decode().strip() == f"chore: release v{new}"
    before, after = tree(source_cut), tree(release_head)
    assert before.keys() == after.keys(), "tracked path additions/removals are not in the official recipe"
    root = tomllib.loads(blob(source_cut, "Cargo.toml").decode())
    assert root["workspace"]["package"]["version"] == old
    assert tomllib.loads(blob(release_head, "Cargo.toml").decode())["workspace"]["package"]["version"] == new
    recipe = json.loads(Path(recipe_path).read_bytes())
    assert recipe["schema"] == "vize.release.official-version-bridge-recipe"
    for row in recipe["sources"]:
        assert sha(blob(source_cut, row["path"])) == row["sha256"], ("release recipe needs a new review", row["path"])

    manifests, own_names = {}, set()
    for path in before:
        if path.endswith("Cargo.toml"):
            manifest = tomllib.loads(blob(source_cut, path).decode())
            manifests[path] = manifest
            package = manifest.get("package", {})
            value = package.get("version")
            if value == old or isinstance(value, dict) and value.get("workspace") is True:
                own_names.add(package["name"])
    # Every owned identity has a tracked source manifest; registry packages with
    # a coincidentally equal version are never admitted by name prefix alone.
    npm_paths = set()
    readme_paths = {"README.md"} if "README.md" in before else set()
    for path in before:
        parts = path.split("/")
        if len(parts) == 3 and parts[0] == "npm" and parts[-1] == "package.json":
            npm_paths.add(path)
        elif len(parts) == 4 and parts[0] == "npm" and parts[-1] == "package.json" and (
            "/".join(parts[:2]) + "/package.json" not in before
        ):
            if json.loads(blob(source_cut, path)).get("private") is not True:
                npm_paths.add(path)
    for path in ("editors/vscode/package.json", "editors/vscode-art/package.json"):
        if path in before:
            npm_paths.add(path)
    for path in before:
        parts = path.split("/")
        if len(parts) == 3 and parts[0] == "npm" and parts[-1] == "README.md":
            readme_paths.add(path)
    for path in npm_paths:
        if path.startswith("npm/"):
            readme_path = path.removesuffix("package.json") + "README.md"
            if readme_path in before:
                readme_paths.add(readme_path)

    locks = {"Cargo.lock", "editors/zed/Cargo.lock", *GUEST_LOCKS} & before.keys()
    zed_manifests = {"editors/zed/Cargo.toml", "editors/zed/extension.toml"} & before.keys()
    allowed = {"Cargo.toml", "pnpm-lock.yaml", "pnpm-workspace.yaml"} | locks | zed_manifests | npm_paths | readme_paths
    changed, all_paths = [], []
    for path in sorted(before):
        a, b = before[path], after[path]
        all_paths.append({"path": path, "before": a, "after": b})
        if a == b:
            continue
        assert a["mode"] == b["mode"] and a["type"] == b["type"] == "blob", ("mode/type changed", path)
        assert path in allowed, ("unapproved implementation/changelog/fixture/provider/dependency path", path)
        original, actual = blob(source_cut, path), blob(release_head, path)
        content = original.decode()
        if path == "Cargo.toml":
            expected = workspace_manifest(content, old, new, own_names)
            kind = "owned workspace package version and exact in-tree crate pins"
        elif path in locks:
            expected = cargo_lock(content, old, new, own_names)
            kind = "source-less owned crate versions only; all foreign package records and bytes unchanged"
        elif path in npm_paths:
            expected = first_version(content, old, new, json_field=True)
            d = json.loads(content)
            expected_fields = copy.deepcopy(d)
            if expected != content:
                assert d["version"] == old
                expected_fields["version"] = new
            assert json.loads(expected) == expected_fields, path
            kind = "owned package top-level version only; dependency fields unchanged"
        elif path in zed_manifests:
            expected = first_version(content, old, new)
            kind = "Zed first release version only"
        elif path == "pnpm-workspace.yaml":
            expected = native_catalog(content, old, new, workspace=True)
            kind = "own native-binaries catalog and own minimum-release-age additions only"
        elif path == "pnpm-lock.yaml":
            expected = native_catalog(content, old, new, workspace=False)
            kind = "own native-binaries catalog specifier/version only; all dependency graph bytes unchanged"
        elif path in readme_paths:
            expected = readme(content, old, new)
            kind = "official README release references; marked benchmark sections unchanged"
        else:
            raise AssertionError(path)
        assert expected.encode() == actual, ("not byte-exact official scoped version transformation", path)
        changed.append({"path": path, "before": a, "after": b,
                        "beforeSha256": sha(original), "afterSha256": sha(actual), "classification": kind})
    assert changed, "empty candidate"
    assert recipe["trackedChangelogMutationsAllowed"] is False
    result = {
        "schema": SCHEMA, "version": 1,
        "status": "OFFLINE_HISTORICAL_RELATION_ONLY" if offline else "CANDIDATE_TREE_RELATION_VERIFIED",
        "sourceCut": {"sha": source_cut, "tree": git("rev-parse", source_cut + "^{tree}").decode().strip(), "version": old},
        "releaseHead": {"sha": release_head, "tree": git("rev-parse", release_head + "^{tree}").decode().strip(), "version": new, "parents": [source_cut]},
        "recipeSha256": sha(Path(recipe_path).read_bytes()), "recipeSources": recipe["sources"],
        "allPathManifestSha256": sha(compact(all_paths)), "allPathManifest": all_paths,
        "changedPathCount": len(changed), "unchangedPathCount": len(before) - len(changed),
        "changedPaths": changed, "allowedCandidatePaths": sorted(allowed),
        "allNonRecipeEntriesByteIdentical": True, "thirdPartyDependencyContentUnchanged": True,
        "trackedChangelogChanged": False, "generatedReleaseNotes": "GitHub Release metadata only; not a tracked blob allowance",
        "implementationFixtureHelperCapsProviderContentUnchanged": True,
        "limits": ["This verifies the candidate/source relation, not publication, performance, installed behavior or native-default completion.",
                   "Public authority must separately authenticate actual tagHead==releaseHead; sourceCut and tagHead remain distinct.",
                   "Only initialize.serverInfo.version may be adjusted from sourceCut.version to tagHead.version in the explicitly reviewed public comparator; all other authored objects remain whole."]}
    result["authorityProjection"] = {
        "schema": SCHEMA, "version": 1,
        "sourceCut": result["sourceCut"],
        "tagHead": {k: result["releaseHead"][k] for k in ("sha", "tree", "version")},
        "recipeSha256": result["recipeSha256"],
        "allPathManifestSha256": result["allPathManifestSha256"],
        "changedPathManifestSha256": sha(compact(changed)),
        "changedPathCount": len(changed),
        "unchangedPathCount": len(before) - len(changed),
    }
    result["authorityProjectionSha256"] = sha(compact(result["authorityProjection"]))
    if not offline:
        assert pr and int(pr) > 7811
        def api(path):
            raw = subprocess.check_output(["gh", "api", "repos/ubugeeei-prod/vize/" + path])
            return {"rawSha256": sha(raw), "body": json.loads(raw)}
        source_api = api("commits/" + source_cut)
        head_api = api("commits/" + release_head)
        pull = api("pulls/" + str(pr))
        assert source_api["body"]["commit"]["verification"]["verified"] is True
        assert source_api["body"]["commit"]["verification"]["reason"] == "valid"
        assert source_api["body"]["sha"] == source_cut and head_api["body"]["sha"] == release_head
        assert len(head_api["body"]["parents"]) == 1
        assert head_api["body"]["parents"][0]["sha"] == source_cut
        assert source_api["body"]["commit"]["tree"]["sha"] == result["sourceCut"]["tree"]
        assert head_api["body"]["commit"]["tree"]["sha"] == result["releaseHead"]["tree"]
        assert pull["body"]["head"]["sha"] == release_head
        assert pull["body"]["head"]["ref"] == "release/v" + new
        assert pull["body"]["base"]["ref"] == "main"
        assert pull["body"]["head"]["repo"]["full_name"] == "ubugeeei-prod/vize"
        assert pull["body"]["base"]["repo"]["full_name"] == "ubugeeei-prod/vize"
        assert f"<!-- vize-release-base-version: {old} -->" in pull["body"]["body"]
        assert "<!-- vize-release-bump: minor -->" in pull["body"]["body"]
        result["officialApi"] = {"sourceCut": source_api, "releaseHead": head_api, "releasePr": pull}
        result["publicationState"] = "separate actual tag and publication proof required"
    return result


def main():
    p = argparse.ArgumentParser()
    p.add_argument("repo")
    p.add_argument("source_cut")
    p.add_argument("release_head")
    p.add_argument("old_version")
    p.add_argument("new_version")
    p.add_argument("recipe")
    p.add_argument("output")
    p.add_argument("--pr", type=int)
    p.add_argument("--offline-historical", action="store_true")
    args = p.parse_args()
    result = audit(args.repo, args.source_cut, args.release_head, args.old_version,
                   args.new_version, args.recipe, args.pr, args.offline_historical)
    output = Path(args.output)
    output.write_bytes(json.dumps(result, ensure_ascii=False, indent=2).encode() + b"\n")
    print(json.dumps({"path": str(output), "sha256": sha(output.read_bytes()),
                      "status": result["status"], "changed": result["changedPathCount"],
                      "unchanged": result["unchangedPathCount"]}))


if __name__ == "__main__":
    main()
