<!-- Scrolling sponsor strip with a pause button; duplicate copies are hidden from assistive technology. -->
<script setup lang="ts">
import { Marquee, MarqueeContent, MarqueePauseButton } from "../marquee.ts";

const sponsors = ["Acme Robotics", "Globex", "Initech", "Umbrella Labs"];
</script>

<template>
  <Marquee aria-label="Conference sponsors" :speed="40" pause-on-hover>
    <MarqueePauseButton v-slot="{ playing }">{{ playing ? "Pause" : "Play" }}</MarqueePauseButton>
    <MarqueeContent>
      <ul>
        <li v-for="sponsor in sponsors" :key="sponsor">{{ sponsor }}</li>
      </ul>
    </MarqueeContent>
  </Marquee>
</template>
