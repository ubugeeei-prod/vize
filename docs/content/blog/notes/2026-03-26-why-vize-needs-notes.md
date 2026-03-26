---
title: Why Vize Now Has a Notes Lane
description: Some project updates need room for context, not just a changelog entry.
---

# Why Vize Now Has a Notes Lane

<div class="blog-post-meta">
  <div class="blog-meta-chip">
    <div>
      <span class="blog-meta-label">Published</span>
      <span class="blog-meta-value">2026-03-26</span>
    </div>
  </div>

  <a class="blog-author-card" href="https://github.com/ubugeeei">
    <img src="https://github.com/ubugeeei.png" alt="ubugeeei" />
    <span class="blog-author-text">
      <span class="blog-meta-label">Author</span>
      <span class="blog-meta-value">ubugeeei</span>
    </span>
  </a>
</div>

Reference docs are great at answering "how do I use this?" They are much worse at answering questions like:

- Why was this feature added?
- What tradeoff led to this architecture?
- Which experiments are promising, but not stable yet?
- What is the project trying to learn next?

That is why the docs now include a separate **Notes** lane.

## What Belongs Here

Notes are intentionally irregular. A post can be:

- a devlog after a week of compiler work
- an architecture writeup for a new crate
- a design note about Musea, the LSP, or the Vite integration
- a short project update that is useful, but not tied to a version tag

## Why Not Put Everything in Release Notes

Release notes are optimized for shipped changes. They should stay clear and actionable.

Notes make room for broader storytelling: the context behind a feature, the shape of the roadmap, and the thinking that helps readers understand the project over time.

## Writing Direction

When in doubt:

- use **Release Notes** if the post announces something shipped
- use **Notes** if the post explains thinking, progress, or experiments
