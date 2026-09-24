<script setup lang="ts">
import "./theme.css";
import MuseaButton from "./components/MuseaButton.vue";
import MuseaInput from "./components/MuseaInput.vue";
import MuseaCard from "./components/MuseaCard.vue";
import MuseaBadge from "./components/MuseaBadge.vue";
import MuseaAlert from "./components/MuseaAlert.vue";
import MuseaAvatar from "./components/MuseaAvatar.vue";
</script>

<template>
  <div class="app">
    <h1>Vite + Vue + Musea Example</h1>
    <p>Visit <a href="/__musea__">/__musea__</a> to see the component gallery.</p>

    <section class="section" aria-label="Buttons">
      <h2>Buttons</h2>
      <div class="demo">
        <MuseaButton>Default</MuseaButton>
        <MuseaButton variant="primary">Primary</MuseaButton>
        <MuseaButton variant="secondary">Secondary</MuseaButton>
        <MuseaButton disabled>Disabled</MuseaButton>
      </div>
    </section>

    <section class="section" aria-label="Inputs">
      <h2>Inputs</h2>
      <div class="demo demo--column">
        <MuseaInput label="Text input" placeholder="Text input..." />
        <MuseaInput label="Search" type="search" placeholder="Search..." />
        <MuseaInput
          label="Required field"
          error="This field is required"
          placeholder="With error..."
        />
      </div>
    </section>

    <section class="section" aria-label="Badges">
      <h2>Badges</h2>
      <div class="demo">
        <MuseaBadge>Default</MuseaBadge>
        <MuseaBadge variant="success">Success</MuseaBadge>
        <MuseaBadge variant="warning">Warning</MuseaBadge>
        <MuseaBadge variant="error">Error</MuseaBadge>
        <MuseaBadge variant="info">Info</MuseaBadge>
      </div>
    </section>

    <section class="section" aria-label="Cards">
      <h2>Cards</h2>
      <div class="demo">
        <MuseaCard title="Default Card" description="A simple card component.">
          <MuseaButton variant="primary">Action</MuseaButton>
        </MuseaCard>
        <MuseaCard
          title="Featured"
          description="A card with image header."
          image="#6b5090"
          variant="elevated"
        >
          <MuseaButton variant="primary">View</MuseaButton>
        </MuseaCard>
      </div>
    </section>

    <section class="section" aria-label="Alerts">
      <h2>Alerts</h2>
      <div class="demo demo--column demo--alerts">
        <MuseaAlert>This is a default informational alert.</MuseaAlert>
        <MuseaAlert variant="success">Operation completed successfully.</MuseaAlert>
        <MuseaAlert variant="warning">Please review before proceeding.</MuseaAlert>
        <MuseaAlert variant="error">Something went wrong. Please try again.</MuseaAlert>
        <MuseaAlert variant="success" closable>This alert can be dismissed.</MuseaAlert>
      </div>
    </section>

    <section class="section" aria-label="Avatars">
      <h2>Avatars</h2>
      <div class="demo demo--centered">
        <MuseaAvatar name="Jane Doe" size="sm" />
        <MuseaAvatar name="Jane Doe" />
        <MuseaAvatar name="Jane Doe" size="lg" />
        <div class="avatar-group">
          <MuseaAvatar name="Alice" class="avatar-group__item" />
          <MuseaAvatar name="Bob" class="avatar-group__item" />
          <MuseaAvatar name="Charlie" class="avatar-group__item" />
        </div>
      </div>
    </section>
  </div>
</template>

<style>
body {
  background: var(--musea-paper);
  color: var(--musea-ink);
  margin: 0;
}

.app {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem;
  font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
}

h1 {
  color: var(--musea-ink);
  font-weight: 700;
}

h2 {
  color: var(--musea-text-subtle);
  font-size: 1.125rem;
  font-weight: 600;
  margin-block-end: 0.75rem;
}

a {
  color: var(--musea-ink);
  font-weight: 500;
}

a:hover {
  color: var(--musea-accent);
}

.section {
  margin-block-start: 2rem;
}

.demo {
  display: flex;
  gap: 1rem;
  flex-wrap: wrap;
  align-items: flex-start;
}

.demo--alerts {
  max-width: 480px;
}

.demo--centered {
  align-items: center;
}

.avatar-group {
  display: flex;
}

.avatar-group__item {
  border: 2px solid var(--musea-paper);

  & + & {
    margin-inline-start: -8px;
  }
}

.demo--column {
  flex-direction: column;
  max-width: 320px;
}
</style>
