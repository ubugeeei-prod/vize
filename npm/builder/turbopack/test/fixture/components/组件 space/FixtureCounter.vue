<script setup lang="ts">
import { ref, useCssModule } from "vue";
import { label } from "@fixture/label";
import Child from "@fixture/FixtureChild.vue";
const props = defineProps<{ initial: number }>();
const count = ref(props.initial);
const emoji: string = "🐱";
const tint = ref("rgb(22,33,44)");
const captured = useCssModule();
const theme = useCssModule("theme");
</script>
<template>
  <section>
    <button id="counter" class="counter" :class="$style.button" @click="count++">
      {{ label }} {{ count }} {{ emoji }}
    </button>
    <div id="captured" :class="captured.button">captured</div>
    <div id="named" :class="theme.label">named</div>
    <div id="scss" class="sass">scss</div>
    <div id="external" class="external">external</div>
    <div id="bound" class="tone">bound</div>
    <img id="pixel" src="./pixel.png" alt="pixel" />
    <Child />
  </section>
</template>
<style scoped>
@import "./theme.css";
.counter {
  color: rgb(12, 34, 56);
  background-image: url("./dot.svg");
}
.tone {
  color: v-bind(tint);
}
</style>
<style module>
.button {
  padding: 7px;
}
</style>
<style module="theme">
.label {
  font-weight: 700;
}
</style>
<style scoped lang="scss">
$margin: 9px;
.sass {
  margin-left: $margin;
}
</style>
<style src="./nested/external.css" scoped></style>
