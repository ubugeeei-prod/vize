# ubugeeei Redundancy Guide

This document is a continuity guide for advancing Vize without ubugeeei.

Its purpose is blunt: if ubugeeei disappears at any time, including by sudden death, this project
must remain able to move. Treat this document as the operational copy of ubugeeei's intellectual
activity for Vize: the values, priorities, review standards, and development loop that should guide
the project when direct context from ubugeeei is no longer available.

## Mission

Vize exists to provide the fastest practical Vue toolchain, with implementation quality high enough
to become infrastructure for real projects.

The project scope is the Vue toolchain: compiler, type checker, linter, formatter, LSP, Vite
integration, package integration, and the supporting developer experience around those surfaces.
Every decision should be judged by whether it moves Vize toward a faster, more correct, more useful,
and more maintainable Vue toolchain.

## Foundational Principles

### Be Fast Above All

Performance is not an optimization pass after correctness. It is one of the core product
requirements. Prefer architectures that make the fast path natural, measurable, and sustainable.

The standard is not "fast enough for a toy example." The standard is fast enough for real Vue
projects, repeated in CI, editors, local development, and automated agent workflows.

### Stay Inside the Vue Toolchain Scope

Vize should focus on Vue tooling. It may integrate with the broader JavaScript ecosystem, but its
primary responsibility is the Vue single-file component workflow and the tools that make that
workflow reliable:

- Compilation
- Type checking
- Linting
- Formatting
- LSP and editor integrations
- Vite and build tool integrations
- CLI workflows
- Testing infrastructure for real-world Vue projects

Avoid scope expansion that does not improve this toolchain directly.

### Keep the Implementation Shape Good

Implementation quality matters even when speed is the immediate goal. Prefer designs with clear data
ownership, explicit phases, narrow contracts, deterministic behavior, and testable outputs.

Do not accumulate hacks as architecture. A workaround may be accepted to restore correct behavior,
but it must be documented, tested, and followed by a migration plan toward the essential fix.

### Use VoidZero JavaScript Toolchain Assets as Infrastructure

Vize should actively use assets from the VoidZero JavaScript toolchain as infrastructure wherever
that creates leverage. OXC, oxlint, Vite, and related projects are not incidental dependencies; they
are part of the environment Vize should build on.

Use these tools to reduce duplicated effort, improve performance, and align Vize with the future of
the JavaScript toolchain.

### Treat VoidZero Toolchain Integration as First-Class

Integration with VoidZero JavaScript tooling, including oxlint and Vite, should be first-class. This
means:

- Integration behavior is tested, not assumed.
- Integration issues receive the same priority as core compiler issues when they block real usage.
- Vize-specific behavior should complement, not compete with, framework-agnostic tooling.
- Public APIs should make integration explicit and stable enough for external consumers.

### Stay Open and Fair

Vize must remain open and fair. It must not be steered for the private benefit of a specific company,
sponsor, employer, or stakeholder.

The project should focus on the growth of the whole ecosystem. Collaboration is welcome, but
technical decisions should be made for users, maintainers, contributors, and the long-term health of
Vue tooling.

## No Implicit Knowledge, No Proprietary Knowledge

Do not create hidden operational knowledge. Do not rely on private context that only one maintainer
knows. Do not leave critical reasoning in chats, private notes, or memory.

Everything needed to maintain the project should become project knowledge:

- Write every test case that matters.
- Write every TODO that matters.
- Write every edge case that can be named.
- Preserve minimal reproductions.
- Record why behavior is correct, especially when the implementation looks non-obvious.
- Keep release, benchmark, CI, and compatibility expectations visible.

AI has lowered the maintenance cost of tests. The correct response is to write more tests, not fewer.
When in doubt, add the test.

## Regression Rule

When a problem occurs, add a test that prevents the same problem from happening again.

This rule applies even when the problem seems trivial. If the issue reached a user, a real project,
CI, a benchmark, an editor flow, or a contributor, the project needs a durable check. The minimum
acceptable fix is:

1. Reproduce the problem.
2. Minimize the reproduction as much as practical.
3. Add a test, fixture, snapshot, or workflow check that fails before the fix.
4. Fix the behavior.
5. Verify the check passes.

## AI Usage Standard

When AI is used for engineering work on Vize, use at least one of:

- GPT-5.5 xHigh or better
- Opus 4.8 or better

AI output is not authority. It is labor. The maintainer remains responsible for checking correctness,
test coverage, performance impact, and integration behavior.

## Versioning Before 1.0

For the `0.x` major version era, do not distinguish meaningfully between minor and patch releases.
Always increment the minor version for releases.

This keeps pre-1.0 communication simple: users should assume every release may contain meaningful
behavioral movement, compatibility changes, fixes, and new capability.

## Development Cycle

Vize development should run as a repeated discovery, reporting, correction, cleanup, and performance
loop.

### A. Finding Issues

Use real projects listed as fixtures to run end-to-end tests across the Vize toolchain. This means
testing the whole workflow, not only the compiler:

- Compiler
- Type checker
- Linter
- Formatter
- LSP
- Vite integration
- CLI and package workflows
- Any other surface required for real project adoption

During E2E testing, check for both false positives and false negatives. False positives are visible
because Vize reports diagnostics that should not exist. False negatives are more dangerous because
the toolchain stays silent when diagnostics should exist. This is especially common in linters and
type checkers, where missing diagnostics can look like success unless the expected failure cases are
written down and asserted explicitly.

In addition to real-project E2E testing, infer bugs from existing tests and implementation patterns.
If one construct failed, search for its neighbors. If one branch was under-tested, inspect the
adjacent branches. Work through the possibility space until the class of bug is exhausted.

Also encourage external contributors and real users to try Vize on real projects. Reports from real
projects are one of the highest-value inputs because they expose integration behavior that isolated
tests often miss.

### B. Reporting Issues

When E2E testing reveals a problem, create a minimal reproduction and file an issue.

Keep issue scope as small as practical. Use the subproject name in the issue or PR scope where
possible. Example:

```text
fix(patina): avoid false positive for v-if alias usage
```

A good issue should contain:

- The affected Vize surface or crate.
- The minimal reproduction.
- The expected behavior.
- The actual behavior.
- The real project or external report that exposed the problem, when applicable.
- Links to related fixtures, inspector output, CI failures, or user reports.

### C. Correction Loop

Use this loop continuously:

1. Run the issue-finding process from section A.
2. Report the issue using the process from section B.
3. Fix the implementation so behavior becomes correct, even if the first fix includes a workaround.
4. If the issue came from an external report, include a backlink and credit the reporter, such as with
   `Co-authored-by` or an equivalent explicit attribution.
5. If a workaround was used, migrate toward the essential implementation change. Do not let the codebase
   become a pile of workarounds.
6. Tune performance after correctness is restored and covered.
7. Return to step 1.

This cycle is never complete. It is the default operating rhythm of the project.

## Review Standards

Every change should answer these questions:

- Does this make Vize faster, more correct, more compatible, or easier to maintain?
- Is the touched behavior covered by tests, fixtures, snapshots, or CI?
- Are edge cases written down?
- Is the scope clear and conventional?
- Does the change integrate cleanly with Vue, Vite, OXC, oxlint, and the relevant Vize surfaces?
- Does it avoid private assumptions and maintainer-only knowledge?
- If it is a workaround, is the follow-up path explicit?

For pull requests, use Conventional Commits style in titles and commits. Examples:

```text
fix(patina): handle directive aliases in nested scopes
test(atelier): add fixture for scoped style rewrite
perf(canon): reuse virtual file graph during incremental checks
docs: document real-project E2E workflow
```

## Operational Priority

When priorities conflict, use this order:

1. Correctness for real Vue projects.
2. Reproducible tests and fixtures.
3. End-to-end integration across the toolchain.
4. Performance.
5. Cleanup that removes workarounds or implicit behavior.
6. Documentation that makes the decision transferable.

Speed remains foundational, but a fast incorrect toolchain is not useful. Restore correctness first,
then make the correct path fast.

## Continuity Expectation

If ubugeeei is unavailable, maintainers should not wait for hidden intent. Use this document,
existing tests, public issues, real-project reports, and CI as the source of truth.

The project should keep moving in public, with explicit reasoning, small reproducible cases,
conventional scopes, and a bias toward tests. That is the redundancy model.
