<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="Numbers" default>
    <Button v-for="entry in [1, 2]" :label="entry" />
  </variant>
  <variant name="Strings">
    <Button v-for="entry in ['a', 'b']" :label="entry" />
  </variant>
</art>
