# Type-checking performance

Choose the number of checker workers in each Corsa process:

```sh
vpx vize check --checkers 4 --servers 1
```

`--checkers` accepts a positive integer and overrides `VIZE_CHECKERS`. Without either setting, Vize uses one checker worker. `--servers` controls separate Corsa processes; automatic server sizing accounts for the number of checker workers in each process.

The one-worker default preserves the diagnostic behavior established in [#3905](https://github.com/ubugeeei-prod/vize/issues/3905). Wider Corsa pools produced missing and different diagnostics in that comparison. Opt into a wider pool when that tradeoff is acceptable for your workflow.

Rust callers can use `BatchTypeChecker::set_checker_count(Some(4))` or `CorsaExecutor::set_checker_count(Some(4))` without changing process-wide environment variables. Passing `None` restores the environment/default selection.

To inspect where a run spends its time:

```sh
vpx vize check --profile
```

Alias dependency collection shares Vue-free closure results within each run. Changes are read again on the next collection.
