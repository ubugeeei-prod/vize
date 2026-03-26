---
title: Docs Now Have a Blog
description: The Vize docs can now host both release notes and irregular notes.
---

# Docs Now Have a Blog

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

The Vize docs can now host two kinds of posts directly inside `docs/content/blog/`:

- `releases/` for shipped changes and release communication
- `notes/` for irregular writing such as devlogs, architecture writeups, and project updates

## What Changed

- Added a top-level **Blog** section to the docs.
- Split the writing flow into **Release Notes** and **Notes**.
- Added starter templates so future posts are easy to create and keep consistent.

## Why This Matters

Vize has already grown into more than a package README. Some updates belong in reference docs, but others need a place for narrative context: what shipped, why it matters, what is still experimental, and where the project is heading.

This new blog structure creates that space without introducing a separate site or a second publishing workflow.

## Where to Write

- Release posts: `docs/content/blog/releases/`
- Irregular posts: `docs/content/blog/notes/`
- Templates: `docs/templates/blog-release.md` and `docs/templates/blog-note.md`
