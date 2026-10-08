# Patterned runtime evidence custody

Issue: [#7951](https://github.com/ubugeeei-prod/vize/issues/7951). This supplements
[the first failure and diagnostic decision](./2026-10-05-patterned-runtime-failure-evidence.md).
The historical cause remains **unknown**. Later successful runs do not resolve it.

## Recovered first-run evidence

On 2026-10-08, the original raw source-coverage log was downloaded again from
[Check 37256849378, job 111595669351](https://github.com/ubugeeei-prod/vize/actions/runs/37256849378/job/111595669351).
It contains 776,494 bytes, and its SHA-256 matches the original retained receipt:
`c9fe7938c49677fa79170f0ad0e4e438bbb4b83febc2c2fbf2a089367e0a08e1`.
The API log, rather than the differently prefixed `gh run view --log` rendering,
is the hash source. It can be recovered with:

```sh
gh api --allow-escape-sequences repos/ubugeeei-prod/vize/actions/jobs/111595669351/logs > original-source-coverage-raw.log
```

The committed [original Rust producer](https://github.com/ubugeeei-prod/vize/blob/256cb6c0e35804bf714ab8c3d1e299b58bb08a0c/crates/vize_atelier_vapor/tests/patterned_template_runtime.rs)
and [Node runner](https://github.com/ubugeeei-prod/vize/blob/256cb6c0e35804bf714ab8c3d1e299b58bb08a0c/tests/tooling/support/patterned-template-runtime.mjs)
define the inputs. The coverage command runs the unfiltered workspace through
`cargo llvm-cov --workspace --json --summary-only`; the recorded Cargo invocation
was `cargo test --tests --manifest-path /home/runner/_work/vize/vize/Cargo.toml
--target-dir /home/runner/_work/vize/vize/target/llvm-cov-target --workspace`.
The failing binary retained all 24 tests: 23 passed, 1 failed, 0 ignored and
0 filtered. Its Cargo exit was 101.

| Evidence | What the original receipt establishes |
| --- | --- |
| Source | `256cb6c0e35804bf714ab8c3d1e299b58bb08a0c` |
| Test | `match_preserves_hosts_inside_authored_loop_scopes` |
| Backend | VDOM, the first backend in the original loop |
| Child result | `wait_with_output` completed and `status.success()` was false |
| Stderr | The panic prints empty stderr before `code:` |
| Child exit code / signal | Neither was printed or uploaded |
| Child stdout | Not printed or uploaded; it must not be reconstructed as empty |
| Node setup | The log records Node 24.14.0; the child's resolved executable path was not logged |
| Child working directory | Inherited, with no explicit override or recorded child cwd |
| Runner | `blacksmith-32vcpu-ubuntu-2404-Runner-c03cc1b25c` |

The source-defined command is `node` with one argument:
`/home/runner/_work/vize/vize/crates/vize_atelier_vapor/../../tests/tooling/support/patterned-template-runtime.mjs`.
Its stdin is JSON containing `backend`, `code` and `cases`, written without an
added newline. `spawn`, the stdin write and `wait_with_output` did not trigger
their earlier assertions. This establishes a nonzero child result, rather than
a source-coverage percentage failure. It does not distinguish a runtime defect,
a pending operation, a signal, resource termination or another child failure.

The original run exposes 16 unexpired artifacts, each from other jobs. The
source-coverage summary verification and upload were skipped after failure.
No retained artifact contains this child's original exit/signal/stdout or phase
receipt. Those missing fields cannot be recovered from a later successful run.

## Original values, preserved without a replay substitution

The authored source is unchanged:

```vue
<section v-for="item in items" :key="item.id" :data-id="item.id" v-match="item.state"><p v-when="'ready'">Ready</p><p v-when="_">Other</p></section>
```

The one case starts with `(1, ready), (2, waiting)`, then reorders and changes
states to `(2, ready), (1, waiting)`. The complete original expected trees are
preserved by this JSON, including the authored `section` hosts and `p` children:

```json
[
  {
    "context": {
      "items": [
        {
          "id": 1,
          "state": "ready"
        },
        {
          "id": 2,
          "state": "waiting"
        }
      ]
    },
    "steps": [
      {
        "patch": {
          "items": [
            {
              "id": 2,
              "state": "ready"
            },
            {
              "id": 1,
              "state": "waiting"
            }
          ]
        }
      }
    ],
    "trees": [
      [
        {
          "attributes": {
            "data-id": "1"
          },
          "children": [
            {
              "attributes": {},
              "children": [
                "Ready"
              ],
              "tag": "p"
            }
          ],
          "tag": "section"
        },
        {
          "attributes": {
            "data-id": "2"
          },
          "children": [
            {
              "attributes": {},
              "children": [
                "Other"
              ],
              "tag": "p"
            }
          ],
          "tag": "section"
        }
      ],
      [
        {
          "attributes": {
            "data-id": "2"
          },
          "children": [
            {
              "attributes": {},
              "children": [
                "Ready"
              ],
              "tag": "p"
            }
          ],
          "tag": "section"
        },
        {
          "attributes": {
            "data-id": "1"
          },
          "children": [
            {
              "attributes": {},
              "children": [
                "Other"
              ],
              "tag": "p"
            }
          ],
          "tag": "section"
        }
      ]
    ]
  }
]
```

The panic preserves this entire generated VDOM code display:

```js
const { openBlock: _openBlock, createElementBlock: _createElementBlock, Fragment: _Fragment, createCommentVNode: _createCommentVNode, renderList: _renderList } = Vue

function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.items, (item) => {
    return (_openBlock(), _createElementBlock("section", {
      key: item.id,
      "data-id": item.id
    }, [
      (() => { const __vize_match_0 = ((__vize_match_0_0) => { a0: { if (__vize_match_0_0 !== 'ready') break a0; return [0]; } return [1]; })(item.state); return (__vize_match_0[0] === 0)
        ? (_openBlock(), _createElementBlock("p", { key: 0 }, "Ready"))
        : (__vize_match_0[0] === 1)
          ? (_openBlock(), _createElementBlock("p", { key: 1 }, "Other"))
        : _createCommentVNode("v-if", true) })()
    ], 8 /* PROPS */, ["data-id"]))
  }), 128 /* KEYED_FRAGMENT */))
}
```

Reconstructed UTF-8 files without an added terminal newline have these hashes:

| Value | Bytes | SHA-256 |
| --- | ---: | --- |
| Authored source | 148 | `a3528c82bd2760a7f453f5a9f741895da46da0c21440d9b7c94c9613cc93f33f` |
| Cases, compact sorted-key JSON | 611 | `23458a0d015079f7cec22d40319b6f4c8b88b668dedf325decc338dd41c01e40` |
| Generated code display | 935 | `6a9162009f350614ccce894f6129b8233f78bcb4ffd4e71cd3723044b918bc84` |
| Reconstructed stdin values | 1,614 | `2f401a9f19ad2f6b521140226dcfe70befbf2993795e1258fdf3df983b8eaae4` |

These are source/log reconstructions, not an original captured stdin stream.
The original write bytes and the generated string's final line ending were not
independently recorded. Do not give the reconstruction stronger custody credit.

## Independent diagnostic gap and correction

The current Rust reporter decodes stdout/stderr with `String::from_utf8_lossy`.
Two genuine Node children emitting first byte `255` and `254` both exit 13,
but produce identical decoded text. Their stdout bytes are respectively
`[255, 0, 10, 97]` and `[254, 0, 10, 97]`; stderr bytes are `[255, 13, 27]` and
`[254, 13, 27]`. The text loses the byte identity, including in the failure report.
This is a reproducible diagnostic defect; it is not the original coverage cause.

Retain readable text and add complete `stdout_bytes` and `stderr_bytes` arrays
on every existing failure path. The new genuine-child law checks both outputs
and the entire expected report for both byte values, including NUL, newline,
carriage return and escape. Existing silent-exit, signal and protocol laws retain
whole-report expectations. Successful protocol output and every original
patterned source/case/backend/tree/update/read-count/compiler option remain
unchanged. No fixture is ignored and no compiler behavior is weakened.

A focused extraction of the actual reporter and new law, using serde_json
1.0.149, Node 24.14.0 and local Rust 1.98.0, failed before the correction and
passes after it. This is local diagnostic evidence, not exact-head hosted
qualification or historical cause resolution. The repository's Rust 1.99.0
Actions and protected queue checks must pass at the actual PR head.

## Remaining acceptance

Keep #7951 open while the original failure cause is unknown. The fresh manual
[Check 37695376731](https://github.com/ubugeeei-prod/vize/actions/runs/37695376731)
at `fd6241bf8ea5466794cc955138a59a9b75a8aac2` passed the original runtime case
and all 27 existing tests without skips, but establishes only that replay's
success. Preserve this successful result and the first failure independently.
A future genuine failure must preserve its complete child input, status, streams
and phase, then receive a source-bound diagnosis and necessary correction.
Do not change release candidates, infer a historical cause from a controlled
child, or close this P0 on diagnostic-green evidence alone.
