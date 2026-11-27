# 2026-11-26 Nuxt mode-suffix lint snapshot review

This archive extends, rather than replaces, the 2026-10-21, 2026-10-27 and
2026-11-06 reviews. It binds the elk and npmx.dev lint snapshots after
`vue/component-definition-name-casing` stopped reporting Nuxt mode suffixes
(`.client`, `.server`, `.global`, `.island`) to their complete predecessors at
`b7dfe22573ae4b35a08e292efb4d68ec335222eb`. The other five snapshots, the
fixture Git pins and the ordered file packets are unchanged.

`snapshots.json.gz` retains both complete before and after snapshots, both
whole-file hashes, every changed packet hash, message totals, and field-level
classifications. `inventory.json` authenticates that archive. The loader keeps
the three prior review boundaries intact, then admits this boundary only when
the checked-in snapshots, archived bytes, and fixture pins match exactly.

The admission policy accepts one kind of change and rejects every other:

- A packet may lose only its `vue/component-definition-name-casing` finding,
  with the exact message for its own file stem, and only when the stem ends in
  a Nuxt mode suffix and the name before the suffix is PascalCase or
  kebab-case. Such a file keeps no casing finding.
- Every other message stays byte for byte and in order; nothing is added.
- Every non-message field stays, and the warning count drops by exactly the
  number of removed findings.
- elk removes 7 findings and npmx.dev removes 12. Dotted names that are not
  Nuxt modes, such as npmx.dev's `*.takumi.vue` and `*.unused.vue`, stay
  reported.
