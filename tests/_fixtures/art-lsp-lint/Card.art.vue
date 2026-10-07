<script setup lang="ts">
import MyCard from "./my-card.vue";
</script>

<art title="MyCard" component="./my-card.vue">
  <variant name="Default" default>
    <MyCard>
      <a href="#">Read more</a>
      <img src="/logo.png" />
    </MyCard>
  </variant>
</art>
