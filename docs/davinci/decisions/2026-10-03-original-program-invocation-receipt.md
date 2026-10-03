# Original Program invocation observation

Issues: [#6838](https://github.com/ubugeeei-prod/vize/issues/6838),
[#6849](https://github.com/ubugeeei-prod/vize/issues/6849),
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

`ScriptUnit::has_invocations` reads the existing private original-Program
observation. The genuine File provider records Call, New, TaggedTemplate and
Import expressions in its existing declaration/reference walk before invoking
observers. This readonly getter adds no field, parser call, traversal or stage.

The observation remains true after subtree rollback or callback unwind. False
on an incomplete File does not establish absence: a callback may interrupt the
walk before a later invocation. Callers must independently require complete
File analysis and any native source/custody capabilities they consume. The
getter grants neither completion nor native admission authority.

The Canon native whole-Program adapter needs this conservative observation to
refuse invocations until an actual provider owns authored filename authority.
It must use the genuine File-provider ancestry; the observation is not copied
onto an unrelated baseline. Static import and source-export rows remain separate
facts. This prerequisite does not replace the legacy checker, close its
fix-history gate, or supply a Vue/product integration.

The fixture uses actual stock-parser admission and the same File producer once
per original Program. It covers completed absence, all four invocation forms,
callback unwind, subtree rollback, and interruption before the invocation.
Exact-head Actions and protected merge validation remain required for delivery.
