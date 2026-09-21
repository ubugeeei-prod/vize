**Art Block**

Defines component variants. Metadata and target component are declared with `defineArt` in `<script setup>`.

```vue
<script setup lang="ts">
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="Primary" default>
    <Button>Click</Button>
  </variant>
</art>
```