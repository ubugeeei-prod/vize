import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

export function captureRustLaws({
  root,
  rustBuild,
  requiredLaws,
  run,
  successful,
  messages,
  packagePath,
  sha256,
  write,
  failures,
  evidence,
  selectedIntegrationTargets,
}: any) {
  const rust = run("rust-laws-build", "cargo", rustBuild);
  const laws: any[] = [];
  try {
    assert(successful(rust), "retained Rust law build failed");
    const data = messages(rust.stdout);
    assert.deepEqual(
      data.filter((x: any) => x.reason === "build-finished").map((x: any) => x.success),
      [true],
    );
    const binaries = data.filter(
      (x: any) =>
        x.reason === "compiler-artifact" &&
        x.executable &&
        x.profile.test &&
        packagePath(x.package_id) === fs.realpathSync(path.join(root, "crates/vize_glyph")),
    );
    assert.deepEqual(
      binaries.map((binary: any) => `${binary.target.kind.join(",")}:${binary.target.name}`).sort(),
      [
        "lib:vize_glyph",
        ...selectedIntegrationTargets.map((name: string) => `test:${name}`),
      ].sort(),
      "Cargo must emit exactly the reviewed lib plus eleven integration targets",
    );
    const listed: any[] = [];
    for (const [index, binary] of binaries.entries()) {
      const srcPath = binary.target.kind.includes("lib")
        ? "crates/vize_glyph/src/lib.rs"
        : `crates/vize_glyph/tests/${binary.target.name}.rs`;
      assert.equal(
        fs.realpathSync(binary.target.src_path),
        fs.realpathSync(path.join(root, srcPath)),
        "selected target source owner changed",
      );
      const bytes = fs.readFileSync(binary.executable);
      const binarySha256 = sha256(bytes);
      const frozenPath = path.join(evidence, `rust-binary-${index}-${binary.target.name}`);
      fs.copyFileSync(binary.executable, frozenPath, fs.constants.COPYFILE_EXCL);
      fs.chmodSync(frozenPath, 0o755);
      assert.equal(sha256(fs.readFileSync(frozenPath)), binarySha256);
      const listing = run(
        `rust-list-${index}`,
        frozenPath,
        ["--list", "--format", "terse"],
        {},
        30_000,
      );
      assert(successful(listing));
      const names = listing.stdout
        .toString()
        .split("\n")
        .filter((line: string) => line.endsWith(": test"))
        .map((line: string) => line.slice(0, -6));
      const executed = run(`rust-run-${index}`, frozenPath, [
        "--test-threads",
        "1",
        "--format",
        "pretty",
      ]);
      assert(successful(executed));
      const passed = executed.stdout
        .toString()
        .split("\n")
        .flatMap((line: string) => /^test (.+) \.\.\. ok$/.exec(line)?.slice(1) ?? []);
      assert.deepEqual(new Set(passed), new Set(names), "listed Rust tests must all actually pass");
      assert.equal(
        sha256(fs.readFileSync(frozenPath)),
        binarySha256,
        "frozen Rust binary changed during list/run",
      );
      listed.push({
        index,
        name: binary.target.name,
        kinds: binary.target.kind,
        tests: names,
        passed,
        binarySha256,
        binaryBytes: bytes.length,
        frozenPath: path.relative(evidence, frozenPath),
        cargoArtifact: binary,
        actualBuildCommand: ["cargo", ...rustBuild],
        sourceOwner: { path: srcPath, sha256: sha256(fs.readFileSync(path.join(root, srcPath))) },
        listProcess: listing.id,
        runProcess: executed.id,
      });
    }
    write("rust-binaries.json", listed);
    for (const law of requiredLaws) {
      const matches = listed
        .filter((x) => x.name === law.target && x.kinds.includes(law.kind))
        .flatMap((x) =>
          x.tests
            .filter(
              (name: string) =>
                (name === law.function || name.endsWith(`::${law.function}`)) &&
                (!law.modulePrefix || name.startsWith(`${law.modulePrefix}::`)),
            )
            .map((name: string) => ({ binaryIndex: x.index, test: name })),
        );
      assert.equal(
        matches.length,
        1,
        `unresolved actual Rust test owner ${law.law}:${law.owner}::${law.function}`,
      );
      laws.push({ ...law, ...matches[0], state: "actual-passed", publicOutputCredit: false });
    }
  } catch (error) {
    failures.push(`Rust laws: ${String(error)}`);
  }
  write("rust-law-results.json", laws);
  return laws;
}
