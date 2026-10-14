# Misskey LSP churn RSS ceiling from its measured distribution

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

The Misskey LSP churn stress oracle's server RSS ceiling (`maxPeakRssMiB` in
the `misskey` `lspChurnBudget` block of `tests/_fixtures/vue-ecosystem-fixtures.json`)
moves from 128 MiB to 136 MiB. Every other churn budget, the process-tree
ceiling (384 MiB) and the workload stay unchanged.

## Reason

The 128 MiB ceiling sat inside the noise of the measurement it gates. The 80
most recent `misskey-lsp-churn-metrics` artifacts (2026-10-11 to 2026-10-14,
Check full, release and merge-queue runs) sampled a peak server RSS of
114.1 to 131.2 MiB: mean 124.9 MiB, standard deviation 3.5 MiB, p50 125.4,
p90 129.2, p99 130.8. 79 of the 80 peaks fall in the first file lifecycle.

There is no trend. Values above 128 MiB appear on 2026-10-11 as often as on
2026-10-14, and merge-queue candidates built on the same base commit differ by
more than 5 MiB (for example 124.6 and 129.9 MiB on base `17b0244`). Thirteen
of the 80 runs exceeded 128 MiB. After vue-parity joined the merge queue
(#9051), all four of its queue failures were this ceiling (128.0, 128.6, 129.2
and 129.9 MiB), each ejecting a PR that did not change the language server.
The same noise failed the release Check runs for v0.447.0 and v0.448.0, and
the 4.5 MiB improvement credited to #8802 is within one run-to-run spread.

136 MiB is the observed maximum plus about 1.5 standard deviations (mean plus
3.2 standard deviations). It still fails a real regression of about 11 MiB
over the current mean, which is the smallest growth this sampled peak can
distinguish from noise.

## Tradeoff

This is a rebaseline, not a ratchet: the ceiling rises by 8 MiB. A stable
memory gate needs a less noisy statistic than one sampled peak per run;
until one exists, a ceiling below p99 only measures runner noise. Lower the
ceiling again when a quieter measurement or a real reduction supports it.
