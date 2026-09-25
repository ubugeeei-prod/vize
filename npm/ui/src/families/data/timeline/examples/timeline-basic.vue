<!-- Labelled release timeline where the root value marks the current milestone. -->
<script setup lang="ts">
import {
  TimelineConnector,
  TimelineContent,
  TimelineIndicator,
  TimelineItem,
  TimelineRoot,
  TimelineTime,
} from "../timeline.ts";

const releases = [
  { value: "alpha", title: "Alpha", date: "2026-06-01", label: "June 1" },
  { value: "beta", title: "Beta", date: "2026-08-15", label: "August 15" },
  { value: "stable", title: "Stable", date: "2026-10-24", label: "October 24" },
] as const;
</script>

<template>
  <TimelineRoot aria-label="Release history" value="beta">
    <TimelineItem v-for="release in releases" :key="release.value" :value="release.value">
      <TimelineIndicator />
      <TimelineConnector />
      <TimelineContent>
        <strong>{{ release.title }}</strong>
        <TimelineTime :datetime="release.date">{{ release.label }}</TimelineTime>
      </TimelineContent>
    </TimelineItem>
  </TimelineRoot>
</template>
