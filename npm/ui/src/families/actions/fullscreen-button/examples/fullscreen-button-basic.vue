<!-- Toggles fullscreen for a report figure instead of the whole document. -->
<script setup lang="ts">
import { useTemplateRef } from "vue";

import { FullscreenButton } from "../fullscreen-button.ts";

const reportRef = useTemplateRef<HTMLElement>("report");

function resolveReport(): HTMLElement | null {
  return reportRef.value;
}
</script>

<template>
  <figure ref="report">
    <figcaption>Quarterly revenue report</figcaption>
    <table>
      <thead>
        <tr>
          <th scope="col">Quarter</th>
          <th scope="col">Revenue</th>
        </tr>
      </thead>
      <tbody>
        <tr>
          <td>Q1</td>
          <td>$1.2M</td>
        </tr>
        <tr>
          <td>Q2</td>
          <td>$1.5M</td>
        </tr>
      </tbody>
    </table>
    <FullscreenButton
      :target="resolveReport"
      enter-label="View report fullscreen"
      exit-label="Exit fullscreen"
    />
  </figure>
</template>
