<script setup lang="ts">
defineOptions({ name: "AppStyles04" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.group-title {
  flex: 1;
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--text-secondary);
}

.group-count {
  font-size: 0.625rem;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.group-vars {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  padding: 0.625rem 0.75rem;
}

.var-chip {
  padding: 0.25rem 0.5rem;
  font-size: 0.6875rem;
  font-family: var(--font-mono);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-primary);
  border-radius: 3px;
  color: var(--text-secondary);
  transition: all 0.15s;
}

.var-chip:hover {
  border-color: var(--border-primary);
  color: var(--text-primary);
}

/* ===================== */
/* Tokens Tab - Professional UI */
/* ===================== */

.tokens-output {
  padding: 0;
}

.tokens-output {
  & h4 {
    font-size: 0.6875rem;
    font-weight: 500;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
    margin: 1rem 0 0.5rem;
  }
}

.tokens-output {
  & h4:first-of-type {
    margin-top: 0;
  }
}

.token-stats {
  display: flex;
  gap: 0.375rem;
  flex-wrap: wrap;
}

.token-stats {
  & .stat-card {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.375rem 0.625rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-primary);
    border-radius: 3px;
  }
}

.token-stats {
  & .stat-value {
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-primary);
    font-family: var(--font-mono);
  }
}

.token-stats {
  & .stat-label {
    font-size: 0.625rem;
    color: var(--text-muted);
  }
}

.token-stream {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 300px;
  overflow-y: auto;
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
}

.token-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.375rem 0.625rem;
  background: transparent;
  border-bottom: 1px solid var(--border-primary);
  transition: all 0.15s;
}

.token-item:last-child {
  border-bottom: none;
}

.token-item:hover {
  background: var(--bg-tertiary);
}

.token-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  padding: 0.125rem 0.375rem;
  border-radius: 3px;
  font-size: 0.5625rem;
  font-weight: 500;
  background: var(--bg-tertiary);
  color: var(--text-muted);
  flex-shrink: 0;
}

.token-content {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-width: 0;
}

.token-main {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  min-width: 0;
}

.tokens-output {
  & .token-name {
    font-size: 0.75rem;
    font-weight: 500;
    font-family: var(--font-mono);
    color: var(--text-primary);
  }
}

.token-value-text {
  font-size: 0.6875rem;
  font-family: var(--font-mono);
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 200px;
}

.token-location {
  font-size: 0.5625rem;
  font-family: var(--font-mono);
  color: var(--text-muted);
  flex-shrink: 0;
}

.token-groups {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.token-group {
  background: var(--bg-secondary);
  border: 1px solid var(--border-primary);
  border-radius: 4px;
  overflow: hidden;
}

.tokens-output {
  & .group-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0.75rem;
    background: var(--bg-tertiary);
    border-bottom: 1px solid var(--border-primary);
  }
}

.tokens-output {
  & .group-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
    padding: 0.125rem 0.375rem;
    border-radius: 3px;
    font-size: 0.5625rem;
    font-weight: 500;
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-muted);
    flex-shrink: 0;
  }
}

.tokens-output {
  & .group-title {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--text-secondary);
  }
}

.tokens-output {
  & .group-count {
    margin-inline-start: auto;
    font-size: 0.625rem;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }
}

.group-tokens {
  display: flex;
  flex-wrap: wrap;
  gap: 0.375rem;
  padding: 0.625rem 0.75rem;
}

.group-token-chip {
  display: inline-block;
  padding: 0.25rem 0.5rem;
  font-size: 0.6875rem;
  font-family: var(--font-mono);
  background: var(--bg-tertiary);
  border-radius: 3px;
  color: var(--text-secondary);
  border: 1px solid var(--border-primary);
  transition: all 0.15s;
}

.group-token-chip:hover {
  color: var(--text-primary);
}

.more-indicator {
  font-size: 0.625rem;
  color: var(--text-muted);
  padding: 0.25rem 0.5rem;
}

@media (max-width: 768px) {
  .token-stats {
    flex-direction: column;
  }
}
</style>
