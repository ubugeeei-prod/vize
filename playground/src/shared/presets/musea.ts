/**
 * Musea Playground Preset
 *
 * This preset demonstrates the Musea story format:
 * - defineArt() for component documentation and target resolution
 * - Multiple <variant> blocks for different component states
 * - CSS custom properties for design tokens
 *
 * Used to showcase story parsing and CSF generation.
 *
 * Note: This file is separate from the Vue component to avoid
 * linting issues with embedded Vue code in template literals.
 */

export const ART_PRESET = `<script setup lang="ts">
defineArt('./Button.vue', {
  title: 'Button',
  description: 'A versatile action control for forms, dialogs, and toolbar workflows.',
  category: 'Components',
  tags: ['button', 'action', 'form'],
  status: 'ready',
})
</script>

<art>
  <variant name="Default" default>
    <Button>Default Button</Button>
  </variant>

  <variant name="Primary">
    <Button variant="primary">Primary Button</Button>
  </variant>

  <variant name="Secondary">
    <Button variant="secondary">Secondary Button</Button>
  </variant>

  <variant name="With Icon">
    <Button variant="primary" icon="plus">Add Item</Button>
  </variant>

  <variant name="Disabled">
    <Button variant="primary" disabled>Disabled</Button>
  </variant>
</art>

<style>
:root {
  --color-primary: #121212;
  --color-primary-hover: #2a2a2a;
  --color-secondary: #6b5090;
  --color-secondary-hover: #5a4080;
  --color-success: #2d6a35;
  --color-warning: #8b7040;
  --color-error: #a04040;
  --color-text: #121212;
  --color-text-muted: #6b6b6b;
  --color-background: #e6e2d6;
  --color-surface: #ddd9cd;
  --color-border: #c8c4b8;

  --spacing-xs: 4px;
  --spacing-sm: 8px;
  --spacing-md: 16px;
  --spacing-lg: 24px;
  --spacing-xl: 32px;

  --radius-sm: 4px;
  --radius-md: 8px;
  --radius-lg: 12px;

  --font-size-sm: 12px;
  --font-size-md: 14px;
  --font-size-lg: 16px;
}
</style>
`;
