"""Independent default Canon and isolated Vize feature compilation contracts."""

import json
from pathlib import Path
import tomllib

MINIMAL_STORY_TEST = "commands::musea::migrate::tests::story_collection_is_available_without_formatter_features"


def verify_minimal_story_test(output):
    if f"test {MINIMAL_STORY_TEST} ... ok" not in output or "test result: ok. 1 passed; 0 failed;" not in output:
        raise ValueError("minimal CLI story collection must execute its one exact regression")


def source_manifest(dependency, directory, defaults=True, features=()):
    manifest = '[package]\nname = "source-feature-consumer"\nversion = "0.0.0"\nedition = "2024"\n[workspace]\n'
    manifest += '[dependencies.' + dependency + ']\npath = ' + json.dumps(str(directory)) + '\n'
    # Canon's native feature must arrive through its actual defaults. CLI
    # variants explicitly opt out where required rather than unifying features.
    if dependency == "vize":
        manifest += 'default-features = ' + str(defaults).lower() + '\nfeatures = ' + json.dumps(list(features)) + '\n'
    return manifest


def package_features(metadata, name, manifest):
    package = next(package for package in metadata["packages"] if package["name"] == name)
    if Path(package["manifest_path"]).resolve() != manifest.resolve() or package["source"] is not None:
        raise ValueError("source consumer resolved an unexpected " + name + " provider")
    return next(node["features"] for node in metadata["resolve"]["nodes"] if node["id"] == package["id"])


def cli_source(common, features):
    source = common
    if "glyph" in features:
        source += b'\npub fn glyph_options_layout() -> usize { std::mem::size_of::<vize::glyph::FormatOptions>() }\n'
    if "maestro" in features:
        source += b'\npub fn maestro_entrypoint() -> fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> { vize::maestro::serve_blocking }\n'
    if "profiling" in features:
        source += b'\npub fn profiling_layout() -> usize { std::mem::size_of::<vize::DoctorTuiBenchmark<\'static>>() }\n'
    return source


def qualify_source_consumers(runner, version, providers, verify_graph):
    root = runner.root
    canon_manifest = root / "crates/vize_canon/Cargo.toml"
    fixture_root = root / "tests/_fixtures/ci"
    directory = runner.consumer("canon-default-native", source_manifest("vize_canon", canon_manifest.parent),
                                (fixture_root / "checked-canon-consumer/lib.rs").read_bytes())
    metadata = runner.metadata("canon-default-native-metadata", directory)
    canon_features = package_features(metadata, "vize_canon", canon_manifest)
    if not {"default", "native"} <= set(canon_features):
        raise ValueError("independent Canon consumer must activate native through defaults")
    canon = {"features": canon_features, "packages": verify_graph(metadata, version, providers, False)}
    runner.build_once("canon-source", "default-native", directory)

    cli = []
    cli_manifest = root / "crates/vize/Cargo.toml"
    all_features = sorted(tomllib.loads(cli_manifest.read_text())["features"])
    common = (fixture_root / "checked-vize-consumer/lib.rs").read_bytes()
    # cargo metadata --all-features enables all workspace members. External
    # consumers select only vize's requested features, matching actual users.
    for label, defaults, requested in [("default", True, []), ("minimal", False, []),
                                       ("glyph", False, ["glyph"]), ("maestro", False, ["maestro"]),
                                       ("all", False, all_features)]:
        activated = set(requested) | ({"glyph", "maestro"} if defaults or "default" in requested else set())
        directory = runner.consumer("cli-" + label, source_manifest("vize", cli_manifest.parent, defaults, requested),
                                    cli_source(common, activated))
        metadata = runner.metadata("cli-" + label + "-metadata", directory)
        features = package_features(metadata, "vize", cli_manifest)
        if not set(requested) <= set(features) or any((feature in features) != (feature in activated) for feature in ["glyph", "maestro"]):
            raise ValueError("CLI optional feature parity changed")
        canon_native = package_features(metadata, "vize_canon", canon_manifest)
        if "native" not in canon_native:
            raise ValueError("CLI consumer lost its native Canon provider")
        cli.append({"mode": label, "features": features, "canonFeatures": canon_native,
                    "packages": verify_graph(metadata, version, providers, False)})
        runner.build_once("cli-source", label, directory)
    output = runner.command("cli-minimal-story-collection-test", ["cargo", "test", "--locked", "--offline", "--profile", "ci",
        "-p", "vize", "--no-default-features", "--lib", MINIMAL_STORY_TEST, "--", "--exact", "--nocapture"])
    verify_minimal_story_test(output)
    return {"canon": canon, "cli": cli, "minimalStoryCollectionTestPassed": True}
