# Project initialization reader flow

Issues: #8365 and #8372. Configuration and released setup acceptance in
#8370 and #8371 remain separate and unfinished.

## Decision

Provide a complete authored Japanese `guide/init.md`, with the same detection,
feature selection, preview, application, recovery and next-step flow as English.
Correct the English explanations where the current CLI source differs from the
old prose. This changes documentation rather than CLI behavior or defaults.

`npm/cli/src/init.ts` resolves feature selection before its dry-run return, so a
non-TTY preview needs automatic confirmation. Explicit positive feature flags
override their detected defaults without disabling unmentioned features.
`init/plan-project.ts` creates a missing TypeScript configuration when selected;
an existing configuration is retained. `--no-install` still writes planned files.

A blocked feature does not cancel unrelated planned edits. Files are written
before dependency commands, and a later failure may leave earlier changes.
Readers must check each feature result and the actual changes before continuing.
`init/lint-target.ts` preserves a readable existing Oxlint configuration. A lint
block that cannot accept an initial Vize integration requires manual editing;
an already detected Vize bridge remains unchanged. The original refusal output
is explicitly presented as the initial-adoption example.

Retain all nine original complete code and output fences and eight ordered URL
destinations. Both articles preserve all 117 original inline occurrences and add
only three contextual occurrences: the project package file, the Vite-specific
example qualification and the valid unattended preview command. Preserve all
nine English bookmark aliases in Japanese. Existing configuration guides continue
to state release availability; source inspection provides no installed-host or
whole toolchain setup acceptance.

## Qualification and remaining work

The existing translation collector must retain the complete authored Japanese
body, all original recipes, finite inline contracts, URLs and bookmarks.
Keep every pre-existing collector law and all unrelated coverage entries.
The existing revised More-guides mechanism covers English and Japanese.

Delivery requires exact-head source Actions, the full protected merge queue,
actual integration, the natural official Docs build and source-coupled publication.
Then check the actual EN/JA desktop/mobile reading flow, recipes, fonts, links and
recovery guidance. These public checks are pending at source review. Do not infer
#8370/#8371 completion, installed CLI execution or a performance gain from prose.
