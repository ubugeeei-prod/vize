#!/usr/bin/env bash
cp -R tools/support/compat/github/corsa-package-refresh "$RUNNER_TEMP/corsa-package-refresh-harness"
vp node "$RUNNER_TEMP/corsa-package-refresh-harness/compare.mjs"
vp node "$RUNNER_TEMP/corsa-package-refresh-harness/fixture.mjs"
