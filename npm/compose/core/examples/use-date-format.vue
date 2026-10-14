<script setup lang="ts">
import { ref } from "vue";
import { useDateFormat } from "@vizejs/composable/use-date-format";

const start = Date.UTC(2026, 9, 24, 1, 0);
const timeZone = ref("UTC");
const pattern = ref("YYYY-MM-DD HH:mm");
const formatted = useDateFormat(start, pattern, { timeZone });
</script>

<template>
  <div class="composable-example">
    <p>Show a talk's start time with a format string, in the time zone the reader chooses.</p>
    <label>
      Time zone
      <select v-model="timeZone">
        <option value="UTC">UTC</option>
        <option value="Asia/Tokyo">Tokyo</option>
        <option value="America/New_York">New York</option>
      </select>
    </label>
    <label>
      Format
      <select v-model="pattern">
        <option value="YYYY-MM-DD HH:mm">YYYY-MM-DD HH:mm</option>
        <option value="ddd D MMM, h:mm A">ddd D MMM, h:mm A</option>
      </select>
    </label>
    <output aria-live="polite">Starts {{ formatted }}</output>
  </div>
</template>
