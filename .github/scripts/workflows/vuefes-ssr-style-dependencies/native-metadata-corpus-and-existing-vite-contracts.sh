#!/usr/bin/env bash
vuefes_contract_rc=0
node --test tests/tooling/vuefes-failure-*.test.mjs tools/support/compat/vuefes/passive-failure-sidecar.browser.test.mjs || vuefes_contract_rc=$?
node --experimental-vm-modules tools/support/compat/vuefes/page-meta-dependencies.ts || vuefes_contract_rc=$?
node npm/native/scripts/test-preparation.mjs vp run --filter './npm/builder/vite' test:prepared || vuefes_contract_rc=$?
exit "$vuefes_contract_rc"
