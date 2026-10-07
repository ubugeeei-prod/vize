<script setup lang="ts">
defineArt("./my-card.vue", { title: "MyCard", category: "Layout" });
</script>

<art>
  <variant name="With action" default>
    <MyCard title="Hello">
      <MyButton>OK</MyButton>
      <my-button>OK</my-button>
      <MissingWidget />
    </MyCard>
  </variant>
</art>
