# Local AI Issue Agent

This local runner watches GitHub Issues from your machine, runs a local agent, opens a draft PR, and waits for CI.

It intentionally avoids GitHub Actions for the agent execution path. GitHub Actions still run as the normal PR CI after the local runner pushes a branch.

## Requirements

- `gh` authenticated for the target repository.
- `git` configured for pushing to the repository remote.
- `codex` CLI authenticated, or a custom `AI_ISSUE_AGENT_COMMAND`.

## Commands

Process one issue:

```sh
node tools/ai-issue-agent.mjs run --issue 123 --repo ubugeeei-prod/vize
```

Watch newly opened issues:

```sh
node tools/ai-issue-agent.mjs watch --repo ubugeeei-prod/vize --interval 300
```

Also process already-open issues when the watcher starts:

```sh
node tools/ai-issue-agent.mjs watch --repo ubugeeei-prod/vize --include-existing
```

By default the runner uses:

```sh
codex exec --full-auto --cd <repo-root> -o <result-file> -
```

To use another agent, set a command that reads the prompt from stdin or from `$AI_ISSUE_AGENT_PROMPT_FILE`:

```sh
AI_ISSUE_AGENT_COMMAND='my-agent --prompt-file "$AI_ISSUE_AGENT_PROMPT_FILE"' \
  node tools/ai-issue-agent.mjs watch --repo ubugeeei-prod/vize
```

The runner writes transient state under `.git/ai-issue-agent-state.json` and `.git/ai-issue-agent/`.

## Behavior

- Newly opened issues are processed by default. `--include-existing` opts into old open issues.
- Generated branches use `ai-agent/issue-<number>`.
- PR titles are conventional. Any leading `[codex]` is removed from the issue title.
- The runner creates draft PRs and waits for `gh pr checks` unless `--no-wait-ci` is passed.
- Issue text is treated as untrusted input in the prompt; the local agent must not handle secrets from issue content.
