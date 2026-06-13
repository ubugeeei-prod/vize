---
title: Credits
description: People and community feedback that have shaped Vize.
---

# Credits

Vize is shaped by practical feedback from people using it against real applications, adjacent
tooling, and ecosystem experiments. In particular, thank you to:

- [Blacksmith](https://www.blacksmith.sh/) for sponsoring high-performance CI/CD runners and Testbox infrastructure, giving Vize the compute needed to run frequent benchmarks and compatibility checks across real projects.
- [Mates Inc.](https://eng.mates.education/) for allowing ubugeeei, its employee, to dedicate discretionary work time to OSS and for adopting Vize in the build for the company's engineering website.
- [OpenAI Codex for Open Source](https://openai.com/form/codex-for-oss/) for supporting open-source maintainers through a program that helps keep critical OSS development moving.
- [かっこかり](https://github.com/kakkokari-gtyih) for continuously testing Vize's compiler and Vite Plugin on [Misskey](https://github.com/misskey-dev/misskey) — a Vue application with ~103k lines of Vue across 586 SFCs — and sending timely reports as the implementation changed ([report](https://github.com/ubugeeei-prod/vize/discussions/71)).
- [ushironoko](https://github.com/ushironoko) for bug reports from the compiler, linter, and CLI sides of the project, along with reference implementations for fixes and reproduction repositories for difficult issues.
- [dannote](https://github.com/dannote) for bringing Vize into the Elixir community through [Volt](https://hexdocs.pm/volt/readme.html), an Elixir-native frontend toolchain built on Vize, and for reporting missing pieces and sending PRs as Volt adopted Vize as a foundation.
- [umbrella22](https://github.com/umbrella22) for shaping the Rspack Plugin integration through real-project validation, reports about native CSS handling, compatibility feedback across Rspack 1.x and 2.x, and guidance on defaults that keep newer native CSS behavior usable without regressing older Rspack setups.
- [n13u](https://x.com/%5Fn13u%5F) and the `#frontend_phpcon_do` team for persistently reporting bugs while building a Nuxt-based conference website with Vize, then carrying that validation all the way to production adoption ([report](https://x.com/%5Fn13u%5F/status/2061408599788892230?s=20), [write-up](https://www.n13u.dev/ja/blog/detail/nYZKQ3UmslmWfXaP)).
- yamanoku for accessibility-focused feedback around Vize and for using the project in the Vue Fes Japan speaker-site migration documented in the v-tokyo Meetup #25 LT notes ([write-up](https://scrapbox.io/yamanoku/%E3%81%A8%E3%81%82%E3%82%8B%E3%82%B5%E3%82%A4%E3%83%88%E3%81%8Ckingnize%E3%81%95%E3%82%8C%E3%82%8B%E3%81%BE%E3%81%A7%EF%BD%9ENuxt%E3%81%8B%E3%82%89vuerend_%26_Vize%E3%81%B8)).
- [sevenc-nanashi](https://github.com/sevenc-nanashi) for using the [VOICEVOX](https://github.com/VOICEVOX/voicevox) editor — an Electron-based Vue application with ~26k lines of Vue across 128 SFCs — as a real-world target for improving compiler precision and turning production-app gaps into concrete feedback ([report](https://github.com/ubugeeei-prod/vize/discussions/955)).

Thanks also to everyone who has mentioned, shared, tested, or amplified Vize in the community, and
to everyone connected to that work. Those signals make it easier to see where the toolchain is
useful, where it breaks, and what it should become next.

Vize is a personal project by ubugeeei, licensed under the MIT License and maintained as a
non-commercial OSS effort. It is not owned by any specific company, is intended to remain open, and
is not being built with a buyout in mind.
