#!/usr/bin/env bash
# Observe one unchanged builders command; never product qualification.
set -euo pipefail
observation="$RUNNER_TEMP/builders-exit-observation"
mkdir -p "$observation"
real_node="$(node -p 'process.execPath')"
test -x "$real_node"
command -v strace
strace --version > "$observation/strace-version.txt"
"$real_node" --input-type=module - "$observation" <<'NODE'
import fs from 'node:fs';
import cp from 'node:child_process';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import { pathToFileURL } from 'node:url';
const dir = process.argv[2];
const git = (...args) => cp.execFileSync('git', args);
const source = git('rev-parse','HEAD').toString().trim();
const tree = git('rev-parse','HEAD^{tree}').toString().trim();
assert.equal(source, '49f799942728504454c4ff4078448a852dbc8e8e');
assert.equal(tree, '9bec2fc09e10b00a08296619bba5779a5f4fdb28');
assert.equal(git('diff','--binary','HEAD').length, 0);
const manifest = 'npm/builder/unplugin/package.json';
assert.deepEqual(fs.readFileSync(manifest), git('show', 'HEAD:' + manifest));
assert.equal(JSON.parse(fs.readFileSync(manifest)).scripts.test, 'node --test src/*.test.ts src/**/*.test.ts');
assert.equal(process.env.NODE_OPTIONS, '--disable-warning=DEP0040 --disable-warning=DEP0169');
assert.equal(process.env.CARGO_BUILD_JOBS, '12');
// Run the actual source's guard before the one measured command.
const { nativeHistorySource } = await import(pathToFileURL(process.cwd() + '/npm/native/scripts/formatter-history-build.mts').href);
const guarded = nativeHistorySource(process.cwd() + '/npm/native');
assert.equal(guarded.head, source);
assert.equal(guarded.tree, tree);
fs.writeFileSync(dir + '/identity-before.json', JSON.stringify({
  diagnosticOnly:true, qualified:false, source, tree, guarded,
  trackedDiff:'', node:process.execPath,
  nodeSha256:crypto.createHash('sha256').update(fs.readFileSync(process.execPath)).digest('hex'),
  manifestSha256:crypto.createHash('sha256').update(fs.readFileSync(manifest)).digest('hex'),
  command:['node','.github/actions/test-js-packages-with-history/native-fault-evidence.ts','--','vp','run','--workspace-root','test:js:builders'],
  observation:['clone','clone3','fork','vfork','wait4','waitid','exit','exit_group','signals'],
  execveRecorded:false, envRecorded:false, argvRecorded:false,
},null,2));
NODE
set +e
strace -f -qq -ttt -e trace=clone,clone3,fork,vfork,wait4,waitid,exit,exit_group -e signal=all -o "$observation/process-exits.log" -- node .github/actions/test-js-packages-with-history/native-fault-evidence.ts -- vp run --workspace-root test:js:builders 2>&1 | tee "$observation/command.log"
original_status=${PIPESTATUS[0]}
printf '%s\n' "$original_status" > "$observation/original-exit-code.txt"
vp run --last-details > "$observation/vp-last-details.txt" 2>&1
printf '%s\n' "$?" > "$observation/details-exit-code.txt"
set -e
"$real_node" - "$observation" <<'NODE'
const fs=require('node:fs'),cp=require('node:child_process'),crypto=require('node:crypto');
const dir=process.argv[2],before=JSON.parse(fs.readFileSync(dir+'/identity-before.json'));
const source=cp.execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim();
const tree=cp.execFileSync('git',['rev-parse','HEAD^{tree}'],{encoding:'utf8'}).trim();
const diff=cp.execFileSync('git',['diff','--binary','HEAD']);
const manifestSha256=crypto.createHash('sha256').update(fs.readFileSync('npm/builder/unplugin/package.json')).digest('hex');
fs.writeFileSync(dir+'/identity-after.json',JSON.stringify({
 diagnosticOnly:true,qualified:false,source,tree,trackedDiffBytes:diff.length,manifestSha256,
 sourceUnchanged:before.source===source&&before.tree===tree&&diff.length===0&&before.manifestSha256===manifestSha256,
 originalExitCode:Number(fs.readFileSync(dir+'/original-exit-code.txt','utf8')),
},null,2));
NODE
exit "$original_status"
