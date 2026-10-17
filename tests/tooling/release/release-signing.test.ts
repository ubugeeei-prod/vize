import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import {
  chmodSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const command = fileURLToPath(
  new URL("../../../tools/support/release/sign_release_artifacts.ts", import.meta.url),
);
const tag = "v0.461.0";
const assets = [
  ...[
    "aarch64-apple-darwin",
    "aarch64-pc-windows-msvc",
    "aarch64-unknown-linux-gnu",
    "aarch64-unknown-linux-musl",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
  ].map(
    (target) => `cli-artifacts/vize-${target}.${target.includes("windows") ? "zip" : "tar.gz"}`,
  ),
  "editor-extension-artifacts/editors/vscode/dist/vize.vsix",
  ...["emacs", "helix", "nvim", "vim", "zed"].map(
    (editor) => `editor-extension-artifacts/${editor}-vize-extension.tar.gz`,
  ),
  `vize-${tag}-cyclonedx.sbom.json`,
  `vize-${tag}-spdx.sbom.json`,
].sort();
const suffixes = [".cosign.bundle", ".pem", ".sig"];
const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");

// The executable records real process boundaries. Its release-file barrier makes
// overlap deterministic, and append-only events avoid a shared counter race.
const fakeSigner = String.raw`
import { appendFileSync, existsSync, readFileSync, writeFileSync, symlinkSync } from 'node:fs';
import { spawn } from 'node:child_process';
const args = process.argv.slice(2), input = args.at(-1), mode = process.env.FAKE_MODE;
const event = (type) => appendFileSync(process.env.FAKE_EVENTS, JSON.stringify({type,input,pid:process.pid,args,yes:process.env.COSIGN_YES})+'\n');
event('start');
if (mode === 'fail' && input === process.env.FAKE_TARGET) process.exit(9);
if (mode === 'descendant' && input === process.env.FAKE_TARGET) {
  const child = spawn(process.execPath, ['--input-type=module', '-e', "import {existsSync,writeFileSync} from 'node:fs'; writeFileSync(process.env.FAKE_CHILD_READY,String(process.pid));const timer=setInterval(()=>{if(existsSync(process.env.FAKE_RELEASE)){clearInterval(timer);writeFileSync(process.env.FAKE_LATE,'late child wrote');}},10);"], {stdio:'ignore'});
  const ready = setInterval(()=>{if(existsSync(process.env.FAKE_CHILD_READY)){clearInterval(ready);child.unref();process.exit(0);}},10);
} else {
  const wait = setInterval(()=>{
    if (!existsSync(process.env.FAKE_RELEASE)) return;
    clearInterval(wait);
    if(mode==='mutate' && input===process.env.FAKE_TARGET) writeFileSync(input,'changed input');
    for (const option of ['--bundle','--output-certificate','--output-signature']) {
      const output = args[args.indexOf(option)+1];
      if(input===process.env.FAKE_TARGET && option==='--output-signature') {
        if(mode==='missing') continue;
        if(mode==='empty'){writeFileSync(output,'');continue;}
        if(mode==='symlink'){symlinkSync(input.split('/').at(-1),output);continue;}
      }
      writeFileSync(output, option+' '+input+' '+readFileSync(input).toString());
    }
    if(mode==='extra' && input===process.env.FAKE_TARGET) writeFileSync(input+'.foreign.sig','foreign');
    event('finish');
  },10);
}
`;

interface Event {
  type: string;
  input: string;
  pid: number;
  args: string[];
  yes: string;
}
function fixture() {
  const root = mkdtempSync(path.join(tmpdir(), "vize-release-signing-"));
  const bin = path.join(root, "bin");
  mkdirSync(bin);
  for (const file of assets) {
    mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    const contents = `original bytes ${file}\n`;
    writeFileSync(path.join(root, file), contents);
    if (!file.endsWith(".sbom.json"))
      writeFileSync(
        path.join(root, `${file}.sha256`),
        `${hash(contents)}  ${path.basename(file)}\n`,
      );
  }
  const signer = path.join(bin, "cosign");
  writeFileSync(signer, `#!${process.execPath}\n${fakeSigner}`);
  chmodSync(signer, 0o755);
  const events = path.join(root, "events.jsonl");
  const release = path.join(root, "release");
  const late = path.join(root, "late");
  const childReady = path.join(root, "child-ready");
  function rows(): Event[] {
    return existsSync(events)
      ? readFileSync(events, "utf8")
          .trim()
          .split("\n")
          .filter(Boolean)
          .map((row) => JSON.parse(row) as Event)
      : [];
  }
  function start(mode = "success", barrier = false, actualTag = tag) {
    if (!barrier) writeFileSync(release, "go");
    const child = spawn(process.execPath, [command, actualTag], {
      cwd: root,
      env: {
        ...process.env,
        PATH: `${bin}${path.delimiter}${process.env.PATH ?? ""}`,
        COSIGN_YES: "true",
        FAKE_MODE: mode,
        FAKE_TARGET: "cli-artifacts/vize-x86_64-apple-darwin.tar.gz",
        FAKE_EVENTS: events,
        FAKE_RELEASE: release,
        FAKE_LATE: late,
        FAKE_CHILD_READY: childReady,
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "",
      stderr = "";
    child.stdout.setEncoding("utf8").on("data", (chunk: string) => {
      stdout += chunk;
    });
    child.stderr.setEncoding("utf8").on("data", (chunk: string) => {
      stderr += chunk;
    });
    const closed = new Promise<{
      code: number | null;
      signal: NodeJS.Signals | null;
      stdout: string;
      stderr: string;
    }>((resolve, reject) => {
      child.once("error", reject);
      child.once("close", (code, signal) => resolve({ code, signal, stdout, stderr }));
    });
    return { child, closed };
  }
  return {
    signer,
    root,
    rows,
    start,
    release,
    late,
    childReady,
    clean: () => rmSync(root, { recursive: true, force: true }),
  };
}

async function until(predicate: () => boolean): Promise<void> {
  const deadline = Date.now() + 10_000;
  while (!predicate()) {
    assert.ok(Date.now() < deadline, "bounded process fixture condition timed out");
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
}

function notAlive(pid: number) {
  assert.throws(() => process.kill(pid, 0), { code: "ESRCH" });
}

test("four actual signers overlap, all sixteen original protocols and forty-eight outputs remain exact", async () => {
  const f = fixture();
  try {
    const before = new Map(
      assets.map((file) => [file, hash(readFileSync(path.join(f.root, file)))]),
    );
    const run = f.start("success", true);
    await until(() => f.rows().filter((event) => event.type === "start").length === 4);
    assert.equal(f.rows().filter((event) => event.type === "finish").length, 0);
    writeFileSync(f.release, "go");
    const result = await run.closed;
    assert.equal(result.code, 0, result.stderr);
    const events = f.rows();
    let active = 0,
      peak = 0;
    for (const event of events) {
      active += event.type === "start" ? 1 : -1;
      peak = Math.max(peak, active);
    }
    assert.equal(active, 0);
    assert.equal(peak, 4);
    const starts = events.filter((event) => event.type === "start");
    assert.deepEqual(starts.map((event) => event.input).sort(), assets);
    for (const row of starts) {
      assert.deepEqual(row.args, [
        "sign-blob",
        "--yes",
        "--bundle",
        row.input + ".cosign.bundle",
        "--output-certificate",
        row.input + ".pem",
        "--output-signature",
        row.input + ".sig",
        row.input,
      ]);
      assert.equal(row.yes, "true");
      notAlive(row.pid);
      for (const [index, suffix] of suffixes.entries()) {
        const option = ["--bundle", "--output-certificate", "--output-signature"][index];
        assert.equal(
          readFileSync(path.join(f.root, row.input + suffix), "utf8"),
          `${option} ${row.input} original bytes ${row.input}\n`,
        );
      }
    }
    const receipt = JSON.parse(result.stdout.trim().split("\n").at(-1)!) as {
      inputs: Record<string, string>;
      outputs: Record<string, string>;
    };
    assert.deepEqual(receipt.inputs, Object.fromEntries(before));
    assert.equal(Object.keys(receipt.outputs).length, 48);
    for (const [file, digest] of Object.entries(receipt.outputs))
      assert.equal(digest, hash(readFileSync(path.join(f.root, file))));
  } finally {
    f.clean();
  }
});

test("a signer failure stops queued work and reaps every overlapping child before returning", async () => {
  const f = fixture();
  try {
    const run = f.start("fail", true);
    const result = await run.closed;
    assert.equal(result.code, 1);
    assert.match(result.stderr, /Signer failed/);
    assert.ok(f.rows().length <= 4, "failed cohort must not start queued assets");
    for (const row of f.rows()) notAlive(row.pid);
    const snapshot = readFileSync(path.join(f.root, "events.jsonl"), "utf8");
    writeFileSync(f.release, "go");
    await new Promise((resolve) => setTimeout(resolve, 100));
    assert.equal(readFileSync(path.join(f.root, "events.jsonl"), "utf8"), snapshot);
    assert.doesNotMatch(result.stdout, /vize-release-signing-v1/);
  } finally {
    f.clean();
  }
});

test("SIGTERM abort waits for all four owned children and never leaves late outputs", async () => {
  const f = fixture();
  try {
    const run = f.start("success", true);
    await until(() => f.rows().filter((row) => row.type === "start").length === 4);
    run.child.kill("SIGTERM");
    const result = await run.closed;
    assert.equal(result.code, 143);
    assert.equal(result.signal, null);
    assert.match(result.stderr, /aborted|Signer failed/);
    for (const row of f.rows()) notAlive(row.pid);
    writeFileSync(f.release, "go");
    await new Promise((resolve) => setTimeout(resolve, 100));
    assert.equal(f.rows().length, 4);
  } finally {
    f.clean();
  }
});

test("a successful leader with a live descendant is refused and its late writer is killed", async () => {
  const f = fixture();
  try {
    const run = f.start("descendant", true);
    await until(() => existsSync(f.childReady));
    const pid = Number(readFileSync(f.childReady, "utf8"));
    const result = await run.closed;
    assert.equal(result.code, 1);
    assert.match(result.stderr, /descendants|process group remains/);
    writeFileSync(f.release, "go");
    await new Promise((resolve) => setTimeout(resolve, 100));
    assert.equal(existsSync(f.late), false);
    notAlive(pid);
    for (const row of f.rows()) notAlive(row.pid);
  } finally {
    f.clean();
  }
});

test("the same uncontained descendant really writes after its successful leader exits", async () => {
  const f = fixture();
  try {
    const control = spawn(f.signer, ["cli-artifacts/vize-x86_64-apple-darwin.tar.gz"], {
      cwd: f.root,
      detached: true,
      stdio: "ignore",
      env: {
        ...process.env,
        FAKE_MODE: "descendant",
        FAKE_TARGET: "cli-artifacts/vize-x86_64-apple-darwin.tar.gz",
        FAKE_EVENTS: path.join(f.root, "events.jsonl"),
        FAKE_RELEASE: f.release,
        FAKE_CHILD_READY: f.childReady,
        FAKE_LATE: f.late,
      },
    });
    const closed = new Promise<number | null>((resolve, reject) => {
      control.once("error", reject);
      control.once("close", resolve);
    });
    assert.equal(await closed, 0);
    const pid = Number(readFileSync(f.childReady, "utf8"));
    assert.equal(existsSync(f.late), false);
    writeFileSync(f.release, "go");
    await until(() => existsSync(f.late));
    assert.equal(readFileSync(f.late, "utf8"), "late child wrote");
    await until(() => {
      try {
        process.kill(pid, 0);
        return false;
      } catch (error) {
        assert.equal((error as NodeJS.ErrnoException).code, "ESRCH");
        return true;
      }
    });
  } finally {
    f.clean();
  }
});

for (const mode of ["mutate", "missing", "empty", "symlink", "extra"]) {
  test(`successful child exits cannot hide ${mode} output/input corruption`, async () => {
    const f = fixture();
    try {
      const result = await f.start(mode).closed;
      assert.equal(result.code, 1, result.stdout);
      assert.match(
        result.stderr,
        /changed during signing|closure differs|nonempty regular file|non-regular release entry/,
      );
      assert.equal(f.rows().filter((row) => row.type === "finish").length, 16);
      assert.doesNotMatch(result.stdout, /vize-release-signing-v1/);
      for (const row of f.rows()) notAlive(row.pid);
    } finally {
      f.clean();
    }
  });
}

test("missing assets, stale signatures, wrong checksums and linked inputs fail before any signer", async () => {
  for (const mode of ["missing", "stale", "checksum", "symlink"]) {
    const f = fixture();
    try {
      const first = path.join(f.root, assets[0]);
      if (mode === "missing") rmSync(first);
      if (mode === "stale") writeFileSync(first + ".sig", "old");
      if (mode === "checksum") writeFileSync(first + ".sha256", "wrong");
      if (mode === "symlink") {
        rmSync(first);
        symlinkSync(path.join(f.root, assets[1]), first);
      }
      const result = await f.start().closed;
      assert.equal(result.code, 1);
      assert.equal(f.rows().length, 0);
    } finally {
      f.clean();
    }
  }
});

test("invalid tag paths cannot select a different input/output set", async () => {
  const f = fixture();
  try {
    const result = await f.start("success", false, "v0.461.0/../../outside").closed;
    assert.equal(result.code, 1);
    assert.equal(f.rows().length, 0);
  } finally {
    f.clean();
  }
});

const nodeSetup =
  '      - uses: actions/setup-node@249970729cb0ef3589644e2896645e5dc5ba9c38 # v6\n        with: { node-version-file: "package.json", package-manager-cache: false }\n';
const newStep =
  nodeSetup +
  '      - name: Sign release artifacts (keyless)\n        env:\n          COSIGN_YES: "true"\n        run: node tools/support/release/sign_release_artifacts.ts "$RELEASE_TAG_NAME"\n';
const originalSigning = `      - name: Sign release artifacts (keyless)
        env:
          COSIGN_YES: "true"
        run: |
          for artifact in \\
            cli-artifacts/*.tar.gz \\
            cli-artifacts/*.zip \\
            editor-extension-artifacts/editors/vscode/dist/*.vsix \\
            editor-extension-artifacts/*.tar.gz \\
            vize-\${RELEASE_TAG_NAME}-cyclonedx.sbom.json \\
            vize-\${RELEASE_TAG_NAME}-spdx.sbom.json; do
            if [ -f "$artifact" ]; then
              echo "Signing $artifact"
              cosign sign-blob --yes \\
                --bundle "\${artifact}.cosign.bundle" \\
                --output-certificate "\${artifact}.pem" \\
                --output-signature "\${artifact}.sig" \\
                "$artifact"
            fi
          done
`;

test("the finite scheduling inverse preserves the entire original final Release job and every gate", () => {
  const workflow = readFileSync(
    new URL("../../../.github/workflows/release.yml", import.meta.url),
    "utf8",
  );
  const job = workflow.slice(
    workflow.indexOf("  create-github-release:"),
    workflow.indexOf("  release-npm-cli:"),
  );
  function historical(value: string) {
    assert.equal(value.split(newStep).length, 2, "exact new invocation/setup is mandatory");
    return value.replace(newStep, originalSigning + nodeSetup);
  }
  // Immutable whole cab569 final-job bytes; the inverse changes only scheduling
  // and moves the already-pinned Node setup before the new TS invocation.
  const originalDigest = "5811ad12d233d79822b10133dfc468104f2c2edd84fe2c20fe56d94cd48a0481";
  assert.equal(hash(historical(job)), originalDigest);
  for (const [from, to] of [
    ["      - release-npm-native\n", ""],
    ["      - release-preflight\n", ""],
    ["cosign-release: v2.4.1", "cosign-release: v2.4.0"],
    ["      id-token: write", "      id-token: read"],
  ]) {
    assert.notEqual(hash(historical(job.replace(from, to))), originalDigest);
  }
  assert.throws(() => historical(job.replace('"$RELEASE_TAG_NAME"', '"v0.1.0"')), /mandatory/);
  const carrier = readFileSync(
    new URL("../../../.github/workflows/release-preflight-contracts.yml", import.meta.url),
    "utf8",
  );
  assert.ok(carrier.includes("          tests/tooling/release/release-signing.test.ts\n"));
});
