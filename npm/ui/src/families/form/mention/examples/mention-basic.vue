<!-- Comment textarea that suggests teammates after typing "@" and inserts the chosen handle. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { Mention, MentionContent, MentionEmpty, MentionInput, MentionItem } from "../mention.ts";

interface Teammate {
  readonly handle: string;
  readonly name: string;
}

const teammates: readonly Teammate[] = [
  { handle: "ada", name: "Ada Lovelace" },
  { handle: "alan", name: "Alan Turing" },
  { handle: "grace", name: "Grace Hopper" },
];
const comment = ref("Thanks for the review, ");
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Comment</span>
    <Mention
      v-slot="{ filteredItems }"
      v-model="comment"
      :items="teammates"
      :item-text="(teammate: Teammate) => teammate.handle"
    >
      <MentionInput name="comment" :rows="3" :aria-labelledby="labelId" />
      <MentionContent aria-label="Teammates">
        <MentionItem v-for="teammate in filteredItems" :key="teammate.handle" :value="teammate">
          {{ teammate.name }} (@{{ teammate.handle }})
        </MentionItem>
        <MentionEmpty>No teammates match</MentionEmpty>
      </MentionContent>
    </Mention>
  </div>
</template>
