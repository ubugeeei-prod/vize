# Regenerate v-on evidence from the current committed source corpus

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Full qualification reported a missing Maestro shard and a stale Patina shard
in the natural v-on corpus. At repair head
`e4ad440e0b53224952d97cb731621cc0ba3ee513`, the saved inventories were still
byte-identical to main `84e34e54d61297b1a43b171242e0b92fa1ca25cc`.
Those inventories had not incorporated the new natural spellings from
[#9212](https://github.com/ubugeeei-prod/vize/pull/9212),
[#9213](https://github.com/ubugeeei-prod/vize/pull/9213) and the original project
fixtures in [#9219](https://github.com/ubugeeei-prod/vize/pull/9219).

Run the existing generator after the accompanying rule-documentation repair:

```sh
rust-script tools/commands/davinci/v-on-corpus.rs --write
rust-script tools/commands/davinci/v-on-corpus.rs --check
```

Review only the generated per-area TSV differences. They retain the current
source paths, natural modifier spellings and repeated occurrences, including
the moved `valid_v_on` test module, original project fixtures, Maestro option
tests, the numeric-key acceptance probe and complete generated documentation.
No product implementation, corpus classifier, storage capacity or budget is
changed. The existing gate still requires exact whole-inventory equality and
maximum option, event and key buckets of two entries each; synthetic boundary
fixtures remain excluded by the existing scanner.

After the documentation and virtual-list inputs were frozen, regeneration on
the working tree based on `bf049a1a7a67537aae04c47512f58eb023596059` changed five
TSV shards only. All 15 existing tests in `davinci-v-on-storage.test.ts` and
`davinci-matrices.test.ts` passed, including whole-inventory equality, unchanged
two-entry maxima and the matrix mutation controls. Full Actions qualification
remains the integration acceptance.

Full run [38988499789](https://github.com/ubugeeei-prod/vize/actions/runs/38988499789)
at `e2b883eed6378eb7ada73bf0220d89a185a3d6fd` found one later stale shard.
[#9274](https://github.com/ubugeeei-prod/vize/pull/9274) added five complete
numeric-key autofix outputs to `lint-deprecation-source-composition.test.mjs`.
The unchanged generator records their five natural spellings in
`tests--tooling.tsv`, preserving every previous row. Regenerate that shard from
the actual source fixtures; the classifier, two-entry maxima, original packets
and whole-inventory assertions remain unchanged.
Both existing test files pass all 15 controls again, including the unchanged
capacity and mutation checks. Exact-source Actions remain the integration gate.
