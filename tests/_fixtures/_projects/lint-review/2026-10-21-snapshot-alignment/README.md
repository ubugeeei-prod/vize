# 2026-10-21 lint snapshot alignment review

This derivative review preserves the immutable 2026-10-11 producer captures while admitting the seven current snapshots reproduced on unchanged main by [App E2E lint run 39469802447](https://github.com/ubugeeei-prod/vize/actions/runs/39469802447). The reviewed predecessor normalizes the accepted `vue/require-scoped-style` message and help wording before comparison. `review.json.gz` binds the previous and current whole-file hashes and classifies every changed packet with before/after canonical hashes.

| Fixture | Messages | Changed packets | Reviewed change classes |
| --- | ---: | ---: | --- |
| ant-design-vue | 387 → 391 | 55 | narrower attribute ranges; event hyphenation; duplicate attribute inheritance |
| elk | 251 → 181 | 52 | stale deprecated attributes removed; duplicate class and attribute inheritance |
| misskey | 1,753 → 1,950 | 305 | corrected attribute and v-for ranges; event hyphenation; v-for validity and keys; computed returns |
| npmx.dev | 431 → 459 | 36 | duplicate classes; event hyphenation; computed returns; attribute inheritance; accessibility and ordering adjustments |
| nuxt-ui | 20 → 20 | 4 | three stale v-html findings replaced by three attribute-inheritance findings |
| reka-ui | 368 → 472 | 64 | duplicate keys and attribute inheritance; duplicate class; v-else/v-for; one form-label false positive removed |
| vuefes-2025 | 40 → 44 | 5 | net four attribute-inheritance findings |
