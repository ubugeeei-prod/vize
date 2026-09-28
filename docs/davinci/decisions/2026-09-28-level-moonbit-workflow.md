# MoonBit workflow name (#6832)

- The active MoonBit CI workflow is `level-moonbit.yml`, displayed as “Level MoonBit”. Its self-trigger path, concurrency group, and cache key use the same level name. The job name, triggers, toolchain, commands, and fixture bytes stay as they were.
- The change is replayable with `vp node tools/support/compat/levels/rename-moonbit-workflow.mjs` after the move-only commit. The script rejects ambiguous references and is safe to re-run on current `main` after a conflict.
- The current `main` ruleset requires `check-js`, `check-vize-apps`, `fmt-rust`, and `test-report`; none is the MoonBit job. The existing `davinci` branch selector is left in the trigger because changing branch coverage is a separate CI decision.
- The historical MoonBit run link in `docs/davinci/plan/exprref-validation.md` retains its original displayed workflow name. This slice does not rename the remaining Davinci tooling/workflow paths or finish #6832.
