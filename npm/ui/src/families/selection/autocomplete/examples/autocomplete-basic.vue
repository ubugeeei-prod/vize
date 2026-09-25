<!-- Search box that suggests matching cities and remembers recent choices while the query is empty. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import {
  Autocomplete,
  AutocompleteContent,
  AutocompleteEmpty,
  AutocompleteInput,
  AutocompleteItem,
} from "../autocomplete.ts";

const cities = ["Amsterdam", "Barcelona", "Lisbon", "Osaka", "Tokyo", "Toronto"];
const city = ref<string | null>(null);
const recent = ref<readonly string[]>(["Lisbon"]);
const labelId = useId();
</script>

<template>
  <div>
    <span :id="labelId">Destination</span>
    <Autocomplete
      v-slot="{ suggestions, showingHistory, clearHistory }"
      v-model="city"
      v-model:history="recent"
      :items="cities"
      name="destination"
    >
      <AutocompleteInput :aria-labelledby="labelId" placeholder="Search cities" />
      <AutocompleteContent>
        <AutocompleteItem v-for="entry in suggestions" :key="entry" :value="entry">
          {{ entry }}
        </AutocompleteItem>
        <AutocompleteEmpty>No matching cities</AutocompleteEmpty>
        <button v-if="showingHistory" type="button" @click="clearHistory">Clear recent</button>
      </AutocompleteContent>
    </Autocomplete>
    <output>Destination: {{ city ?? "none" }}</output>
  </div>
</template>
