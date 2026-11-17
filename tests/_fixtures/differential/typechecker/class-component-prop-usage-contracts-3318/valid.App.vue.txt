<script lang="ts">
import { Vue, Options } from "vue-property-decorator";
import HelloDecorator from "./HelloDecorator.vue";

@Options({
  components: { HelloDecorator },
})
export default class App extends Vue {
  title = "Class component parity";

  get heading(): string {
    return this.title.toUpperCase();
  }
}
</script>

<template>
  <main>
    <h1>{{ heading }}</h1>
    <HelloDecorator name="World" />
  </main>
</template>
