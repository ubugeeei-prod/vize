#!/usr/bin/env bash
# Observe one unchanged builders command; never product qualification.
set -euo pipefail
observation="$RUNNER_TEMP/builders-exit-observation"
mkdir -p "$observation/bin"
real_node="$(command -v node)"
real_node="$(readlink -f "$real_node")"
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
fs.writeFileSync(dir + '/bin/node', `#!/usr/bin/env bash
set +e
printf '%s\\t%s\\t%s\\t%s\\n' "$$" "$PPID" start "$PWD" >> ${quote(dir + '/node-processes.tsv')}
${quote(node)} --trace-exit --trace-uncaught "$@"
status=$?
printf '%s\\t%s\\t%s\\t%s\\n' "$$" "$PPID" "$status" "$PWD" >> ${quote(dir + '/node-processes.tsv')}
exit "$status"
`, {mode:0o755});
NODE
export PATH="$observation/bin:$PATH"
export NODE_OPTIONS="${NODE_OPTIONS:-} --trace-exit --trace-uncaught"
set +e
node .github/actions/test-js-packages-with-history/native-fault-evidence.ts -- vp run --workspace-root test:js:builders 2>&1 | tee "$observation/command.log"
original_status=${PIPESTATUS[0]}
printf '%s\n' "$original_status" > "$observation/original-exit-code.txt"
vp run --last-details > "$observation/vp-last-details.txt" 2>&1
printf '%s\n' "$?" > "$observation/details-exit-code.txt"
set -e
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
