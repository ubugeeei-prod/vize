# AI Issue Agent

This directory supports the `AI Issue Agent` GitHub Actions workflow.

## Required secrets

- `OPENAI_API_KEY`: API key used by `openai/codex-action`.
- `AI_AGENT_GITHUB_TOKEN`: a fine-grained PAT or GitHub App installation token that can push branches, create pull requests, and comment on issues.

For a fine-grained PAT, grant repository access with `Contents: Read and write`, `Pull requests: Read and write`, `Issues: Read and write`, and read access for checks/actions metadata. If the agent is expected to edit workflow files, the token also needs workflow-file write permission.

Do not use the default `GITHUB_TOKEN` for `AI_AGENT_GITHUB_TOKEN`. GitHub does not start normal `push` or `pull_request` workflow runs for most events created by `GITHUB_TOKEN`, so the agent PR would be opened without the usual CI signal.

## Trigger policy

- Every newly opened issue runs the agent.
- Adding the `ai-agent` label reruns the agent for an existing issue.
- The workflow can also be run manually with `workflow_dispatch` and an issue number.

This intentionally allows untrusted issue text to trigger Codex. The prompt treats issue content as untrusted input, but API usage should still be monitored and rate-limited at the account or repository level if abuse appears.

The agent creates or updates a draft PR from `ai-agent/issue-<number>`, uses a conventional PR title, and waits for PR checks before reporting back to the issue.
