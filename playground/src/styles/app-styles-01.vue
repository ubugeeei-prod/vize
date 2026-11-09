<script setup lang="ts">
defineOptions({ name: "AppStyles01" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
/* Variables — Light (default) */
:global(:root) {
  --bg-primary: #e6e2d6;
  --bg-secondary: #ddd9cd;
  --bg-tertiary: #d4d0c4;
  --bg-panel: rgba(230, 226, 214, 0.95);
  --border-color: rgba(18, 18, 18, 0.15);
  --border-primary: rgba(18, 18, 18, 0.15);
  --text-primary: #121212;
  --text-secondary: #3a3a3a;
  --text-muted: #6b6b6b;
  --accent-rust: #121212;
  --accent-rust-light: #121212;
  --accent-metal: #121212;
  --accent-metal-dark: #6b6b6b;

  --font-mono: "JetBrains Mono", monospace;
  --font-sans: "Helvetica Neue", Helvetica, Arial, sans-serif;
  --radius: 8px;
  --radius-lg: 12px;
  --layer-header: 100;

  /* Semantic accent colors */
  --color-notice: #8b7040;
  --color-notice-bg: rgba(139, 112, 64, 0.1);
  --color-notice-border: rgba(139, 112, 64, 0.3);
  --color-success: #1f5b28;
  --color-success-bg: rgba(31, 91, 40, 0.12);
  --color-success-border: rgba(31, 91, 40, 0.34);
  --color-error: #8a2f2f;
  --color-error-bg: rgba(138, 47, 47, 0.1);
  --color-error-border: rgba(138, 47, 47, 0.32);
  --color-warning: #60430e;
  --color-warning-bg: rgba(96, 67, 14, 0.12);
  --color-warning-border: rgba(96, 67, 14, 0.34);
  --color-info: #4a6b8a;
  --color-info-bg: rgba(74, 107, 138, 0.1);
  --color-info-border: rgba(74, 107, 138, 0.28);
  --color-purple: #6b5090;
  --color-purple-bg: rgba(107, 80, 144, 0.15);
  --color-purple-border: rgba(107, 80, 144, 0.3);
  --color-teal: #3a7a70;
  --color-teal-bg: rgba(58, 122, 112, 0.15);
  --color-teal-border: rgba(58, 122, 112, 0.3);

  /* Readable code surfaces */
  --code-bg: #f3efe2;
  --code-gutter-bg: #ebe4d4;
  --code-border: rgba(18, 18, 18, 0.2);
  --code-foreground: #141414;
  --code-line-number: #5f5a52;

  /* Syntax highlighting (for code blocks in hints etc.) */
  --hl-keyword: #7a2f15;
  --hl-function: #6b4a00;
  --hl-string: #1b6a38;
  --hl-comment: #59554f;
  --hl-tag: #005f80;
  --hl-directive: #5a3d86;
  --hl-delimiter: #5c5750;
  --hl-type: #3d4b80;
  --hl-number: #73470d;
  --hl-property: #005f73;
  --hl-value: #40680f;
}

/* Variables — Dark */
:global(body[data-theme="dark"]) {
  --bg-primary: #121212;
  --bg-secondary: #1a1a1a;
  --bg-tertiary: #242424;
  --bg-panel: rgba(18, 18, 18, 0.95);
  --border-color: rgba(230, 226, 214, 0.15);
  --border-primary: rgba(230, 226, 214, 0.15);
  --text-primary: #e6e2d6;
  --text-secondary: #d4d1c6;
  --text-muted: #9a9890;
  --accent-rust: #e6e2d6;
  --accent-rust-light: #e6e2d6;
  --accent-metal: #e6e2d6;
  --accent-metal-dark: #8a8880;

  /* Semantic accent colors */
  --color-notice: #d4ba92;
  --color-notice-bg: rgba(212, 186, 146, 0.1);
  --color-notice-border: rgba(212, 186, 146, 0.25);
  --color-success: #a8b5a0;
  --color-success-bg: rgba(168, 181, 160, 0.1);
  --color-success-border: rgba(168, 181, 160, 0.3);
  --color-error: #d09090;
  --color-error-bg: rgba(208, 144, 144, 0.1);
  --color-error-border: rgba(208, 144, 144, 0.3);
  --color-warning: #d4ba92;
  --color-warning-bg: rgba(212, 186, 146, 0.12);
  --color-warning-border: rgba(212, 186, 146, 0.3);
  --color-info: #8aabca;
  --color-info-bg: rgba(138, 171, 202, 0.1);
  --color-info-border: rgba(138, 171, 202, 0.28);
  --color-purple: #b8a0d0;
  --color-purple-bg: rgba(184, 160, 208, 0.12);
  --color-purple-border: rgba(184, 160, 208, 0.28);
  --color-teal: #80bdb0;
  --color-teal-bg: rgba(128, 189, 176, 0.12);
  --color-teal-border: rgba(128, 189, 176, 0.28);

  /* Readable code surfaces */
  --code-bg: #171717;
  --code-gutter-bg: #202020;
  --code-border: rgba(230, 226, 214, 0.2);
  --code-foreground: #eee8db;
  --code-line-number: #b2aaa0;

  /* Syntax highlighting */
  --hl-keyword: #f2b8a0;
  --hl-function: #ffd08a;
  --hl-string: #a7d8a8;
  --hl-comment: #a39b8f;
  --hl-tag: #8ecdf0;
  --hl-directive: #d8b4fe;
  --hl-delimiter: #b8afa2;
  --hl-type: #c4c8ff;
  --hl-number: #f1bc78;
  --hl-property: #9dd4ff;
  --hl-value: #bade9b;
}

/* Base */
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  overflow: hidden;
}

/* Header */
.header {
  display: flex;
  align-items: center;
  padding: 16px 24px;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border-color);
  backdrop-filter: blur(10px);
  z-index: var(--layer-header);
  gap: 24px;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
}

.logo-icon {
  width: 40px;
  height: 40px;
}

.logo-icon {
  & svg {
    width: 100%;
    height: 100%;
  }
}

.logo-text {
  & h1 {
    font-size: 1.25rem;
    font-weight: 500;
    color: var(--text-primary);
  }
}

.logo-text {
  & .version {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-weight: 500;
  }
}

.wasm-status {
  font-size: 0.625rem;
  padding: 2px 6px;
  border-radius: 4px;
  margin-inline-start: 6px;
}

.wasm-status.loading {
  background: var(--color-notice-bg);
  color: var(--color-notice);
}

.wasm-status.ready {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.wasm-status.mock {
  background: rgba(184, 189, 201, 0.2);
  color: var(--accent-metal);
}

/* Main Tabs */
.main-tabs {
  display: flex;
  gap: 4px;
}

.main-tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-width: 64px;
  padding: 6px 10px;
  font-size: 0.8125rem;
  font-weight: 500;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  color: var(--text-muted);
  cursor: pointer;
  white-space: nowrap;
  transition:
    background 0.15s ease,
    color 0.15s ease,
    border-color 0.15s ease;
}

.main-tab {
  & .tab-name {
    font-weight: 600;
    text-align: center;
    width: 100%;
  }
}

.main-tab {
  & .tab-desc {
    font-size: 0.625rem;
    font-weight: 400;
    opacity: 0.7;
    margin-top: 1px;
    text-align: center;
    width: 100%;
  }
}

.main-tab:hover {
  color: var(--text-secondary);
  background: var(--bg-tertiary);
}

.main-tab.active {
  color: var(--text-primary);
  background: var(--bg-tertiary);
  border-color: var(--border-color);
}

.main-tab.active {
  & .tab-desc {
    opacity: 0.9;
  }
}

.options {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-inline-start: auto;
}

.option {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8125rem;
  color: var(--text-secondary);
}

.option {
  & select {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    color: var(--text-primary);
    padding: 6px 12px;
    border-radius: var(--radius);
    font-size: 0.8125rem;
    font-family: var(--font-sans);
    cursor: pointer;
    transition: border-color 0.2s;
  }
}

.option {
  & select:hover {
    border-color: var(--accent-metal);
  }
}

.option.checkbox {
  cursor: pointer;
  user-select: none;
}

.option.checkbox {
  & input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent-rust);
    cursor: pointer;
  }
}

.theme-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius);
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.2s;
}

.theme-toggle:hover {
  border-color: var(--accent-rust);
  color: var(--text-primary);
}

.github-link {
  display: flex;
  align-items: center;
  color: var(--text-muted);
  transition: color 0.2s;
  margin-inline-start: 8px;
}

.github-link:hover {
  color: var(--accent-rust);
}

/* Main */
.main {
  display: grid;
  grid-template-columns: 1fr 1fr;
  flex: 1;
  overflow: hidden;
}

/* Full-width playgrounds */
.cross-file-playground,
.patina-playground,
.croquis-playground,
.typecheck-playground,
.musea-playground {
  grid-column: 1 / -1;
}

.panel {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  min-height: 0;
}

.editor-container {
  flex: 1;
  overflow: hidden;
  min-height: 0;
  position: relative;
}

.input-panel {
  border-right: 1px solid var(--border-color);
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--border-color);
  flex-wrap: wrap;
  gap: 4px 0;
}
</style>
