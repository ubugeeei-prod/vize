# Release integration synthetic merge refresh

Paired issues: [#6239](https://github.com/ubugeeei-prod/vize/issues/6239) and
[#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Observed refusal

The fresh v0.437.0 source is PR #8314 at
`b303befe31132cd559dbe02eb3d45be11f63fbf8`, cut from actual main
`04ec4c43d9c84c46892fc33f013917d2561bfe78`. Its official Release run
is [37774663155](https://github.com/ubugeeei-prod/vize/actions/runs/37774663155).
Version integration PR #8315 retains head
`f54fc98c345b5f00ee4973e2885068e96360c49f`.

The first integration event was rejected because its body/base snapshot no
longer matched the live receipt. A controlled close/reopen retained the complete
body, head, draft state and base branch. Fresh Check
[37774831137](https://github.com/ubugeeei-prod/vize/actions/runs/37774831137),
job `113303918486`, then refused the current synthetic merge identity before
the catalog comparison:

| Receipt                                            | Commit                                     |
| -------------------------------------------------- | ------------------------------------------ |
| Exact reopened event and checked-out candidate     | `ad3174004749406384c488dbc28fc4eea39db11a` |
| Later REST merge commit and `refs/pull/8315/merge` | `eb8d2897ee4acd68fec8d56c0dc920766ed5c214` |

Both genuine commits have the ordered parents
`04ec4c43d9c84c46892fc33f013917d2561bfe78`,
`f54fc98c345b5f00ee4973e2885068e96360c49f`, and complete tree
`9d8e9b7b6df4ac8690eccb21a0533d6a1d496455`. GitHub regenerated the
synthetic commit without changing either parent or any checked-out byte.
Retrying the original event retains its original candidate SHA.

## Bounded correction

For a genuine pull-request event only, authenticate the current API synthetic
commit through Git and require the same complete ordered two-parent vector and
complete tree as the original event candidate. The original event SHA remains
the checked-out and tested identity. Changed PR head, base, body, parent vector
or tree remains fatal. Local receipts still require the current exact synthetic
SHA. Merge-group candidate SHA, own queue ref and base checks are unchanged.

The generated version delta and complete source/candidate shipping catalogs
remain exact. The original v0.436.0 script-drift refusal stays covered. Genuine
Git merge-commit controls exercise identical-tree/parent refresh, changed trees,
reordered, omitted and additional parents, and the separate queue/local laws.
The Rust entry source pin advances so normal cached drivers compile the changed
support module.

The source C/H, durable pin, R, artifacts, publication requirements and original
failed receipts remain unchanged. This repair must actually merge before a
fresh integration event can qualify against it. No metadata admission, tag,
publication, installed-package or issue-completion credit is granted by this
preparation.
