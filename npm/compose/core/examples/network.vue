<script setup lang="ts">
import { ref, useId } from "vue";
import { useNetwork } from "@vizejs/composable/network";

const draftId = useId();
const draft = ref("");
const reviews = ref(0);
const { isSupported, isOnline, onlineAt, offlineAt, effectiveType, downlink, rtt, saveData } =
  useNetwork();

function eventTime(value: number | null): string {
  return value === null ? "not observed" : new Date(value).toISOString();
}
</script>

<template>
  <div class="composable-example">
    <label :for="draftId">Local guide draft</label>
    <input :id="draftId" v-model="draft" />
    <button type="button" @click="() => (reviews += 1)">Review local draft</button>
    <p>Local review count: {{ reviews }}.</p>
    <output aria-live="polite">Browser status: {{ isOnline ? "online" : "offline" }}</output>
    <p v-if="!isOnline">The browser reports offline. Your local draft remains editable.</p>
    <dl>
      <dt>Last online event</dt>
      <dd>{{ eventTime(onlineAt) }}</dd>
      <dt>Last offline event</dt>
      <dd>{{ eventTime(offlineAt) }}</dd>
      <dt>Network Information API</dt>
      <dd>{{ isSupported ? "available" : "unavailable" }}</dd>
      <dt>Estimated connection class</dt>
      <dd>{{ effectiveType ?? "unknown" }}</dd>
      <dt>Estimated downlink</dt>
      <dd>{{ downlink === null ? "unknown" : `${downlink} Mbit/s` }}</dd>
      <dt>Estimated round trip</dt>
      <dd>{{ rtt === null ? "unknown" : `${rtt} ms` }}</dd>
      <dt>Reduced data requested</dt>
      <dd>{{ saveData ? "yes" : "no" }}</dd>
    </dl>
    <p>
      Use your browser's offline switch or change the actual connection to observe these flags and
      event times. Online status is the browser's report; it does not guarantee that a service is
      reachable. Connection estimates are optional and may be unknown. No request or draft save is
      sent here. Server rendering starts online with no observed event or quality measurements; the
      client reads the real browser after mounting.
    </p>
  </div>
</template>
