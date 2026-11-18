In the Real Project Matrix (#12336, run 44093557816), reka-ui's typecheck surface fails with:

> Documented typecheck difference ledger is stale for reka-ui: 1 of 16 entries did not reproduce; remove converged rows from tests/_fixtures/compat-documented-differences.json:
> - packages/core/src/PinInput/story/PinInput.story.vue:20:11

The row recorded a Vize-only `TS2322 Type 'string[]' is not assignable to type 'number[]'` on `v-model="value"` for the generic `PinInputRoot` (`<script setup generic="Type extends PinInputType">`). The story declares `const value = ref<string[]>([])` and passes no `type`, so `Type` infers as `'text'` and `PinInputValue<'text'>` accepts the model. vue-tsc accepts it too. The row was a false positive labelled as a stricter check. Vize now infers the generic from the model and agrees with vue-tsc, so no stricter-than-vue-tsc check is lost.

The only canon commits between the last passing observation and b3858ac that touch generic inference are #13254 (infer slot props of generic children inside generic components) and #13119. #13254 is the likely convergence, but I did not bisect, so it is a candidate only.

This PR removes only that row; the other 15 reka-ui rows still reproduce.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
