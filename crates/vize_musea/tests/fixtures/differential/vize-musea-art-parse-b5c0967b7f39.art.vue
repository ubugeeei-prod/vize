<art title="Card" description="A flexible card component with multiple layout options" component="./Card.vue" category="molecules" tags="layout,container,content,interactive" status="ready" order="10">
  <variant name="Default" default>
    <Card>
      <template #header>
        <h3>Card Title</h3>
      </template>
      <p>Card content goes here. This is a simple card with default styling.</p>
      <template #footer>
        <Button>Action</Button>
      </template>
    </Card>
  </variant>
  <variant name="With Image" args='{"image":"/placeholder.jpg","imageAlt":"Placeholder"}'>
    <Card :image="args.image" :image-alt="args.imageAlt">
      <template #header>
        <h3>Featured Card</h3>
      </template>
      <p>A card with a featured image at the top.</p>
    </Card>
  </variant>
  <variant name="Horizontal" args='{"layout":"horizontal"}'>
    <Card :layout="args.layout">
      <template #media>
        <img src="/thumbnail.jpg" alt="Thumbnail" />
      </template>
      <h3>Horizontal Layout</h3>
      <p>Content appears beside the media in horizontal layout.</p>
    </Card>
  </variant>
  <variant name="Interactive" args='{"clickable":true,"hoverable":true}'>
    <Card :clickable="args.clickable" :hoverable="args.hoverable" @click="handleClick">
      <h3>Interactive Card</h3>
      <p>Click or hover to see the interaction effects.</p>
    </Card>
  </variant>
  <variant name="Loading" args='{"loading":true}'>
    <Card :loading="args.loading">
      <h3>Loading State</h3>
      <p>Shows skeleton loading animation.</p>
    </Card>
  </variant>
  <variant name="Mobile View" viewport="375x667">
    <Card>
      <h3>Mobile Optimized</h3>
      <p>This variant shows how the card looks on mobile devices.</p>
    </Card>
  </variant>
  <variant name="Tablet View" viewport="768x1024">
    <Card>
      <h3>Tablet View</h3>
      <p>Optimized layout for tablet-sized screens.</p>
    </Card>
  </variant>
  <variant name="High DPI" viewport="375x667@2">
    <Card>
      <h3>Retina Display</h3>
      <p>Testing on high DPI displays.</p>
    </Card>
  </variant>
  <variant name="Dark Theme" skip-vrt>
    <Card class="dark-theme">
      <h3>Dark Theme</h3>
      <p>Card with dark theme styling. Skipped in VRT due to theme variations.</p>
    </Card>
  </variant>
  <variant name="Custom Styling" skip-vrt args='{"borderRadius":"16px","shadow":"xl"}'>
    <Card :style="{ borderRadius: args.borderRadius }" :shadow="args.shadow">
      <h3>Custom Styled</h3>
      <p>Demonstrates custom styling options.</p>
    </Card>
  </variant>
</art>

<script setup lang="ts">
import { ref } from 'vue'
import Card from './Card.vue'
import Button from '../Button/Button.vue'

const handleClick = () => {
  console.log('Card clicked')
}
</script>

<style scoped>
.art-container {
  padding: 24px;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 24px;
  background: #f5f5f5;
}

.dark-theme {
  --card-bg: #1a1a1a;
  --card-text: #ffffff;
  --card-border: #333333;
}
</style>

<style>
/* Global styles for the gallery */
.musea-variant {
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  overflow: hidden;
}
</style>
