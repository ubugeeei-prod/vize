# Davinci — Implementation Plan

> [!NOTE]
> PR-granular decomposition of the [roadmap](../roadmap.md) phases, written
> for the agent-implements / maintainer-reviews regime (charter #25). One file
> per phase. This page defines the task format; the phase files are the plan.

## Task format

Every task is one reviewable PR (or an explicitly-marked small series) and
carries:

- **ID** — `P<phase>-<n>`, stable once assigned; referenced in PR titles as
  `davinci(P0-3): …`.
- **Deliverable** — what exists after merge, stated as an artifact, never as
  activity.
- **Acceptance criteria** — machine-checkable conditions (commands that pass,
  artifacts that exist, budgets that hold). A criterion a CI job cannot
  evaluate is a smell; prose criteria appear only as explicitly-marked
  review points.
- **Deps** — task IDs that must land first. Tasks without dependency edges
  between them are parallelizable by different agents.
- **Non-goals** — the nearest scope creep, named.

Rules of engagement, inherited from the charter: behavior changes need
fixtures before code (#21); every PR holds the standing gates that exist at
its merge time; a task that discovers its own scope was wrong updates the
plan file in the same PR (the plan is code).

**Provisional exception:** phases marked provisional (2–6) carry compressed
per-task blocks instead of the full format above. The full format becomes
mandatory when a phase is re-cut at its predecessor's exit — a provisional
task cannot be picked up for implementation until it has been expanded to
carry the full contract.

## Phase files

| File                               | Phase                                                                                           | Status                                               |
| ---------------------------------- | ----------------------------------------------------------------------------------------------- | ---------------------------------------------------- |
| [phase-0.md](./phase-0.md)         | Instrumentation and groundwork                                                                  | **Drafted, full detail** — ready to execute          |
| [phase-1.md](./phase-1.md)         | One arena, real expressions                                                                     | **Drafted, full detail** — dependency chain explicit |
| [phase-2.md](./phase-2.md)         | Disegno and the pass manager                                                                    | Drafted, provisional — re-cut at P1 exit             |
| [phase-3.md](./phase-3.md)         | Impeto and backend convergence                                                                  | Drafted, provisional — re-cut at P2 exit             |
| [phase-4.md](./phase-4.md)         | Consumer convergence                                                                            | Drafted, provisional — re-cut at P3 exit             |
| [phase-5.md](./phase-5.md)         | Incrementality substrate                                                                        | Drafted, provisional — re-cut at P4 exit             |
| [phase-6.md](./phase-6.md)         | Extension contracts GA                                                                          | Drafted, provisional — re-cut at P5 exit             |
| [continuous.md](./continuous.md)   | Cross-phase workstreams (Spolvero, AI loop, corpus, assurance, formal)                          | Drafted — items trigger on their substrate           |
| [test-suites.md](./test-suites.md) | The canonical suite registry (TS-1..51): commands, oracles, and the phase → mandatory-suite map | Drafted                                              |

P0 and P1 carry full per-task **Steps** sub-checklists (concrete paths,
commands, type names). P2–P6 are enumerated to maximum known detail as
per-task blocks but marked **provisional**: each is re-cut when its
predecessor exits, so measured reality — not today's guesses — sets the final
task boundaries. Suites are referenced by TS-id from the registry — a gate
naming an unregistered suite is a plan bug. Every phase file keeps a checkbox
TODO index at the top; checking a box happens in the PR that satisfies the
task's acceptance criteria, never before.
