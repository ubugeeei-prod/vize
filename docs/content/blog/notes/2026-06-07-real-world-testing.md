---
title: Real World Testing
description: Vize enters the Real World Testing phase — real projects are the test suite now, with a clear roadmap to v1.0.0.
---

# Real World Testing

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">Published</span>
      <span class="blog-meta-value">2026-06-07</span>
    </span>
  </span>
  <a class="blog-author-card" href="https://github.com/ubugeeei">
    <img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
    <span class="blog-author-text">
      <span class="blog-meta-label">Author</span>
      <span class="blog-meta-value">ubugeeei</span>
    </span>
  </a>
</div>

[▶ Watch the Real World Testing PV](/blog/vize-real-world-testing.mp4)

Vize is entering a new phase.

Until now, development has focused on implementing features, building infrastructure,
and validating behavior through dedicated test suites and synthetic examples.

The next step is different.

We are now actively looking for **real-world projects** to test Vize against.

## The Goal

The goal of this phase is to discover compatibility issues, specification gaps,
performance bottlenecks, and edge cases that only appear in production codebases.

If you maintain a Vue application, library, framework, or tool, we would love to hear
about your experience running it with Vize.

Every bug report, reproduction, benchmark result, and compatibility issue helps move
the project closer to its first stable release.

## Still Experimental — Correctness First

Vize should still be considered experimental. Breaking changes may occur, bugs are
expected, and behavior may differ from Vue in certain scenarios.

The focus of this phase is not feature development. The focus is correctness.
Real-world applications are the test suite now. If you encounter an issue, please
report it — every report helps improve the compiler, the language specification, and
the overall ecosystem.

## How to Help

We are waiting for plenty of issues and PRs. We are also actively recruiting reasonably
large Vue projects to use as test beds — the bigger and more real the codebase, the more
valuable the signal. If you maintain (or know of) a substantial Vue application, library,
framework, or tool, please open an issue or reach out so we can run Vize against it. Bug
reports, reproductions, and benchmark results are all welcome.

See the [Testing & Feedback](../../guide/testing.md) guide for how to inspect output in the
playground, read the existing test cases, profile with `vize check --profile`, and offer a project
as an E2E / VRT test bed.

## Roadmap to v1.0.0

The current phase is **Real World Testing**.

Once Vize successfully completes this phase, the project will move through:

- v1.0.0-alpha
- v1.0.0-beta
- v1.0.0-rc
- v1.0.0

The alpha, beta, and release candidate stages will focus on stabilization, ecosystem
compatibility, performance improvements, and long-term maintenance guarantees.

The goal is not to rush to 1.0. The goal is to earn it.

If you are interested in helping shape the future of Vize, now is the best time to get
involved.
