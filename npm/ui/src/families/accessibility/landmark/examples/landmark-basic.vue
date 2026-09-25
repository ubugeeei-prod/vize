<!-- Page regions as native landmarks; F6 and Shift+F6 cycle focus between them. -->
<script setup lang="ts">
import { Landmark, LandmarkProvider } from "../landmark.ts";
</script>

<template>
  <div>
    <LandmarkProvider v-slot="{ landmarks }">
      <Landmark role="banner">
        <p>Acme docs</p>
      </Landmark>
      <Landmark role="navigation" aria-label="Primary">
        <a href="#guides">Guides</a>
      </Landmark>
      <Landmark role="main">
        <p>Press F6 to move between regions.</p>
        <output>{{ landmarks.length }} landmarks registered</output>
      </Landmark>
    </LandmarkProvider>
  </div>
</template>
