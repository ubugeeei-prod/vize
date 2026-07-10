---
title: Tooling Compare
description: A practical comparison of Vize and nearby projects across official Vue tooling, Oxc, Golar, Verter, Flint, and TSSLint.
---

# Tooling Compare

<div class="blog-post-meta">
  <span class="blog-meta-chip">
    <span>
      <span class="blog-meta-label">Published</span>
      <span class="blog-meta-value">2026-05-16</span>
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

Vize is close enough to several projects that comparison is inevitable.

That comparison is useful, but only if the axis is clear. "Faster" is not enough. "Rust" is not enough. "Vue support" is not enough.

The real question is: **which layer does each project want to own?**

![Relationship map showing Vize in the nearby tooling landscape, with reference-only, adjacent platform, used-by-Vize, and compare-only groups](/blog/vize-toolchain-map.svg)

## Quick Map

| Project              | Center of gravity                                             | How Vize relates to it                                                             |
| -------------------- | ------------------------------------------------------------- | ---------------------------------------------------------------------------------- |
| Official Vue tooling | The production baseline for Vue compiler and language tooling | Vize is independent and experimental, so it must treat this as the reference point |
| Oxc / Oxlint         | General JavaScript and TypeScript infrastructure              | Vize can reuse and cooperate with Oxc while owning Vue-specific semantics          |
| Golar                | `typescript-go`-based embedded-language type checking         | Vize has a broader Vue toolchain scope than type checking alone                    |
| Verter               | Alternative next-generation Vue compiler and toolchain        | Closest in ambition, different in architecture and product shape                   |
| Flint                | Friendly, typed JS/TS linting with strong defaults            | Complementary for general TS linting, not a Vue SFC toolchain                      |
| TSSLint              | TypeScript-native linting inside the language server          | Strong semantic linting idea, but not a full Vue compiler/linter/gallery stack     |

## Official Vue Tooling

The official stack matters first.

[Vue Language Tools](https://github.com/vuejs/language-tools), `vue-tsc`, the Vue compiler packages, and the official editor integrations are the production baseline. When Vize disagrees with official behavior, that disagreement is not automatically a bold new idea. Most of the time it is a needed fix, an incomplete implementation, or a place where Vize needs a clearer compatibility story.

That does not make Vize pointless.

It defines the contract.

Vize can experiment with a more unified Rust-native architecture, but it still needs to care about the shape of real Vue code, real compiler output, real diagnostics, and real editor expectations. The official stack is the reference point that keeps the experiment honest.

## Oxc and Oxlint

[Oxc](https://oxc.rs/) is a general-purpose JavaScript and TypeScript compiler infrastructure project. [Oxlint](https://oxc.rs/docs/guide/usage/linter.html) is the high-performance linter built on top of that world.

Vize should not compete with Oxc at the JavaScript and TypeScript layer. That would be wasteful. Oxc already gives the ecosystem a fast parser, semantic infrastructure, formatter direction, linter direction, and a growing set of shared primitives.

The Vize question is narrower and more Vue-specific:

- What is a `.vue` file as a whole?
- How do template scopes connect to script bindings?
- How do directives, slots, props, emits, style blocks, and compiler output relate?
- How do we map diagnostics back to the exact source that humans edit?
- How do those semantics feed compile, lint, format, type-check, LSP, Musea, and AI workflows?

Oxc can be the general JS/TS foundation. Vize can be the Vue-specific toolchain that uses that foundation without flattening Vue into "just script blocks."

## Golar

[Golar](https://github.com/auvred/golar) is interesting because it takes `typescript-go` seriously for embedded languages.

Its center is type checking, virtual code, and `tsgo` integration. For Vue, that naturally puts it close to the official language-core model. That is a good and practical shape: reuse the Vue virtual-code machinery and make the TypeScript engine faster or more flexible.

Vize is trying to solve a wider problem.

The type-checking layer matters, but it is not the whole project. Vize wants the parser, semantic model, compiler, linter, formatter, native type-check path, LSP, component gallery, and AI-facing surfaces to share more of the same Vue-aware core.

So the difference is not "Golar is type checking and Vize is faster type checking."

The difference is:

- Golar is primarily an embedded-language TypeScript processing story.
- Vize is a full Vue toolchain story where type checking is one consumer of the Vue analysis model.

## Verter

[Verter](https://github.com/pikax/verter) is probably the closest comparison philosophically.

It is also asking a big question: what would a next-generation Vue toolchain look like if we were willing to rethink the layers?

That is close to Vize's question. Both projects care about compiler behavior, language tooling, diagnostics, and a stricter experience than a bag of unrelated plugins can easily provide.

The differences are in emphasis:

- Verter appears stricter and language-service oriented from the beginning.
- Vize emphasizes a Rust-native shared core across compile, lint, format, check, LSP, Musea, and AI workflows.
- Vize also treats component-gallery and design-system tooling as first-class parts of the frontend environment, not as separate documentation afterthoughts.

I do not see Verter as an enemy. It is another serious experiment in a space that deserves multiple experiments.

## Flint

[Flint](https://www.flint.fyi/) is a different kind of comparison.

It is a JavaScript and TypeScript linter with an emphasis on useful defaults, caching, and typed linting. That is valuable because the JS/TS ecosystem has a real problem: syntax-only linting is fast but incomplete, while semantic linting can become slow and operationally expensive.

Vize agrees with the premise that semantic feedback should be practical, fast, and pleasant.

But Flint is not trying to be a Vue SFC compiler, formatter, template analyzer, component gallery, or Vue-specific LSP. It is better understood as a high-quality general linting direction.

The complementary shape is:

- Flint can push the JS/TS linting experience forward.
- Vize can push Vue-specific analysis forward.
- A good frontend environment should make those layers cooperate instead of forcing every tool to own every concern.

## TSSLint

[TSSLint](https://marketplace.visualstudio.com/items?itemName=johnsoncodehk.vscode-tsslint) is important because it treats TypeScript semantic linting as something that can live close to the TypeScript language server.

That idea is compelling: if the TypeScript checker already has a project open, why rebuild the world in a separate linter process just to answer semantic questions?

Vize has a similar instinct, but pointed at Vue as a multi-language artifact.

For Vize, the question is not only "can lint rules reuse TypeScript state?" It is:

- Can template analysis reuse the same Vue semantic model as the compiler?
- Can type-aware Vue lint rules ask focused questions without paying a full rebuild cost?
- Can editor diagnostics, batch checks, and AI repair loops agree on the same source mapping?
- Can the system keep a project session alive long enough to amortize work?

TSSLint is a strong signal that semantic linting wants to move closer to existing language state. Vize extends that instinct into Vue-specific structure.

## What Vize Is Trying to Own

Vize should not own everything.

It should own the places where Vue-specific knowledge must be coherent:

- SFC parsing and block structure
- template semantics
- directive and component analysis
- compiler output decisions
- Vue-aware lint diagnostics
- source mapping from generated artifacts back to `.vue`
- component metadata for Musea
- machine-readable diagnostics for AI workflows

It should cooperate elsewhere:

- use Oxc for JavaScript and TypeScript parsing where possible
- compare behavior against official Vue tooling
- learn from Golar, TSSLint, and Flint on type-aware feedback loops
- stay aware of Verter as another full-toolchain experiment

## The Product Position

The cleanest positioning is this:

> Vize is an independent, experimental, Rust-native Vue toolchain that tries to make compiler, linter, formatter, type checker, LSP, component gallery, and AI-facing diagnostics feel like one coherent environment.

That means Vize is not the official answer.

It is a high-speed experimental answer.

The job now is to make that answer useful in real projects, narrow the gap with official behavior, and keep the architecture sharp enough that the experiment is worth having.
