# Handoff: refactor/generated-name-sigil

Unify generated `__vize_*` names on `$name` through `vize_l0::ts_emit::generated_name`.
One squashed commit (`refactor(canon): spell generated values as $name through one table`),
based on main `2de08e4787` (#13241). Draft PR #13230 was closed by the coordinator
(no-draft policy); this branch has no open PR. Delete this `.handoff/` file before merge.

## State at handoff

- Locally verified: canon lib (only the 13 known macOS path/config failures, which also fail on main),
  `--test empty_vue_star_exports` 115/115, `lsp_named_model_transactions_cli` 9/9,
  maestro `canonical::mapping` tests, clippy on maestro, source-file-lengths vs base,
  `generated-name-spelling` + `content-mapper-helper-abi` tooling tests.
- Not yet run on CI for this exact head: the last `Check` run was cancelled by the coordinator's CI pause.
- Expected non-required failures (byte-compare vs base by design): `parity`, `compare` (x2),
  `unaffected-projections`, `unchanged-projection-packets`, `published-sdk-packets`, `admission`, `real-beta-host`.
  Coordinator accepted the rename as a deliberate documented change; diagnostics, spans, LSP results and
  compiled runtime output must stay identical.

## Next steps (in order, only after the coordinator lifts the CI pause)

1. `git fetch origin && git rebase origin/main`. Main has moved (at least #13255, which touches canon prop typing,
   e.g. `crates/vize_canon/src/virtual_ts/props/opaque_runtime.rs`). Respell any new `__vize_*` the new main commits
   added in producers or test expectations (`node --test tests/tooling/generated-name-spelling.test.ts` finds
   producer spellings; grep test expectations by hand).
2. Rebuild the custody record against the NEW base and pin it (scripts below; copy them to a scratch dir with `hunks.py`):
   - `git grep -hoE '(crates|davinci)/[A-Za-z0-9_./-]+\.(rs|txt|snap|json|ts)' -- tools tests crates ':!tests/_fixtures/tooling/generated-names-source.json' | sort -u > <scratch>/referenced.txt`
   - `python3 <scratch>/make_record.py origin/main tests/_fixtures/tooling/generated-names-source.json && python3 <scratch>/pin_record.py`
   - Re-run this every time any producer file changes, immediately before tests and before pushing.
3. `cargo fmt --all`, `vp fmt`, clippy (`--profile ci -- -D warnings -D clippy::wildcard_imports`) on touched crates,
   `node tools/commands/ci/source-file-lengths.ts --check --base-ref origin/main`, then local tests:
   `cargo test --profile ci -p vize_canon --lib --test empty_vue_star_exports`,
   `cargo test --profile ci -p vize --test integration -- lsp_named_model_transactions_cli` (set `TSGO_PATH`).
   If ecosystem hashes move: build the CLI (`cargo build --locked -p vize --profile ci && node tests/differential/build-receipt.ts`)
   and run `eco_loop.py`.
4. Squash onto origin/main with the commit message below, push once.
5. Open a fresh READY PR (never draft) with the PR body below. Immediately run `python3 <scratch>/repoint.py <NEW_PR>`
   (moves the maestro generated-names receipt from #13230 to the new PR and re-pins its sha), amend, push.
6. Let that one CI run finish; fix required failures (check-js, check-vize-apps, fmt-rust, test-report and its deps).
7. Perf: dispatch ONE quiet-runner interleaved base/head benchmark (pattern: #13216's bench workflow) and ONE
   `level-instruction-counts.yml` dispatch; put the table in the PR body (local numbers in the body are not evidence
   under the 2026-11-17 rule; replace them).
8. `gh pr merge <n> --squash`, verify `gh pr view <n> --json mergeStateStatus,state`, then final report:
   merged SHA, mapping table (in PR body), what keeps `__vize_` and why (in PR body, "Unchanged on purpose").

## Pending decisions

- None from the user. PR2 (renaming file names / module specifiers / bundler ids / published `__vize_component__` and
  `virtual-ts-globals.d.ts` shims) was evaluated and deferred: those are external or published contracts.

## Gotchas

- The `origin/main` ref is shared with other agents and moves under you; pin the base SHA explicitly when comparing.
- Editing a byte-pinned custody layer file breaks its own hash pin (e.g. `emit_overload_window.rs`); only the layers
  already routed through `super::generated_names` were edited.
- maestro picked mappings by shortest generated length; short `$handler…` anchors stole `@update:` navigation.
  Fixed in `canonical/mapping.rs` (generated-name anchors rank last). Watch for similar length tie-breaks.
- `generated_names.rs` in canon is at 496 lines (limit 500); `vize_l0/src/lib.rs` is pinned by the host-path test,
  which is why the module lives under `ts_emit`.
- Local-only failures to ignore: macOS `/tmp` vs `/private/tmp` canonicalization, missing element-plus submodule,
  `vp check` module-not-found in a playground.

## Scripts

### `make_record.py`

```python
"""Write the generated-name source custody record.

Usage: python3 make_record.py BASE_REF OUT_JSON  (run from the repo root)
Every changed non-test producer file gets before/after identities and
unique hunks whose inverse restores the BASE_REF bytes exactly.
"""
import difflib, hashlib, json, os, subprocess, sys

base, out = sys.argv[1], sys.argv[2]
changed = subprocess.run(["git", "diff", "--name-only", base, "--", "crates", "davinci"],
                         capture_output=True, text=True, check=True).stdout.split()

REFERENCED = set(open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "referenced.txt")).read().split())

def is_producer(path):
    # Producers, plus any changed file a custody record or cone names.
    if path.endswith("generated_names.rs") and "payload_update" in path:
        return False
    if not path.startswith(("crates/", "davinci/")):
        return False
    parts = path.split("/")
    producer = "/src/" in path and not path.endswith(".snap") and "tests" not in parts \
        and not path.endswith(("tests.rs", "_tests.rs"))
    return producer or path in REFERENCED

def ident(data: bytes):
    return {"bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}

import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from hunks import hunks_for

files = []
for path in changed:
    if not is_producer(path):
        continue
    try:
        before = subprocess.run(["git", "show", f"{base}:{path}"], capture_output=True, check=True).stdout
    except subprocess.CalledProcessError:
        continue
    after = open(path, "rb").read()
    hunks = hunks_for(before.decode(), after.decode())
    restored = after.decode()
    for h in hunks:
        assert restored.count(h["after"]) == 1, path
        restored = restored.replace(h["after"], h["before"], 1)
    assert restored.encode() == before, path
    files.append({"path": path, "before": ident(before), "after": ident(after), "hunks": hunks})

record = {
    "schema": "vize.generatedNameSource.v1",
    "credit": {"compiled": False, "native": False, "performance": False},
    "files": files,
}
text = json.dumps(record, indent=2, ensure_ascii=False) + "\n"
open(out, "w").write(text)
print(len(files), "files", hashlib.sha256(text.encode()).hexdigest())
```

### `hunks.py`

```python
import difflib

def hunks_for(before: str, after: str):
    a, b = before.splitlines(keepends=True), after.splitlines(keepends=True)
    ops = [op for op in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes() if op[0] != "equal"]
    groups = [[k, k, 0] for k in range(len(ops))]  # first op, last op, context

    def window(g):
        f, l, c = g
        i1, j1 = ops[f][1], ops[f][3]
        i2, j2 = ops[l][2], ops[l][4]
        return (max(0, i1 - c), min(len(a), i2 + c), max(0, j1 - c), min(len(b), j2 + c))

    while True:
        # merge groups whose after windows overlap or touch
        merged = []
        for g in groups:
            if merged:
                pw, gw = window(merged[-1]), window(g)
                if gw[2] <= pw[3]:
                    merged[-1] = [merged[-1][0], g[1], max(merged[-1][2], g[2])]
                    continue
            merged.append(g)
        groups = merged
        changed = False
        for g in groups:
            lo_a, hi_a, lo_b, hi_b = window(g)
            ha = "".join(b[lo_b:hi_b])
            if not ha or after.count(ha) != 1:
                g[2] += 1
                changed = True
        if not changed:
            break
    out = []
    for g in groups:
        lo_a, hi_a, lo_b, hi_b = window(g)
        out.append({"before": "".join(a[lo_a:hi_a]), "after": "".join(b[lo_b:hi_b])})
    return out
```

### `pin_record.py`

```python
import hashlib, re
rec = "tests/_fixtures/tooling/generated-names-source.json"
sha = hashlib.sha256(open(rec, "rb").read()).hexdigest()
for path, pat in [
    ("tools/support/compat/content-mapper/generated-names-transition.ts", r'(const recordSha256 = ")([^"]*)(";)'),
    ("crates/vize_canon/tests/empty_vue_star_exports/membership/constructor_without_args/payload_update/generated_names.rs", r'(const RECORD_SHA: &str = ")([^"]*)(";)'),
]:
    s = open(path).read()
    s2 = re.sub(pat, lambda m: m.group(1) + sha + m.group(3), s)
    assert s2 != s or sha in s, path
    open(path, "w").write(s2)
print(sha)
```

### `repoint.py`

```python
"""repoint.py NEW_PR: move the maestro generated-names receipt from #13230 to NEW_PR and re-pin its sha."""
import hashlib, sys
new = sys.argv[1]
base = "crates/vize_maestro/tests/davinci_ts40_projection_support/"
receipt = base + "fixtures/public-instance-options-api-slots/successor-generated-names/receipt.json"
readme = base + "fixtures/public-instance-options-api-slots/README.md"
evidence = base + "evidence/generated_names.rs"

text = open(receipt).read()
old_sha = hashlib.sha256(text.encode()).hexdigest()
assert text.count('"pullRequest": 13230,') == 1
text = text.replace('"pullRequest": 13230,', f'"pullRequest": {new},')
text = text.replace("(`vize_l0::generated_name`)", "(`vize_l0::ts_emit::generated_name`)")
open(receipt, "w").write(text)
new_sha = hashlib.sha256(text.encode()).hexdigest()

src = open(evidence).read()
assert src.count(old_sha) == 1 and src.count("        13230,\n") == 1
src = src.replace(old_sha, new_sha).replace("        13230,\n", f"        {new},\n")
src = src.replace("(`vize_l0::generated_name`)", "(`vize_l0::ts_emit::generated_name`)")
open(evidence, "w").write(src)

doc = open(readme).read()
assert doc.count("records #13230") == 1
doc = doc.replace("records #13230", f"records #{new}")
doc = doc.replace("from `vize_l0::generated_name`", "from `vize_l0::ts_emit::generated_name`")
open(readme, "w").write(doc)
print("receipt", old_sha, "->", new_sha)
```

### `eco_loop.py`

```python
import subprocess, re, os
W = os.path.dirname(os.path.abspath(__file__))
ENV = dict(os.environ)
ENV["PATH"] = open(os.path.join(W, "nixpath.txt")).read().strip()
p = 'tests/_fixtures/ecosystem-producer-qualification-11806.json'
for round_ in range(8):
    r = subprocess.run(["node", "--test", "tests/tooling/ecosystem-producer-qualification.test.ts"], capture_output=True, text=True, env=ENV)
    out = r.stdout + r.stderr
    if r.returncode == 0:
        print("PASS after", round_); break
    plus = dict(re.findall(r"^\s*\+\s+'([^']+)': '([0-9a-f]{64})'", out, re.M))
    minus = dict(re.findall(r"^\s*-\s+'([^']+)': '([0-9a-f]{64})'", out, re.M))
    if not plus:
        print(out[-2000:]); break
    s = open(p).read()
    # replace only the first occurrence of each expected hash after its key within the failing map
    for key, new in plus.items():
        old = minus.get(key)
        if not old:
            continue
        needle = f'"{key}": "{old}"'
        assert s.count(needle) >= 1, needle
        s = s.replace(needle, f'"{key}": "{new}"', 1)
    open(p, "w").write(s)
    print("round", round_, "updated", len(plus))
```

### `run_tooling.py`

```python
"""Run node --test per file with concurrency; print FAIL lines. Usage: run_tooling.py LIST OUT"""
import subprocess, sys, os, concurrent.futures as cf
ENV = dict(os.environ)
ENV["PATH"] = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "nixpath.txt")).read().strip()
files = open(sys.argv[1]).read().split()
out = open(sys.argv[2], "w")
def run(f):
    try:
        p = subprocess.run(["node", "--test", f], capture_output=True, text=True, timeout=900, env=ENV)
        return f, p.returncode, (p.stdout + p.stderr)
    except subprocess.TimeoutExpired:
        return f, -1, "TIMEOUT"
with cf.ThreadPoolExecutor(6) as ex:
    for f, code, text in ex.map(run, files):
        status = "PASS" if code == 0 else "FAIL"
        out.write(f"== {status} {f}\n")
        if code != 0:
            out.write(text[-4000:] + "\n")
        out.flush()
        print(status, f, flush=True)
```

`eco_loop.py` and `run_tooling.py` read `nixpath.txt` (the `PATH` inside `nix develop`): create it with `nix develop --command bash -c 'echo $PATH' > <scratch>/nixpath.txt`. `run_tooling.py LIST LOG` runs each test file listed in LIST.

## Commit message

```text
refactor(canon): spell generated values as $name through one table

Generated values, keys, markers and parse probes were `__vize_snake_case`.
That spelling was scattered across ~160 files and showed up in hovers and
`--show-virtual-ts` as names like `__vize_template_instance`.

`vize_l0::ts_emit::generated_name` is now the only place a generated name is spelled.
Producers capture its constants in format strings or `ts!` holes, or use
`generated_name!` where a literal is required. Consumers recognise names
through `GeneratedName::{is, is_member, strip, occurs_in, ends}` and
`is_generated`.

Names are short and plain (`$inst`, `$args`, `$cb`, `$propsNav0`) and never
equal a Vue instance member. While a projection is bound, a name an SFC
authors takes trailing `$`s (`$args` -> `$args$`), so authored `$` bindings
check exactly as before.

Maestro ranks a generated-name anchor after every other mapping with the same
authored length. The shorter `$handler` anchor no longer captures
`@update:` navigation.

Producer custody is inverted first in every source layer by
`generated-names-source.json`. Published declaration output, the
`virtual-ts-globals.d.ts` macro shims, file names, module specifiers,
compiled runtime output and `__VLS_*` keep their spelling.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

## PR body

````markdown
## Why

#11045 spelled generated Vue **types** as `$` plus the real name (`$Slots`, `$PropChecker`) but left every generated **value** as `__vize_snake_case`. Hovers, diagnostics, `--show-virtual-ts` and editor projections still showed names like `__vize_template_instance` and `__vize_prop_check_0_label`. They are long and hard to read, and their spelling was scattered across ~160 files.

## What

**One naming mechanism.** `vize_l0::ts_emit::generated_name` is now the only place a generated name is spelled. Each name is a `GeneratedName` constant (`TEMPLATE_INSTANCE`, `PROPS_NAV`, …). Producers capture it in a format string (`cstr!("{PROPS_NAV}{idx}")`), splice it in `ts!` / `emit_ts!` (`#TYPED_SLOTS`), or, where Rust needs a literal (`const` text, `concat!`), use `generated_name!(NAME)`. Consumers (content-mapper span features, unmapped-ignore blocks, directive expectations, maestro model links, rename and hover, patina markers) recognise names through `GeneratedName::is` / `is_member` / `strip` / `occurs_in` / `ends` and `generated_name::is_generated`, so they also accept a hygiene spelling (below).

**Spelling.** A value, property key, marker or parse probe is `$` plus a short camelCase name (`$inst`, `$args`, `$cb`, `$propsNav0`). A projection-local type is `$PascalCase` (`$EventsResolver3`). A family adds its index directly (`$propsNav0`), and any other tail after `_` (`$prop0_label`). Per review, names are short and plain: no `vize`, no redundant words. Words are added only to avoid a collision (`$refMap`, not `$refs`).

**Collision guard.**
- *Vue members.* No name or family stem equals a Vue instance member, template local or modelled plugin global (`$props $emit $slots $attrs $refs $el $parent $root $data $options $watch $forceUpdate $nextTick $event $route $router $t $i18n $nuxt $fetch …`). The `RESERVED` list and its unit test enforce this.
- *Authored bindings.* `hygiene::bind` takes the `$`-identifiers the SFC authors (script bindings, template scope aliases, template expressions, type declarations) once per projection, in `generated_names::bind`. A generated name that an authored identifier spells, or whose family it falls in, takes the first spelling with trailing `$`s that nothing authored uses: `$args` becomes `$args$`. Nothing else changes. An SFC without such an identifier pays one thread-local flag read per name. Generated static text pushed with a renamed name is rewritten by `hygiene::text`, which follows the `push_lib_text` precedent.
- `generated_name_hygiene.rs` (tsgo) checks an SFC that declares `const $args`, `const $cb` and `const $value = ref(...)`, uses them in component `@event` handlers, `@click` and `v-model`, and reads `$attrs`. Its diagnostics equal those of the same SFC with equal-length plain names: exactly one TS2345, with the authored name. Unit tests cover the vize_l0 table, recognition, nesting and restoring of the hygiene scope, and `ts!` hole lengths.
- `tests/tooling/generated-name-spelling.test.ts` fails on any new `__vize_*` spelling in production sources outside the external set below.

**Editor mapping.** Shorter names exposed a length-based tie-break in maestro. An authored span such as `@update:saveItem` maps both to its navigation key (`'update:saveItem'`, 15 bytes) and to the handler anchor (`$handler9_115`, 13 bytes). `mapping_for_source_offset` picked the shorter generated range. The old `__vize_handler_9_115` (20 bytes) always lost that tie, so `@update:` rename, definition and references went to the child model. The new 13-byte anchor won it, and they stayed on the handler. A generated-name anchor now ranks after every other row with the same authored length, so the outcome no longer depends on name length. This also fixes the same latent bug for event names longer than 20 bytes. The fix is covered by a unit test and by `lsp_named_model_transactions_cli`.

**Custody.** The byte-pinned producers are not hand-edited around the chain. `tests/_fixtures/tooling/generated-names-source.json` records every changed producer (before and after identity plus unique hunks). `generated-names-transition.ts` and `payload_update/generated_names.rs` invert it first in every existing source-custody layer, so older certificates keep authenticating their exact predecessor bytes. Frozen historical captures and custody records keep `__vize_*`, as #11045 kept `__Vize`.

**Unchanged on purpose** (separate evaluation, see the follow-up PR): published declaration output (`__vize_component__`), the macro shims the published `@vizejs/cli` `virtual-ts-globals.d.ts` declares (`__vize_defineProps`, …), file names and module specifiers (`__vize_helpers.d.ts`, `__vize_vue_modules.d.ts`, `__vize_external__`, …), compiled runtime output (`__vize_static_N`, `__vize_match`), and bundler-visible ids in `npm/builder`. `__VLS_*` stays too: `__VLS_PROPS_FALLBACK` is the global that vue-tsc-emitted library declarations probe, and `__VLS_StyleScopedClasses` is the name Vue Language Tools users write. Both are external contracts, not names vize generates.

## Performance

Measured on a 1,000-SFC corpus (`tools/benchmarks/scripts/generate.mjs 1000`) with `vize check -q`. Base (`main` d9e29d032c) and head were both built with `--profile ci-opt` on rustc 1.100.0, and the 15 runs were interleaved on a shared machine:

| phase | base median | head median | base min | head min |
|---|---:|---:|---:|---:|
| projection + mapping (`gen`) | 497.8 ms | 475.9 ms | 297.6 ms | 295.6 ms |
| collect | 36.3 ms | 31.5 ms | 23.6 ms | 24.4 ms |
| imports | 129.8 ms | 136.8 ms | 98.8 ms | 108.6 ms |

The tsgo phase dominates wall time and was too noisy on the shared host to compare: medians 4.1 s and 3.4 s, both dominated by load. Generated output (`--show-virtual-ts --format json`) went from **12,653,098 to 12,118,798 bytes (-534,300, -4.2%)**. Occurrences of `__vize_` in that output went from 56,300 to 10,000. The 10 left per SFC are the published names this PR deliberately keeps: `__vize_component__` ×2, the `__vize_helpers` reference and the seven macro shims.

## Name mapping (old → new, with lengths)

`N` is a numeric index; `x` is a tail kept after `_`.

| old | len | new | len |
|---|---:|---|---:|
| `__vize_template_instance` | 24 | `$inst` | 5 |
| `__vize_public_instance` | 22 | `$public` | 7 |
| `__vize_instance_globals` | 23 | `$globals` | 8 |
| `__vize_strict_template_context` | 30 | `$strictCtx` | 10 |
| `__vize_keyed_template_context` | 29 | `$keyedCtx` | 9 |
| `__vize_options_instance` | 23 | `$this` | 5 |
| `__vize_props` | 12 | `$propsObj` | 9 |
| `__vize_setup_props` | 18 | `$setupProps` | 11 |
| `__vize_options_props` | 20 | `$optionsProps` | 13 |
| `__vize_refs` | 11 | `$refMap` | 7 |
| `__vize_root_el` | 14 | `$rootEl` | 7 |
| `__vize_template` | 15 | `$template` | 9 |
| `__vize_template_slots` | 21 | `$templateSlots` | 14 |
| `__vize_typed_slots` | 18 | `$typedSlots` | 11 |
| `__vize_typed_emits` | 18 | `$typedEmits` | 11 |
| `__vize_typed_exposed` | 20 | `$typedExposed` | 13 |
| `__vize_exposed` | 14 | `$exposed` | 8 |
| `__vize_emits` | 12 | `$emits` | 6 |
| `__vize_emit_options` | 19 | `$emitOptions` | 12 |
| `__vize_template_emit` | 20 | `$templateEmit` | 13 |
| `__vize_emit_calls_nav` | 21 | `$emitCalls` | 10 |
| `__vize_forwarded` | 16 | `$forwarded` | 10 |
| `__vize_model_defaults` | 21 | `$modelDefaults` | 14 |
| `__vize_with_defaults_props` | 26 | `$defaultedProps` | 15 |
| `__vize_plain_script_exports` | 27 | `$scriptExports` | 14 |
| `__vize_src` | 10 | `$src` | 4 |
| `__vize_default` | 14 | `$default` | 8 |
| `__vize_css_module_global` | 24 | `$cssModule` | 10 |
| `__vize_typed_route_global` | 25 | `$typedRoute` | 11 |
| `__vize_resolved_runtime_default` | 31 | `$runtimeDefault` | 15 |
| `__vize_args` | 11 | `$args` | 5 |
| `__vize_cb` | 9 | `$cb` | 3 |
| `__vize_event` | 12 | `$evt` | 4 |
| `__vize_listener` | 15 | `$listener` | 9 |
| `__vize_model_value` | 18 | `$value` | 6 |
| `__vize_v` | 8 | `$item` | 5 |
| `__vize_jsx_expr__` | 17 | `$jsx` | 4 |
| `__vize_jsx_component__` | 22 | `$jsxComponent` | 13 |
| `__vize_jsx_component_slot__` | 27 | `$jsxSlot` | 8 |
| `__vize_jsx_component_spread__` | 29 | `$jsxSpread` | 10 |
| `__vize_emit_closed` | 18 | `$EmitClosed` | 11 |
| `__vize_emit_closed_factory` | 26 | `$EmitClosedFactory` | 18 |
| `__vize_emit_resolver` | 20 | `$EmitResolver` | 13 |
| `__vize_emit_input` | 17 | `$EmitGate` | 9 |
| `__vize_emit_native_input` | 24 | `$EmitNativeGate` | 15 |
| `__vize_template_use` | 19 | `$use` | 4 |
| `__vize_template_hover` | 21 | `$hover` | 6 |
| `__vize_model_event` | 18 | `$modelEvent` | 11 |
| `__vize_plain_host` | 17 | `$plainHost` | 10 |
| `__vize_template_handler` | 23 | `$handlerHost` | 12 |
| `__vize_spread` | 13 | `$spread` | 7 |
| `__vize_handler` | 14 | `$handlerBody` | 12 |
| `__vize_handler__` | 16 | `$handlerBody` | 12 |
| `__vize_generic` | 14 | `$typeParams` | 11 |
| `__vize_scope` | 12 | `$scope` | 6 |
| `__vize_runtime` | 14 | `$runtime` | 8 |
| `__vize_pragma` | 13 | `$pragma` | 7 |
| `__vize_value` | 12 | `$iterable` | 9 |
| `__vize_async_setup__` | 20 | `$asyncSetup` | 11 |
| `__vize_defaults__` | 17 | `$defaults` | 9 |
| `__vize_default__` | 16 | `$defaultValue` | 13 |
| `__vize_incomplete_member` | 24 | `$incomplete` | 11 |
| `__vize_unknown_reference` | 24 | `$unknown` | 8 |
| `__vize_props_nav_N` | 18 | `$propsNavN` | 10 |
| `__vize_slots_nav_N` | 18 | `$slotsNavN` | 10 |
| `__vize_slot_nav_N` | 17 | `$slotNavN` | 9 |
| `__vize_events_nav_N` | 19 | `$eventsNavN` | 11 |
| `__vize_kebab_events_nav_N` | 25 | `$kebabNavN` | 10 |
| `__vize_model_events_nav_N` | 25 | `$modelNavN` | 10 |
| `__vize_model_events_completion_N` | 32 | `$modelCompletionN` | 17 |
| `__vize_events_resolved_N` | 24 | `$resolvedEventsN` | 16 |
| `__vize_event_keys_N` | 19 | `$eventKeysN` | 11 |
| `__vize_prop_check_N` | 19 | `$propN` | 6 |
| `__vize_native_prop_check_N` | 26 | `$nativePropN` | 12 |
| `__vize_component_ref_check_N` | 28 | `$refCheckN` | 10 |
| `__vize_directive_check_N` | 24 | `$directiveN` | 11 |
| `__vize_unknown_directive_N` | 26 | `$unknownDirectiveN` | 18 |
| `__vize_slot_host_N` | 18 | `$slotHostN` | 10 |
| `__vize_slot_props_host_N` | 24 | `$slotPropsHostN` | 15 |
| `__vize_slot_props_N` | 19 | `$slotPropsN` | 11 |
| `__vize_slot_scope_N` | 19 | `$slotScopeN` | 11 |
| `__vize_slot_payload_N` | 21 | `$slotPayloadN` | 13 |
| `__vize_slot_name_N` | 18 | `$slotNameN` | 10 |
| `__vize_slot_children_N` | 22 | `$slotChildrenN` | 14 |
| `__vize_required_slots_N` | 23 | `$requiredSlotsN` | 15 |
| `__vize_v_for_source_N` | 21 | `$forSourceN` | 11 |
| `__vize_match_N` | 14 | `$matchN` | 7 |
| `__vize_handler_N` | 16 | `$handlerN` | 9 |
| `__vize_inline_handler_N` | 23 | `$inlineHandlerN` | 15 |
| `__vize_emit_factory_N` | 21 | `$emitFactoryN` | 13 |
| `__vize_emit_props_N` | 19 | `$emitPropsN` | 11 |
| `__vize_global_component_N` | 25 | `$globalComponentN` | 17 |
| `__vize_native_key_N` | 19 | `$nativeKeyN` | 11 |
| `__vize_fragment_prop_N` | 22 | `$fragmentPropN` | 14 |
| `__vize_ref_N` | 12 | `$refN` | 5 |
| `__vize_generic_N` | 16 | `$genericN` | 9 |
| `__vize_dynamic_is_N` | 19 | `$isN` | 4 |
| `__vize_setup_import_N` | 21 | `$importN` | 8 |
| `__vize_exported_type_N` | 22 | `$exportedType_x` | 15 |
| `__vize_ambient_N` | 16 | `$ambientN` | 9 |
| `__vize_shipped_default_N` | 24 | `$shippedDefaultN` | 16 |
| `__vize_computed_N` | 17 | `$computed_x` | 11 |
| `__vize_method_N` | 15 | `$method_x` | 9 |
| `__vize_boolean_target_N` | 23 | `$booleanTarget_x` | 16 |
| `__vize_patina_promise_N` | 23 | `$promiseN` | 9 |
| `__vize_patina_prop_N` | 20 | `$propShapeN` | 11 |
| `__vize_patina_emit_N` | 20 | `$emitCheckN` | 11 |
| `__vize_patina_reactivity_N` | 26 | `$reactivityN` | 12 |
| `__vize_events_resolver_N` | 24 | `$EventsResolverN` | 16 |
| `__vize_legacy_props_N` | 21 | `$LegacyPropsN` | 13 |
| `__vize_legacy_prop_N` | 20 | `$LegacyPropN` | 12 |
| `__vize_forwarded_props_N` | 24 | `$forwardedPropsN` | 16 |
| `__vize_forwarded_emits_N` | 24 | `$forwardedEmitsN` | 16 |
| `__vize_vnode_hook_N_listener` | 28 | `$VnodeHookListenerN` | 19 |
| `__vize_vnode_hook_N_handler` | 27 | `$VnodeHookHandlerN` | 18 |
| `__vize_transition_N_listener` | 28 | `$TransitionListenerN` | 20 |
| `__vize_transition_N_handler` | 27 | `$TransitionHandlerN` | 19 |
| `__vize_dynamic_component_N_listener` | 35 | `$DynamicListenerN` | 17 |
| `__vize_dynamic_guard_N_not` | 26 | `$isNotGuardN` | 12 |
| `__vize_dynamic_guard_N` | 22 | `$isGuardN` | 9 |
| `__vize_resolved_X_props_N` | 25 | `$resolvedProps_X_N` | 18 |
| `__vize_selected_X_props_N` | 25 | `$selectedProps_X_N` | 18 |

## Tests

Not required, and failing by design: the projection workflows `unaffected-projections`, `unchanged-projection-packets`, `published-sdk-packets`, `compare` (×2), `parity`, `admission` and `real-beta-host`. Each asserts that the complete generated bytes equal the base commit's bytes for its fixture set. A rename of every generated value changes those bytes everywhere.

- `cargo test -p vize_l0 --lib generated_name`: table, reserved members, family overlap, recognition, hygiene scope nesting and restore, `ts!` hole length.
- `cargo test -p vize_canon --lib`, plus `--test empty_vue_star_exports`, `--test component_contract_strict_battery`, `-p vize_maestro --test davinci_ts40_projection`, and the patina and maestro libs. All run with `TSGO_PATH` set, so the new `generated_name_hygiene` tsgo test ran and passed. Insta snapshots were regenerated. Failures that also occur on `main` locally are left alone; they come from macOS `/tmp` → `/private/tmp` canonicalization and a missing element-plus submodule.
- Custody: `generated-names-source.json` (inverse layer in TS and Rust), a membership successor (`malformed-membership/generated-names-generated.json`, which certifies that A.vue.ts is exactly the respelled predecessor), the maestro `successor-generated-names` projection receipt, updated ecosystem-producer hashes and the strict-battery message.
- Tooling: emit-props gate, callback-owner, generic-event and helper-ABI tests now read producers through the custody chain or the generated-name table. The assertion lint and its ratchet are updated.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
````
