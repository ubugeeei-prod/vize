<script setup lang="ts">
import type { InspectorDiff } from "../../wasm/types/inspector";
import { dumpTokens } from "./dumpLines";
import { withTokenOffsets } from "../../utils/withTokenOffsets";

const { diff, before, after } = defineProps<{
  diff: InspectorDiff;
  /** The page compared against, e.g. `Lowered` or `v-slot`. */
  before: string;
  /** The page on screen. */
  after: string;
}>();
</script>

<template>
  <div class="level-diff">
    <p class="level-diff-summary">
      <template v-if="diff.stats.additions === 0 && diff.stats.removals === 0">
        {{ after }} left the page byte-for-byte identical to {{ before }}: its product is facts, not
        tree edits.
      </template>
      <template v-else>
        {{ after }} against {{ before }}: {{ diff.stats.additions }} added,
        {{ diff.stats.removals }} removed, {{ diff.stats.unchanged }} unchanged
      </template>
    </p>
    <div class="level-dump" aria-label="Page difference">
      <div
        v-for="line in diff.lines"
        :key="`${line.kind}:${line.leftLine}:${line.rightLine}`"
        :class="['level-line', `diff-${line.kind}`]"
      >
        <span class="level-ln">{{ line.rightLine ?? line.leftLine ?? "" }}</span>
        <span class="level-diff-mark" aria-hidden="true">{{
          line.kind === "add" ? "+" : line.kind === "remove" ? "−" : " "
        }}</span>
        <span class="level-code"
          ><span
            v-for="token in withTokenOffsets(dumpTokens(line.text))"
            :key="token.offset"
            :class="`tk-${token.type}`"
            >{{ token.text }}</span
          ><template v-if="line.text.length === 0">&#160;</template></span
        >
      </div>
    </div>
  </div>
</template>
