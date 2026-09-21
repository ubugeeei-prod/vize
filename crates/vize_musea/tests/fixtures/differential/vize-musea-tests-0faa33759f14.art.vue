
<script setup lang="ts">
import MoshiDetailCard from './MoshiDetailCard.vue';
import {
  base,
  preparing,
  finished,
  notSucceeded,
} from './fixtures';

defineArt("./MoshiDetailCard.vue", {
  title: "Moshi Detail Card",
  category: "Features/MoshiDetail",
  tags: ["moshi", "detail"],
});

const localFixture = { id: "local" };
</script>

<art>
  <variant name="Available" default>
    <MoshiDetailCard :moshi-with-student="base" />
  </variant>
  <variant name="Preparing">
    <MoshiDetailCard :moshi-with-student="preparing" :fallback="localFixture" />
  </variant>
</art>
