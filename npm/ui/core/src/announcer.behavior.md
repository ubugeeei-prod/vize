# Announcer behavior contract

Normative state × input → outcome table for `announcer-provider.vue` and the
announcement queue (`@vizejs/ui/announcer`). Every row is proven by the named
test in `src/announcer.test.ts` or `src/announcer-ssr.test.ts`; compile-only
assertions live in `src/announcer.types.test-d.ts`.

| #   | State        | Input                          | Outcome                                            | Proven by                                            |
| --- | ------------ | ------------------------------ | -------------------------------------------------- | ---------------------------------------------------- |
| N1  | idle         | render provider                | one empty polite and one empty assertive region    | `renders one polite and one assertive region`        |
| N2  | idle         | `announce` sequence            | messages flush in order, one channel tick each     | `queues announcements sequentially`                  |
| N3  | queued       | assertive `announce`           | assertive text precedes queued polite text         | `flushes assertive announcements before polite ones` |
| N4  | queued       | identical `announce`           | duplicate pending text on a channel is dropped     | `deduplicates identical pending announcements`       |
| N5  | queued       | keyed `announce`               | a pending message with the same key is replaced    | `coalesces keyed announcements`                      |
| N6  | busy         | `update` then `end`            | only the latest progress and the outcome are heard | `announces busy work without flooding`               |
| N7  | busy ended   | `update`                       | throws a stable busy diagnostic                    | `rejects progress after a busy announcement ends`    |
| N8  | nested       | render provider in provider    | inner provider delegates; no duplicate regions     | `nested providers reuse the owner's regions`         |
| N9  | nested       | descendant `announce`          | text surfaces in the owning provider's region      | `nested providers reuse the owner's regions`         |
| N10 | queued       | `clear`                        | pending queue drops and both channels empty        | `clears pending announcements and both channels`     |
| N11 | any          | `dispose`, then any call       | throws a stable disposed diagnostic                | `rejects use after dispose`                          |
| N12 | no scope     | `useAnnouncer`                 | throws a stable setup diagnostic                   | `rejects composable use outside an effect scope`     |
| N13 | SSR          | render nested + setup announce | byte-identical markup, single empty region pair    | SSR test                                             |
| N14 | public types | invalid politeness             | compilation rejects misuse                         | `src/announcer.types.test-d.ts`                      |
