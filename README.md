<p align="center">
  <img src="./assets/readme-screenshot.png" alt="Vize" width="600" />
</p>

<p align="center">
  <strong>High-Performance Vue.js Toolchain in Rust</strong>
</p>

<p align="center">
  <em>/viːz/: Named after Vizier + Visor + Advisor, a wise tool that sees through your code.</em>
</p>

<p align="center">
  <a href="https://vizejs.dev"><strong>Documentation</strong></a> ・
  <a href="https://vizejs.dev/play/"><strong>Playground</strong></a> ・
  <a href="https://github.com/sponsors/ubugeeei"><strong>Sponsor</strong></a>
</p>

<p align="center">
  <strong>Real World Testing — Wanted</strong>
</p>

<p align="center">
  <video src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/docs/public/blog/vize-real-world-testing.mp4" controls muted width="600"></video>
</p>

<p align="center">
  <a href="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/docs/public/blog/vize-real-world-testing.mp4"><strong>▶ Watch the Real World Testing PV</strong></a>
</p>

> [!WARNING]
> Vize is experimental and in its **Real World Testing** phase — not a completely
> production-ready toolchain yet. Breaking changes and behavior that diverges from Vue are
> expected. Review the [stability guide](https://vizejs.dev/stability),
> [production-readiness checklist](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/production-readiness.md),
> and [support policy](https://github.com/ubugeeei-prod/vize/blob/main/docs/release/support-policy.md)
> before adopting it.

## What Is Vize?

Vize is a Rust-native toolchain for Vue — one fast, vertically integrated lane
for single-file components. A single shared parser powers compilation, linting, type-checking,
formatting, and editor tooling, so your whole Vue workflow runs on the same high-performance core
instead of a patchwork of disconnected tools.

It plugs into where you already work: `@vizejs/vite-plugin` (Vite), the `vize` npm package
(project scripts and shared config helpers), the native `vize` binary (LSP / profiling /
specialized CLI workflows), `@vizejs/vite-plugin-musea` (Musea), and `oxlint-plugin-vize`
(Oxlint).

**Everything lives in the [documentation](https://vizejs.dev)** — start with
[Getting Started](https://vizejs.dev/getting-started).

Vize is in its Real World Testing phase: issues and PRs are very welcome, and we are looking for
reasonably large Vue projects to use as test beds.

## Benchmarks

Measured on Blacksmith `blacksmith-32vcpu-ubuntu-2404`, 15,000 generated Vue SFCs, median of 5 runs
([latest run](https://github.com/ubugeeei-prod/vize/actions/runs/27408931576)):

| Surface     | Existing tool      | Existing |    Vize |    Speedup |
| ----------- | ------------------ | -------: | ------: | ---------: |
| SFC compile | @vue/compiler-sfc  |   16.86s | 292.8ms |  **57.6×** |
| Lint        | eslint-plugin-vue  |   55.54s | 260.7ms | **213.1×** |
| Format      | Prettier           |  139.17s |   1.43s |  **97.5×** |
| Type check  | vue-tsc            |    5.37s | 402.4ms |  **13.3×** |
| Vite build  | @vitejs/plugin-vue |    1.63s | 611.0ms |   **2.7×** |

See the [Blacksmith benchmark snapshot](https://vizejs.dev/architecture/performance-blacksmith) for
methodology and per-variant numbers.

## Credits

This project draws inspiration from [Volar.js](https://github.com/volarjs/volar.js),
[vuejs/language-tools](https://github.com/vuejs/language-tools),
[eslint-plugin-vue](https://github.com/vuejs/eslint-plugin-vue),
[eslint-plugin-vuejs-accessibility](https://github.com/vue-a11y/eslint-plugin-vuejs-accessibility),
[Lightning CSS](https://github.com/parcel-bundler/lightningcss),
[Storybook](https://github.com/storybookjs/storybook), and
[OXC](https://github.com/oxc-project/oxc).

Special thanks to:

- [Blacksmith](https://www.blacksmith.sh/) for sponsoring high-performance CI/CD runners and
  Testbox infrastructure for frequent benchmarks and real-project compatibility checks.
- [Mates Inc.](https://eng.mates.education/) for allowing ubugeeei, its employee, to dedicate
  discretionary work time to OSS and for adopting Vize in the build for the company's engineering
  website.
- [OpenAI Codex for Open Source](https://openai.com/form/codex-for-oss/) for supporting
  open-source maintainers through a program that helps keep critical OSS development moving.
- [かっこかり](https://github.com/kakkokari-gtyih) for continuously testing Vize's compiler and
  Vite Plugin on [Misskey](https://github.com/misskey-dev/misskey) (~103k lines of Vue across 586
  SFCs), with timely reports as the implementation changed
  ([report](https://github.com/ubugeeei-prod/vize/discussions/71)).
- [ushironoko](https://github.com/ushironoko) for compiler, linter, and CLI bug reports,
  reference implementations, and reproduction repositories.
- [dannote](https://github.com/dannote) for bringing Vize into the Elixir community through
  [Volt](https://hexdocs.pm/volt/readme.html), an Elixir-native frontend toolchain built on Vize,
  and for reporting missing pieces and sending PRs as Volt adopted Vize as a foundation.
- [n13u](https://x.com/%5Fn13u%5F) and `#frontend_phpcon_do` for persistently reporting bugs while
  building a Nuxt-based conference website with Vize, then carrying that validation all the way to
  production adoption
  ([report](https://x.com/%5Fn13u%5F/status/2061408599788892230?s=20),
  [write-up](https://www.n13u.dev/ja/blog/detail/nYZKQ3UmslmWfXaP)).
- [sevenc-nanashi](https://github.com/sevenc-nanashi) for using the
  [VOICEVOX](https://github.com/VOICEVOX/voicevox) editor (~26k lines of Vue across 128 SFCs) as a
  real-world target for improving compiler precision
  ([report](https://github.com/ubugeeei-prod/vize/discussions/955)).
- Everyone who has mentioned, shared, tested, or amplified Vize across the community.

Vize is a personal project by ubugeeei, licensed under the MIT License and maintained as a
non-commercial OSS effort. It is not owned by any specific company, is intended to remain open, and
is not being built with a buyout in mind.

## License

[MIT](./LICENSE)
