<script setup lang="ts">
import { ref } from "vue";
import { useIdGenerator } from "@vizejs/composable/use-id-generator";

const nextId = useIdGenerator({ prefix: "guest" });
const guests = ref([{ id: nextId("name"), name: "" }]);

function addGuest() {
  guests.value = [...guests.value, { id: nextId("name"), name: "" }];
}
</script>

<template>
  <div class="composable-example">
    <p>Give every repeated field its own id so each label points at the right input.</p>
    <div v-for="(guest, index) in guests" :key="guest.id">
      <label :for="guest.id">Guest {{ index + 1 }} name</label>
      <input :id="guest.id" v-model="guest.name" />
    </div>
    <button type="button" @click="addGuest">Add guest</button>
    <output aria-live="polite">{{ guests.length }} guest fields with unique ids</output>
  </div>
</template>
