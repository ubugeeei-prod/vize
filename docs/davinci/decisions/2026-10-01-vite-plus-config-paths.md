# Preserve Vite+ temporary config scopes

Issue: [#7307](https://github.com/ubugeeei-prod/vize/issues/7307).

Keep transient task configs in OS temp, with private permissions and guaranteed
cleanup. Rebase top-level ignore globs to the project directory and make each
normalized entry's basePath absolute before serialization. Entry patterns,
negations, ignores, rule overrides, and order retain their meaning. Source
config objects remain immutable. Inline and file-backed task configs share
this path; config discovery currently searches the task's working directory.

Normalize current-directory components in the native lint ignore base so
`--config ./vize.config.json` agrees with `--config vize.config.json`.

JS tests inspect the serialized config received by the native command and
verify its cleanup. The authored CLI fixture executes project, dot-component,
and external-temp config forms and requires identical complete JSON reports:
the generated file is excluded and the legacy entry disables alt-text, while
the ordinary image retains its warning. Actions validates the native CLI.
