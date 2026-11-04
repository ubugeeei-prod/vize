#!/usr/bin/env bash
# Observe one unchanged builders command; never product qualification.
set -euo pipefail
observation="$RUNNER_TEMP/builders-exit-observation"
mkdir -p "$observation/bin"
real_node="$(node -p 'process.execPath')"
test -x "$real_node"
export BUILDERS_OBSERVATION="$observation"
export BUILDERS_REAL_NODE="$real_node"
"$real_node" - <<'NODE'
const fs = require('node:fs');
const cp = require('node:child_process');
const crypto = require('node:crypto');
const dir = process.env.BUILDERS_OBSERVATION;
const node = process.env.BUILDERS_REAL_NODE;
const quote = value => "'" + value.replaceAll("'", "'\\''") + "'";
fs.writeFileSync(dir + '/identity-before.json', JSON.stringify({
  diagnosticOnly: true, qualified: false,
  source: cp.execFileSync('git', ['rev-parse', 'HEAD'], {encoding:'utf8'}).trim(),
  tree: cp.execFileSync('git', ['rev-parse', 'HEAD^{tree}'], {encoding:'utf8'}).trim(),
  trackedDiff: cp.execFileSync('git', ['diff', 'HEAD', '--'], {encoding:'utf8'}),
  node, nodeSha256: crypto.createHash('sha256').update(fs.readFileSync(node)).digest('hex'),
  command: ['node', '.github/actions/test-js-packages-with-history/native-fault-evidence.ts', '--', 'vp', 'run', '--workspace-root', 'test:js:builders'],
}, null, 2));
const manifest = 'npm/builder/unplugin/package.json';
const original = fs.readFileSync(manifest, 'utf8');
const from = '"test": "node --test src/*.test.ts src/**/*.test.ts"';
const to = '"test": "node --trace-exit --trace-uncaught --test src/*.test.ts src/**/*.test.ts"';
if (original.split(from).length !== 2) throw new Error('Unexpected original unplugin test command');
fs.writeFileSync(dir + '/unplugin-package-original.json', original);
fs.writeFileSync(manifest, original.replace(from, to));
fs.writeFileSync(dir + '/trace-command.json', JSON.stringify({
  diagnosticOnly:true, qualified:false, manifest, originalCommand:JSON.parse(original).scripts.test,
  observedCommand:JSON.parse(fs.readFileSync(manifest,'utf8')).scripts.test,
  originalSha256:crypto.createHash('sha256').update(original).digest('hex'),
  observedSha256:crypto.createHash('sha256').update(fs.readFileSync(manifest)).digest('hex'),
  nodeOptionsPropagation:false,
}, null, 2));
fs.writeFileSync(dir + '/bin/node', `#!/usr/bin/env bash
set +e
printf '%s\\t%s\\t%s\\t%s\\n' "$$" "$PPID" start "$PWD" >> ${quote(dir + '/node-processes.tsv')}
${quote(node)} "$@"
status=$?
printf '%s\\t%s\\t%s\\t%s\\n' "$$" "$PPID" "$status" "$PWD" >> ${quote(dir + '/node-processes.tsv')}
exit "$status"
`, {mode:0o755});
NODE
restore_manifest() {
  cp "$observation/unplugin-package-original.json" npm/builder/unplugin/package.json
}
trap restore_manifest EXIT
export PATH="$observation/bin:$PATH"
set +e
node .github/actions/test-js-packages-with-history/native-fault-evidence.ts -- vp run --workspace-root test:js:builders 2>&1 | tee "$observation/command.log"
original_status=${PIPESTATUS[0]}
printf '%s\n' "$original_status" > "$observation/original-exit-code.txt"
vp run --last-details > "$observation/vp-last-details.txt" 2>&1
printf '%s\n' "$?" > "$observation/details-exit-code.txt"
set -e
restore_manifest
"$real_node" - <<'NODE'
const fs = require('node:fs');
const cp = require('node:child_process');
const dir = process.env.BUILDERS_OBSERVATION;
const before = JSON.parse(fs.readFileSync(dir + '/identity-before.json'));
const after = {
  source: cp.execFileSync('git', ['rev-parse', 'HEAD'], {encoding:'utf8'}).trim(),
  tree: cp.execFileSync('git', ['rev-parse', 'HEAD^{tree}'], {encoding:'utf8'}).trim(),
  trackedDiff: cp.execFileSync('git', ['diff', 'HEAD', '--'], {encoding:'utf8'}),
};
fs.writeFileSync(dir + '/identity-after.json', JSON.stringify({
  diagnosticOnly:true, qualified:false, ...after,
  sourceUnchanged: before.source === after.source && before.tree === after.tree && before.trackedDiff === '' && after.trackedDiff === '',
  originalExitCode: Number(fs.readFileSync(dir + '/original-exit-code.txt', 'utf8')),
}, null, 2));
NODE
exit "$original_status"
