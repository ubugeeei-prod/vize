import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { runInNewContext } from "node:vm";
import { parse } from "yaml";

const root = fileURLToPath(new URL("../..", import.meta.url));
const helper = join(root, ".github/actions/test-js-packages-with-history/native-fault-evidence.py");
const importHelper = `import importlib.util, sys
from pathlib import Path
sys.path.insert(0,str(Path(sys.argv[1]).parent))
spec = importlib.util.spec_from_file_location("fault", sys.argv[1])
fault = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fault)
`;

void test("only authenticated private Firecracker guests permit kernel changes", () => {
  const source =
    importHelper +
    `
from unittest.mock import patch
for vm, container, vendor, allowed in [
    ((0,"kvm"),(1,"none"),"Firecracker",True),
    ((0,"kvm"),(0,"docker"),"Firecracker",False),
    ((0,"kvm"),(1,"none"),"QEMU",False),
    ((1,"none"),(1,"none"),"Firecracker",False),
    ((None,""),(1,"none"),"Firecracker",False),
    ((0,"kvm"),(None,""),"Firecracker",False)]:
    def run(argv):
        value = vm if argv[-1] == "--vm" else container
        return {"status": value[0], "stdout": value[1]}
    with patch.object(fault,"run_capture",run), patch.object(fault,"read_text",lambda path: vendor):
        assert fault.private_guest()["privateFirecrackerGuest"] is allowed
print("all six actual guest authority cases passed")
`;
  const result = spawnSync("python3", ["-c", source, helper], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
});

for (const outcome of ["0", "7", "SIGTERM", "SIGKILL"]) {
  void test(`observer preserves one original argv/environment/output/cwd and ${outcome}`, () => {
    const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-"));
    try {
      const child = join(fixture, "original.py");
      const calls = join(fixture, "calls.jsonl");
      writeFileSync(
        child,
        `import json,os,signal,sys
with open(sys.argv[1],"a") as stream:
    stream.write(json.dumps({"argv":sys.argv[1:],"cwd":os.getcwd(),"env":dict(os.environ)})+"\\n")
print("original stdout",flush=True)
print("original stderr",file=sys.stderr,flush=True)
result=sys.argv[2]
if result.startswith("SIG"):
    os.kill(os.getpid(),getattr(signal,result))
sys.exit(int(result))
`,
      );
      const env = {
        ...process.env,
        GITHUB_ACTIONS: "true",
        GITHUB_RUN_ID: "123",
        GITHUB_RUN_ATTEMPT: "1",
        GITHUB_JOB: "law",
        RUNNER_TEMP: fixture,
        VIZE_FAULT_NONCE: "space and 'quote'\nnewline",
      };
      const source =
        importHelper +
        `
fault.sys.platform="linux"
fault.private_guest=lambda: {"privateFirecrackerGuest":False}
def forbidden(argv):
    raise AssertionError("unknown guest must not run sysctl/gdb: "+str(argv))
fault.run_capture=forbidden
sys.exit(fault.main(["--",sys.executable]+sys.argv[2:]))
`;
      const result = spawnSync(
        "python3",
        ["-c", source, helper, child, calls, outcome, "quoted argument with ' space\nnewline"],
        { cwd: fixture, env, encoding: "utf8" },
      );
      const records = readFileSync(calls, "utf8").trim().split("\n").map(JSON.parse);
      assert.equal(records.length, 1);
      assert.deepEqual(records[0].argv, [calls, outcome, "quoted argument with ' space\nnewline"]);
      assert.equal(records[0].cwd, realpathSync(fixture));
      const baselineEnv = spawnSync(
        "python3",
        ["-c", "import json,os;print(json.dumps(dict(os.environ)))"],
        { env, encoding: "utf8" },
      );
      assert.equal(baselineEnv.status, 0);
      assert.deepEqual(records[0].env, JSON.parse(baselineEnv.stdout));
      assert.equal(result.stdout, "original stdout\n");
      assert.equal(result.stderr, "original stderr\n");
      if (outcome.startsWith("SIG")) assert.equal(result.signal, outcome);
      else assert.equal(result.status, Number(outcome));
      const receipt = JSON.parse(
        readFileSync(join(fixture, "js-native-fault-evidence-123-1-law/outcome.json"), "utf8"),
      );
      assert.equal(receipt.coreCapture, "unavailable");
      assert.deepEqual(receipt.cores, []);
      assert.equal(receipt.command.length, 5);
      assert.equal("coreSetup" in receipt, false);
      assert.equal("coreRestore" in receipt, false);
      assert.equal(typeof receipt.observationGaps["source-before"], "string");
      assert.equal(typeof receipt.observationGaps["source-after"], "string");
    } finally {
      rmSync(fixture, { recursive: true, force: true });
    }
  });
}

void test("diagnostic collection failures retain an original command failure", () => {
  const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-gap-"));
  try {
    const source =
      importHelper +
      `
fault.sys.platform="linux"
fault.private_guest=lambda: {"privateFirecrackerGuest":False}
fault.observe_descendants=lambda *args: (_ for _ in ()).throw(PermissionError("typed observation gap"))
sys.exit(fault.main(["--",sys.executable,"-c","import time;time.sleep(.3);raise SystemExit(9)"]))
`;
    const result = spawnSync("python3", ["-c", source, helper], {
      env: {
        ...process.env,
        GITHUB_ACTIONS: "true",
        GITHUB_RUN_ID: "124",
        GITHUB_RUN_ATTEMPT: "1",
        GITHUB_JOB: "law",
        RUNNER_TEMP: fixture,
      },
      encoding: "utf8",
    });
    assert.equal(result.status, 9, result.stderr);
    const receipt = JSON.parse(
      readFileSync(join(fixture, "js-native-fault-evidence-124-1-law/outcome.json"), "utf8"),
    );
    assert.equal(receipt.observationGaps["process-maps"], "PermissionError");
    assert.equal(receipt.commandReturnCode, 9);
  } finally {
    rmSync(fixture, { recursive: true, force: true });
  }
});

void test("only an explicit label observes the frozen original failure; mandatory gates stay mandatory", () => {
  const workflow = parse(
    readFileSync(join(root, ".github/workflows/ci-walltime-benchmark.yml"), "utf8"),
  );
  const job = workflow.jobs["js-native-fault-diagnostic"];
  const admitted = (event, action, label) =>
    runInNewContext(job.if.slice(3, -3), {
      github: { event_name: event, event: { action, label: { name: label } } },
    });
  assert.equal(admitted("pull_request", "labeled", "ci-native-fault-diagnostic"), true);
  for (const event of ["push", "merge_group", "workflow_dispatch"])
    assert.equal(admitted(event, "labeled", "ci-native-fault-diagnostic"), false);
  for (const action of ["opened", "reopened", "synchronize"])
    assert.equal(admitted("pull_request", action, "ci-native-fault-diagnostic"), false);
  assert.equal(admitted("pull_request", "labeled", "ci-walltime-benchmark"), false);
  assert.equal(job["runs-on"], "blacksmith-32vcpu-ubuntu-2404");
  assert.equal(job["timeout-minutes"], 35);
  assert.deepEqual(job.env, {
    NODE_OPTIONS: "--disable-warning=DEP0040 --disable-warning=DEP0169",
    CARGO_BUILD_JOBS: "12",
  });
  assert.equal(job.steps[2].with.ref, "a16d50297208eb4635c824449bdbb345c462351d");
  const commands = job.steps.map((step) => step.run ?? "").join("\n");
  assert.equal((commands.match(/-- vp run --workspace-root test:js/g) ?? []).length, 1);
  assert.match(commands, /670e62ad66b6ff29b441e887bc7f62abb641c106/);
  assert.match(commands, /cp \.github\/actions.*native-fault-evidence\.py/);
  assert.doesNotMatch(commands, /continue-on-error|retry|DISABLE_TSGO|test-name-pattern/);
  const upload = job.steps.at(-1);
  assert.equal(upload.if, "${{ always() }}");
  assert.doesNotMatch(upload.with.path, /fault-cores|core\.\*|\.node/);
  const action = parse(
    readFileSync(join(root, ".github/actions/test-js-packages-with-history/action.yml"), "utf8"),
  );
  assert.equal(action.runs.steps[0].if, "${{ success() && job.status == 'success' }}");
  assert.equal(
    action.runs.steps[0].run,
    "python3 .github/actions/test-js-packages-with-history/native-fault-evidence.py -- vp run --workspace-root test:js",
  );
  assert.equal(action.runs.steps[1].run, "node npm/oxlint/scripts/check-project-transport.mjs");
  assert.equal(
    action.runs.steps[5].run,
    "node npm/oxlint/scripts/project-native-staging.ts --cleanup",
  );
  assert.equal(action.runs.steps[6].if, "${{ always() }}");
  assert.equal(
    action.runs.steps[6].with.path,
    "${{ runner.temp }}/js-native-fault-evidence-*/**/*.json",
  );
});

void test("public gdb text excludes startup arguments and retains bounded stream authority", () => {
  const source =
    importHelper +
    `
import hashlib,json
text="Core was generated by argument-secret\\nVIZE_NATIVE_FAULT_BEGIN\\n"+"f"*300000+"\\nVIZE_NATIVE_FAULT_END\\n"
program="import sys;sys.stdout.write("+repr(text[:text.index("f"*300000)])+"+'f'*300000+"+repr(text[text.index("f"*300000)+300000:])+");sys.stderr.write('private argument-secret')"
result=fault.run_capture([sys.executable,"-c",program],gated=True)
assert "argument-secret" not in result["stdout"]+result["stderr"]
assert len(result["stdout"]) == 262144
assert result["stderr"] == ""
assert result["stdoutAuthority"]["sha256"] == hashlib.sha256(text.encode()).hexdigest()
assert result["stdoutAuthority"]["truncated"] is True
print("bounded diagnostic selection passed")
`;
  const result = spawnSync("python3", ["-c", source, helper], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
});

void test("real core process PID and retained start ticks bind the exact physical ELF; unknowns refuse", () => {
  const source =
    importHelper +
    `
import hashlib,json,struct,tempfile
from native_fault_core import core_pid,producer
with tempfile.TemporaryDirectory() as directory:
    root=Path(directory);output=root/"evidence";packets=output/"processes";packets.mkdir(parents=True)
    header=bytearray(64);header[:6]=b"\\x7fELF\\x02\\x01";struct.pack_into("<HH",header,16,4,62)
    struct.pack_into("<Q",header,32,64);struct.pack_into("<HH",header,54,56,1)
    status=bytearray(40);struct.pack_into("<i",status,32,999)
    info=bytearray(136);struct.pack_into("<i",info,24,123)
    note=lambda kind,body: struct.pack("<III",5,len(body),kind)+b"CORE\\0\\0\\0\\0"+body
    notes=note(1,status)+note(3,info)
    program=bytearray(56);struct.pack_into("<I",program,0,4);struct.pack_into("<Q",program,8,120);struct.pack_into("<Q",program,32,len(notes))
    core=root/"core.123";core.write_bytes(header+program+notes)
    assert core_pid(core)==123 # Faulting thread 999 is not the producing process PID.
    executable=root/"actual-node";elf=bytearray(header);struct.pack_into("<H",elf,16,2);executable.write_bytes(elf)
    identity={"path":str(executable),"mappedInode":executable.stat().st_ino,"bytes":64,"sha256":hashlib.sha256(elf).hexdigest()}
    packet={"pid":123,"startTicks":"456","executable":str(executable),"providers":[identity]}
    first=packets/"123-0.json";first.write_text(json.dumps(packet))
    result=producer(core,output,fault.sha256_file)
    assert result["path"]==str(executable) and result["startTicks"]=="456"
    def refused():
        try: producer(core,output,fault.sha256_file)
        except ValueError: return
        raise AssertionError("Unqualified producer accepted")
    second=packets/"123-1.json";second.write_text(json.dumps(dict(packet,startTicks="789")));refused();second.unlink()
    executable.write_bytes(bytes(elf[:-1])+b"changed");refused()
    executable.write_bytes(elf);first.unlink();refused()
    core.rename(root/"core.999")
    try: producer(root/"core.999",output,fault.sha256_file)
    except ValueError: pass
    else: raise AssertionError("Wrong filename PID accepted")
print("whole actual core identity controls passed")
`;
  const result = spawnSync("python3", ["-c", source, helper], { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
});

for (const gap of ["source", "guest", "gdb"]) {
  void test(`${gap} collection failures execute the original command once and preserve failure`, () => {
    const fixture = mkdtempSync(join(tmpdir(), "vize-js-native-fault-once-"));
    try {
      const source =
        importHelper +
        `
import os
fault.sys.platform="linux"
fault.private_guest=lambda: {"privateFirecrackerGuest":False}
def unavailable(*args): raise PermissionError("typed ${gap} gap")
if "${gap}"=="source": fault.source_snapshot=unavailable
if "${gap}"=="guest": fault.private_guest=unavailable
if "${gap}"=="gdb":
    fault.diagnose_core=unavailable
    cores=Path(os.environ["RUNNER_TEMP"])/"js-native-fault-cores-125-1-law";cores.mkdir();(cores/"core.1").write_bytes(b"typed control")
program="from pathlib import Path;p=Path('calls');p.write_text((p.read_text() if p.exists() else '')+'once');raise SystemExit(11)"
sys.exit(fault.main(["--",sys.executable,"-c",program]))
`;
      const result = spawnSync("python3", ["-c", source, helper], {
        cwd: fixture,
        env: {
          ...process.env,
          GITHUB_ACTIONS: "true",
          GITHUB_RUN_ID: "125",
          GITHUB_RUN_ATTEMPT: "1",
          GITHUB_JOB: "law",
          RUNNER_TEMP: fixture,
        },
        encoding: "utf8",
      });
      assert.equal(result.status, 11, result.stderr);
      assert.equal(readFileSync(join(fixture, "calls"), "utf8"), "once");
      const receipt = JSON.parse(
        readFileSync(join(fixture, "js-native-fault-evidence-125-1-law/outcome.json"), "utf8"),
      );
      assert.equal(receipt.commandReturnCode, 11);
      assert.equal(receipt.coreCapture, "unavailable");
      assert.equal(
        receipt.observationGaps[
          gap === "source" ? "source-before" : gap === "gdb" ? "core-core.1" : gap
        ],
        "PermissionError",
      );
      if (gap === "source")
        assert.equal(receipt.observationGaps["source-after"], "PermissionError");
      assert.equal(typeof receipt.observerFileHash.bytes, "number");
      assert.equal(typeof receipt.observerFileHash.seconds, "number");
    } finally {
      rmSync(fixture, { recursive: true, force: true });
    }
  });
}
