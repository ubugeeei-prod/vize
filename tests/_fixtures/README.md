# Third-party fixtures

Projects under `_git/` are read-only upstream test inputs pinned as Git submodules. They are not
covered by Vize's license. Each project's revision, SPDX expression, and preserved license files
are recorded in `vue-ecosystem-fixtures.json`.

Do not patch fixture source to make a Vize test pass. Fix Vize itself, then rerun the same pinned
revision. When adding a fixture, keep its upstream license files in the submodule and declare every
license that applies to the tested source tree.
