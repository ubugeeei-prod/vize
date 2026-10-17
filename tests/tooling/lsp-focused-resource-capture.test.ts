import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { parse } from "yaml";

import { readRepoFile } from "./support/github-workflows.ts";

test("the traced original packet retains spawn failures and always closes its resource observer", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "lsp-focused-projects.yml"));
  const step = workflow.jobs["original-projects"].steps.find(
    (candidate: { id?: string }) => candidate.id === "lifecycle",
  );
  const body = step.run.match(/python3 - <<'PY'\n([\s\S]*)\nPY\s*$/)?.[1];
  assert.ok(body, "missing original Python packet runner");
  const result = spawnSync(
    "python3",
    [
      "-c",
      String.raw`
import json, os, pathlib, signal, subprocess, sys, tempfile, types
source = sys.stdin.read()
outcomes = []
for mode in ['success', 'packet-spawn-failure', 'observer-spawn-failure']:
    with tempfile.TemporaryDirectory() as temp:
        root = pathlib.Path(temp)
        directory = root / 'misskey'
        directory.mkdir()
        provider = root / 'provider'
        provider.write_bytes(b'fixture provider identity')
        (root / 'plan.json').write_text(json.dumps({
            'preparationTrace': True, 'shardCount': 145,
            'projects': [{'id': 'misskey', 'shardIndex': 12}],
        }))
        os.environ.update(CAMPAIGN_DIR=str(root), CORSA_PATH=str(provider))
        packet = ['node', '--test', '--test-concurrency=1', 'tests/tooling/real-project-lsp.test.ts']
        calls, streams, events = [], [], []
        class Observer:
            status = None
            def poll(self): return self.status
            def send_signal(self, value): events.append(('signal', value))
            def wait(self, timeout=None):
                events.append(('wait', timeout))
                self.status = 0
                return self.status
            def kill(self): events.append(('kill',))
        def popen(command, **options):
            assert command[:3] == ['node', '--input-type=module', '--eval']
            assert 'lsp-resource-sampler.mjs' in command[3]
            assert command[4:] == [str(os.getpid()), str(directory)]
            streams.extend([options['stdout'], options['stderr']])
            if mode == 'observer-spawn-failure':
                raise OSError('fixture observer spawn failure')
            return Observer()
        def run(command, **options):
            calls.append(command)
            if command == packet and mode == 'packet-spawn-failure':
                raise OSError('fixture packet spawn failure')
            return types.SimpleNamespace(returncode=0)
        subprocess.Popen, subprocess.run = popen, run
        try:
            exec(compile(source, 'actual workflow lifecycle runner', 'exec'), {})
        except AssertionError as error:
            assert mode == 'packet-spawn-failure' and 'all selected raw packet' in str(error), error
        else:
            assert mode != 'packet-spawn-failure', 'packet spawn failure was accepted'
        assert calls.count(packet) == 1, calls
        assert all(stream.closed for stream in streams)
        receipt = json.loads((root / 'sessions.json').read_text())[0]
        assert receipt['command'] == packet and receipt['wireExitStatus'] == 0
        if mode == 'packet-spawn-failure':
            assert receipt['exitStatus'] is None
            assert 'fixture packet spawn failure' in receipt['commandError']
            assert 'fixture packet spawn failure' in (directory / 'test.stderr').read_text()
        else:
            assert receipt['exitStatus'] == 0 and receipt['commandError'] is None
        if mode == 'observer-spawn-failure':
            assert receipt['resourceExitStatus'] == -1
            assert 'fixture observer spawn failure' in (directory / 'resource-error.txt').read_text()
        else:
            assert events == [('signal', signal.SIGTERM), ('wait', 10)], events
            assert receipt['resourceExitStatus'] == 0
        outcomes.append(mode)
print(json.dumps(outcomes))
`,
    ],
    { input: body, encoding: "utf8", timeout: 5_000 },
  );
  assert.equal(result.status, 0, result.stderr || String(result.error));
  assert.deepEqual(JSON.parse(result.stdout), [
    "success",
    "packet-spawn-failure",
    "observer-spawn-failure",
  ]);
});
