# Local AI Fix Agent

This local runner watches GitHub fix requests from your machine, runs a local agent, opens a draft PR, and waits for CI.

It intentionally avoids GitHub Actions for the agent execution path. GitHub Actions still run as the normal PR CI after the local runner pushes a branch.

## Requirements

- `gh` authenticated for the target repository.
- `git` configured for pushing to the repository remote.
- `codex` CLI authenticated, or a custom `AI_FIX_AGENT_COMMAND`.

## Commands

Process one fix request:

```sh
node tools/ai-fix-agent.mjs run --fix 123 --repo ubugeeei-prod/vize
```

Watch newly opened fix requests:

```sh
node tools/ai-fix-agent.mjs watch --repo ubugeeei-prod/vize --interval 300
```

Also process already-open fix requests when the watcher starts:

```sh
node tools/ai-fix-agent.mjs watch --repo ubugeeei-prod/vize --include-existing
```

By default the runner uses:

```sh
codex exec --full-auto --cd <repo-root> -o <result-file> -
```

To use another agent, set a command that reads the prompt from stdin or from `$AI_FIX_AGENT_PROMPT_FILE`:

```sh
AI_FIX_AGENT_COMMAND='my-agent --prompt-file "$AI_FIX_AGENT_PROMPT_FILE"' \
  node tools/ai-fix-agent.mjs watch --repo ubugeeei-prod/vize
```

The runner writes transient state under `.git/ai-fix-agent-state.json` and `.git/ai-fix-agent/`.

## Behavior

- Newly opened fix requests are processed by default. `--include-existing` opts into old open fix requests.
- Generated branches use `ai-agent/fix-<number>`.
- PR titles are conventional. Any leading `[codex]` is removed from the fix title.
- The runner creates draft PRs and waits for `gh pr checks` unless `--no-wait-ci` is passed.
- Fix request text is treated as untrusted input in the prompt; the local agent must not handle secrets from fix content.
