<script setup lang="ts">
import type { InspectorDiff } from "../../wasm/types/inspector";
import { folioTokens } from "./folioLines";

defineProps<{
  diff: InspectorDiff;
  /** The page compared against, e.g. `Lowered` or `v-slot`. */
  before: string;
  /** The page on screen. */
  after: string;
}>();
</script>

<template>
  <div class="davinci-diff">
    <p class="davinci-diff-summary">
      <template v-if="diff.stats.additions === 0 && diff.stats.removals === 0">
        {{ after }} left the page byte-for-byte identical to {{ before }}: its product is facts, not
        tree edits.
      </template>
      <template v-else>
        {{ after }} against {{ before }}: {{ diff.stats.additions }} added,
        {{ diff.stats.removals }} removed, {{ diff.stats.unchanged }} unchanged
      </template>
    </p>
    <div class="davinci-folio" aria-label="Page difference">
      <div
        v-for="(line, index) in diff.lines"
        :key="index"
        :class="['davinci-line', `diff-${line.kind}`]"
      >
        <span class="davinci-ln">{{ line.rightLine ?? line.leftLine ?? "" }}</span>
        <span class="davinci-diff-mark" aria-hidden="true">{{
          line.kind === "add" ? "+" : line.kind === "remove" ? "−" : " "
        }}</span>
        <span class="davinci-code"
          ><span
            v-for="(token, ti) in folioTokens(line.text)"
            :key="ti"
            :class="`tk-${token.type}`"
            >{{ token.text }}</span
          ><template v-if="line.text.length === 0">&#160;</template></span
        >
      </div>
    </div>
  </div>
</template>
