# Hydrate Element Plus before full native workspace tests

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The full manual/scheduled Check workspace runner needs the actual Element Plus
gitlink before its unchanged native syntax integration tests. Hydrate that
fixture in the existing Pkl setup step of `clippy-and-test`, before `Test`.
The existing Pkl install must still succeed first. Both commands fail normally;
neither command is conditional or allowed to continue after failure.

The native test compares the original fixture's Git HEAD with the frozen
`20298a21ae54a5417955373b80130ae7c548a6bb` pin, then compares the original
popover/tooltip files and exercises their complete import graph. The gitlink,
upstream repository, input files and every assertion remain unchanged.
This change reads the existing upstream fixture; it makes no upstream writes.

An uninitialized gitlink directory lets `git rev-parse HEAD` discover the
enclosing Vize repository instead. The resulting Vize commit is correctly
rejected by the original pin assertion. Keep that refusal and provide its real
input, rather than substituting a pin, skipping the test or accepting Vize HEAD.

The earlier full run
[38152433486](https://github.com/ubugeeei-prod/vize/actions/runs/38152433486)
failed the Element Plus pin in native job `114508465496`. The latest source
`d7a33c60acc089b6d9545f07048317b7cab4a9a2`, full Check
[38157426440](https://github.com/ubugeeei-prod/vize/actions/runs/38157426440),
still omits hydration in `clippy-and-test` and its complete local action closure.
Its `Test` step was running when this prerequisite was prepared. Preserve that
original outcome; this record does not claim the pending run failed or passed.

The four-fixture hydration at `check.yml` line 291 belongs to `test-scripts`, a
separate runner. Editing that list would not hydrate the native workspace job.
The merge-group Rust workers and the native-phase workflow already provide
their own fixture setup. Leave those setup commands and event selectors intact.

The authored workflow regression fails on the old missing setup and passes with
this change. It checks the same job, setup ordering, unconditional failure
behavior, existing manual/scheduled selector, actual Git gitlink and unchanged
native pin assertion. The complete adjacent Check/source/merge-queue laws also
pass locally. Actual hosted full native workspace execution, the complete
source checks, protected candidate and actual merge remain required.

No native producer, release-law cohort body, corpus, diagnostic oracle or budget
changes. The source Check and full merge-group qualification remain complete.
The manual full Check must independently exercise this repaired setup and retain
the original native test result before this prerequisite is considered qualified.
