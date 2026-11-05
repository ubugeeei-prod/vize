#!/usr/bin/env bash
set -euo pipefail

unset VIZE_TEST_DISABLE_TSGO
mkdir -p "$VIZE_IMPORTED_RAW_CAPTURE"
set +e
cargo test --locked --profile ci -p vize --test integration -- check_imported_raw_callback_cli::imported_mark_raw_getter_reads_keep_direct_callback_and_authored_error_controls --exact --nocapture --color never >"$VIZE_IMPORTED_RAW_CAPTURE/test.stdout" 2>"$VIZE_IMPORTED_RAW_CAPTURE/test.stderr"
status=$?
set -e
printf '%s\n' "$status" >"$VIZE_IMPORTED_RAW_CAPTURE/test.status"
cat "$VIZE_IMPORTED_RAW_CAPTURE/test.stdout"
cat "$VIZE_IMPORTED_RAW_CAPTURE/test.stderr" >&2
node --input-type=module - "$VIZE_IMPORTED_RAW_CAPTURE" <<'NODE'
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {join} from 'node:path';
const out=process.argv[2],stdout=readFileSync(join(out,'test.stdout')),stderr=readFileSync(join(out,'test.stderr'));
const status=Number(readFileSync(join(out,'test.status'),'utf8').trim());
const executed=/^test result: ok\. 1 passed; 0 failed; 0 ignored; /m.test(stdout.toString('utf8'))?1:0;
const sha=b=>createHash('sha256').update(b).digest('hex');
writeFileSync(join(out,'test-run.json'),JSON.stringify({source:execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim(),run:process.env.GITHUB_RUN_ID,attempt:process.env.GITHUB_RUN_ATTEMPT,name:'check_imported_raw_callback_cli::imported_mark_raw_getter_reads_keep_direct_callback_and_authored_error_controls',status,executed,stdoutSha256:sha(stdout),stderrSha256:sha(stderr),historical118Resolved:false,nativeClean:false,nativeKnownDebt:true,timingUsable:false},null,2)+'\n');
if(status!==0)process.exit(status);
if(executed!==1)throw Error('derived callback fixture did not execute exactly once');
NODE
