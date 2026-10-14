# Bare early returns in computed getters (#8828)

Issue: [#8828](https://github.com/ubugeeei-prod/vize/issues/8828). Supersedes the bare-return clause of [the return-flow record](./2026-10-13-8828-computed-return-flow.md); its completion analysis is unchanged.

The return-flow change rejected every reachable bare `return;` in a computed getter. eslint-plugin-vue 10.9.2 does not: with the default `treatUndefinedAsUnspecified`, a getter is valid when every path ends in `return` or `throw` and the getter's last `return` in source order has a value. The common guard below is therefore clean upstream:

```ts
const src = computed(() => {
  if (iconSource.value?.type !== "file") return;
  return iconSource.value.src;
});
```

The v0.452.0 Real Project Matrix (run 38515301568) recorded 87 `script/return-in-computed-property` false positives of this shape across 26 projects. `script/return-in-computed-property` now uses the reference rule's condition. A getter that can fall through, or whose last `return` is bare (`try { return 1 } finally { return; }`), is still reported.

`crates/vize_patina/tests/computed_return_flow_history.rs` now also replays all 18 recorded eslint-plugin-vue controls in `tests/_fixtures/differential/linter/computed-return-flow-8828/flow-reference.json` and requires the same reported/clean result for each.
