<p align="center">
  <img src="./assets/readme-screenshot.png" alt="Vize" width="600" />
</p>

<p align="center">
  <strong>Unofficial High-Performance Vue.js Toolchain in Rust</strong>
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

Vize is an unofficial, Rust-native toolchain for Vue — one fast, vertically integrated pipeline
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
([latest run](https://github.com/ubugeeei-prod/vize/actions/runs/27081731245)):

| Surface     | Existing tool      | Existing |    Vize |    Speedup |
| ----------- | ------------------ | -------: | ------: | ---------: |
| SFC compile | @vue/compiler-sfc  |    9.94s | 272.7ms |  **36.4×** |
| Lint        | eslint-plugin-vue  |   42.14s | 218.0ms | **193.3×** |
| Format      | Prettier           |   85.16s |   1.62s |  **52.6×** |
| Type check  | vue-tsc            |    3.86s | 629.6ms |   **6.1×** |
| Vite build  | @vitejs/plugin-vue |    1.07s | 487.0ms |   **2.2×** |

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

- [Blacksmith](https://www.blacksmith.sh/) for sponsoring CI/CD runner infrastructure.
- [かっこかり](https://github.com/kakkokari-gtyih) for regular debugging and monitoring around
  [Misskey](https://github.com/misskey-dev/misskey) (~103k lines of Vue across 586 SFCs), including many compiler-focused bug reports.
- [ushironoko](https://github.com/ushironoko) for compiler, linter, and CLI bug reports,
  reference implementations, and reproduction repositories.
- [dannote](https://github.com/dannote) for Elixier feedback, PRs, and CSS-facing fixes.
- [n13u](https://x.com/%5Fn13u%5F) and `#frontend_phpcon_do` for Nuxt build debugging, reports,
  and production validation
  ([report](https://x.com/%5Fn13u%5F/status/2061408599788892230?s=20)).
- [sevenc-nanashi](https://github.com/sevenc-nanashi) for building the
  [VOICEVOX](https://github.com/VOICEVOX/voicevox) editor (~26k lines of Vue across 128 SFCs)
  against Vize as a real-world milestone and compatibility feedback.
- Everyone who has mentioned, shared, tested, or amplified Vize across the community.

## License

[MIT](./LICENSE)
