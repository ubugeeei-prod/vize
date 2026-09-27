# Inspector comparison subprocess outcomes

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
The historical diagnostic fixture from
[#1915](https://github.com/ubugeeei-prod/vize/issues/1915) exposed a pipe race
in #6924's real Rust shard: an early Node exit replaced the intended missing
Vue compiler diagnostic with a generic input-write failure.

Always collect and reap the official compiler subprocess after an input-write
failure. When Node fails, preserve its actual exit status and existing stderr
normalization; when Node succeeds but did not receive its complete input,
retain the transport failure before accepting or parsing stdout.

Put the comparison function move in its own move-only commit. Deterministic
CLI fixtures close Node stdin immediately and use an SFC input larger than
pipe capacity: missing compiler status 17, generic compiler status 23 and a
successful child with undelivered input. Keep the original small missing-module
fixture and every assertion. These fixtures exercise the source CLI and shell
process, with no retry, sleep, skip or relaxed assertion.
The CLI regression corpus adds `fixtures/inspector_compare/early_exit.vue.txt`,
mapped to the actual temporary `App.vue` without changing authored Vue census
membership. Tests expand its comment payload at runtime instead of committing
megabytes. Missing-module and generic failure fixtures pin complete stderr
bytes, actual exit status and empty stdout.

Normal compiler output and supported legacy behavior remain unchanged. This
repairs failure reporting and process cleanup; it does not implement Vue 2
comparison, a native compiler path or any level migration.
