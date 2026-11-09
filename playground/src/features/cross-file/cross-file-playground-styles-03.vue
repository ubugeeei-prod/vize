<script setup lang="ts">
defineOptions({ name: "CrossFilePlaygroundStyles03" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.issue-related {
  margin-top: 6px;
  padding-top: 6px;
  border-top: 1px solid var(--border-primary);
}

.related-item {
  display: flex;
  gap: 8px;
  font-size: 0.625rem;
  color: var(--text-muted);
  margin-bottom: 2px;
}

.related-loc {
  font-family: "JetBrains Mono", monospace;
  color: var(--text-secondary);
}

/* === Responsive === */
@media (max-width: 1200px) {
  .cross-file-playground {
    grid-template-columns: 180px 1fr 280px;
  }
}

@media (max-width: 900px) {
  .cross-file-playground {
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr auto;
  }

  .sidebar {
    flex-direction: row;
    border-right: none;
    border-bottom: 1px solid var(--border-primary);
    overflow-x: auto;
  }

  .sidebar-section {
    flex-direction: row;
    min-width: max-content;
  }

  .diagnostics-pane {
    border-left: none;
    border-top: 1px solid var(--border-primary);
    max-height: 300px;
  }
}
</style>
