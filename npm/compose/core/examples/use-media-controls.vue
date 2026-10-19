<script setup lang="ts">
import { useTemplateRef } from "vue";
import { useMediaControls } from "@vizejs/composable/use-media-controls";
import { useMounted } from "@vizejs/composable/use-mounted";
import { useObjectUrl } from "@vizejs/composable/use-object-url";

// A one-second silent WAV generated in the browser stands in for your audio file.
function silentWav(seconds: number, rate = 8000): Blob {
  const samples = seconds * rate;
  const view = new DataView(new ArrayBuffer(44 + samples));
  const text = (offset: number, value: string) =>
    [...value].forEach((char, index) => view.setUint8(offset + index, char.charCodeAt(0)));
  text(0, "RIFF");
  view.setUint32(4, 36 + samples, true);
  text(8, "WAVEfmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, rate, true);
  view.setUint32(28, rate, true);
  view.setUint16(32, 1, true);
  view.setUint16(34, 8, true);
  text(36, "data");
  view.setUint32(40, samples, true);
  for (let index = 0; index < samples; index += 1) view.setUint8(44 + index, 128);
  return new Blob([view.buffer], { type: "audio/wav" });
}

const audio = useTemplateRef<HTMLAudioElement>("audio");
// The generated file only exists in the browser, so create the element after hydration.
const mounted = useMounted();
const captions = `data:text/vtt,${encodeURIComponent("WEBVTT\n\n00:00.000 --> 00:01.000\n(silence)\n")}`;
const source = useObjectUrl(silentWav(1));
const { playing, currentTime, duration, muted, rate } = useMediaControls(audio, {
  src: () => source.value ?? "",
});
</script>

<template>
  <div class="composable-example">
    <p>Build custom audio controls on top of a native media element.</p>
    <audio v-if="mounted" ref="audio" preload="metadata">
      <track kind="captions" srclang="en" label="English" :src="captions" />
    </audio>
    <div class="example-actions">
      <button type="button" @click="() => (playing = !playing)">
        {{ playing ? "Pause" : "Play" }}
      </button>
      <button type="button" @click="() => (muted = !muted)">{{ muted ? "Unmute" : "Mute" }}</button>
      <button type="button" @click="() => (rate = rate === 1 ? 2 : 1)">Speed {{ rate }}×</button>
      <button type="button" @click="() => (currentTime = 0.5)">Jump to 0.5 s</button>
    </div>
    <output aria-live="off">{{ currentTime.toFixed(1) }} / {{ duration.toFixed(1) }} s</output>
  </div>
</template>
