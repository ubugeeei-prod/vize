---
title: Blog
description: Release notes and irregular notes from the Vize project.
---

# Blog

The Vize docs now have two writing lanes:

<div class="blog-grid">
  <a class="blog-card" href="./releases/">
    <span class="blog-card-kicker">Track</span>
    <strong>Release Notes</strong>
    <p>Shipped changes, release highlights, migration notes, and rollout guidance.</p>
  </a>

  <a class="blog-card" href="./notes/">
    <span class="blog-card-kicker">Track</span>
    <strong>Notes</strong>
    <p>Irregular posts for devlogs, design writeups, architecture notes, and behind-the-scenes updates.</p>
  </a>
</div>

## How to Publish

- Release posts live in `docs/content/blog/releases/`.
- Irregular posts live in `docs/content/blog/notes/`.
- Use `YYYY-MM-DD-slug.md` for filenames so posts stay sortable.
- Start from `docs/templates/blog-release.md` or `docs/templates/blog-note.md`.
- New posts show up under the matching section in the docs tree.

## Starting Points

- [Release Notes](./releases/)
- [Notes](./notes/)

## Latest Posts

<div class="blog-post-list">
  <a class="blog-post-list-item" href="./notes/2026-06-07-real-world-testing/">
    <strong>Real World Testing</strong>
    <span>Vize enters the Real World Testing phase — real projects are the test suite now, with a clear roadmap to v1.0.0.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-comparing-vize-with-official-vue-oxc-golar-verter-flint-and-tsslint/">
    <strong>Tooling Compare</strong>
    <span>A practical comparison of Vize and nearby projects across official Vue tooling, Oxc, Golar, Verter, Flint, and TSSLint.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain/">
    <strong>Performance Tuning</strong>
    <span>Practical performance lessons from building a Vue toolchain where parsing, allocation, parallelism, and feedback loops all matter.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-testing-agentic-coding-and-trust/">
    <strong>Testing & Agents</strong>
    <span>Why snapshot-heavy tests, real-world fixtures, and deterministic checks matter more when agents are part of the development loop.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-vapor-mode-and-the-next-vue-compiler-surface/">
    <strong>Vapor Mode</strong>
    <span>Why Vapor Mode matters for Vize, and why a direct fine-grained compiler path changes more than runtime performance.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-vue-as-a-language-and-the-strongest-frontend-environment/">
    <strong>Vue as Language</strong>
    <span>Building on the idea that Vue is a language for UI, this note explains why frontend development needs a coherent environment rather than scattered tools.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-why-musea-and-design-systems-matter-in-the-ai-era/">
    <strong>Musea & AI</strong>
    <span>AI can generate UI quickly, but Musea and design systems make the intent, constraints, accessibility, and review workflow durable.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-real-world-feedback-and-the-road-to-production-ready/">
    <strong>Production Ready</strong>
    <span>Why exhaustive real-world validation and community feedback are the path from experimental project to production-ready toolchain.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-05-16-personal-tooling-and-development-speed/">
    <strong>Personal Speed</strong>
    <span>Why Vize being independent and personal can be an advantage for exploration, speed, and ambitious toolchain design.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-the-advantages-and-beauty-of-toolchains-and-vertical-integration/">
    <strong>Vertical Toolchains</strong>
    <span>Why owning more of the stack can improve speed, coherence, and even the aesthetic quality of developer tools.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-why-ai-needs-deterministic-fast-static-analysis/">
    <strong>Static Analysis for AI</strong>
    <span>As AI writes more code, we need faster and more reliable static feedback, not less.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-where-vize-fits-in-the-vue-tooling-landscape/">
    <strong>Vue Tooling Map</strong>
    <span>A map of where Vize sits in the current Vue tooling landscape, and how it differs from adjacent projects.</span>
  </a>
  <a class="blog-post-list-item" href="./releases/2026-03-26-oxlint-plugin-vize-alpha/">
    <strong><code>oxlint-plugin-vize</code> Alpha</strong>
    <span>A new Oxlint JS plugin bridge brings Vize Patina diagnostics into a single Oxlint run for Vue SFCs.</span>
  </a>
  <a class="blog-post-list-item" href="./releases/2026-03-26-docs-blog-support/">
    <strong>Docs Blog</strong>
    <span>The Vize docs can now host both release notes and irregular notes.</span>
  </a>
  <a class="blog-post-list-item" href="./notes/2026-03-26-why-vize-needs-notes/">
    <strong>Notes Lane</strong>
    <span>Some project updates need room for context, not just a changelog entry.</span>
  </a>
</div>
