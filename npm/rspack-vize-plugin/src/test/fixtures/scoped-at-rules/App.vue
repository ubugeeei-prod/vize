<script setup lang="ts">
const msg: string = "nested at-rules";
</script>

<template>
  <div class="responsive">
    <p>{{ msg }}</p>
  </div>
</template>

<style scoped>
.responsive {
  width: 100%;
}

@media (max-width: 768px) {
  .responsive {
    padding: 0.5rem;
  }

  .responsive p {
    font-size: 14px;
  }
}

@media (min-width: 1024px) {
  .responsive {
    max-width: 960px;
    margin: 0 auto;
  }
}

@supports (display: grid) {
  .responsive {
    display: grid;
    grid-template-columns: 1fr;
  }
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.responsive {
  animation: fade-in 0.3s ease;
}
</style>
