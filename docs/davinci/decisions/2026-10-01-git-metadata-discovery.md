# Git metadata source exclusion

Issue: [#7326](https://github.com/ubugeeei-prod/vize/issues/7326).

Source discovery excludes any path component named `.git` before descending
into directories or selecting explicit files. This rule applies independently
of Git ignore settings and explicit hidden-directory selection. A repository's
`.github` directory and ordinary names containing `.git` remain eligible under
each command's existing input rules.

The formatter's absolute and parent-relative glob routes now use the same
prunable walker as relative globs, retaining their existing ignore-policy
choice. Inspector generic globs also use a walker rather than expanding paths
through Git metadata and discarding the results afterward.

The shared lexical boundary lives in L0 path utilities. CLI fmt, lint, check,
tsconfig ownership, build, inspector and doctor; Nuxt source/generated scans;
and LSP global-component, rename and workspace Vue discovery use it. Walked
files do not require an additional canonicalization syscall per file.

The formatter CLI corpus places malformed Vue text in a nested
`.git/worktrees` directory and confirms that relative and absolute globs format
authored sources and `.github` files without reading or rewriting the metadata.
Collector regressions cover directory roots, explicit metadata files, explicit
metadata directories, glob patterns and tsconfig `files`/includes.

Validation uses exact-head Actions and the protected merge queue. Git commands
and Git ignore-policy reads retain their normal metadata access; this decision
concerns product source discovery.
