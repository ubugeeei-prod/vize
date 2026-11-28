<script setup lang="ts">
defineOptions({ name: "DavinciWorkbenchLayersStyles" });
</script>

<!-- Shared selectors style the IR panes and their generated dump rows. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
.dv-layer {
  --layer-color: var(--dv-l1);
  display: flex;
  flex-direction: column;
  position: relative;
  min-width: 0;
  min-height: 0;
  border-right: 1px solid var(--border-primary);
}
.dv-l2 {
  --layer-color: var(--dv-l2);
}
.dv-l3 {
  --layer-color: var(--dv-l3);
}
.dv-l4 {
  --layer-color: var(--dv-l4);
  border-right: 0;
}
.dv-layer-header {
  position: relative;
  flex-shrink: 0;
  padding: 12px 11px 10px;
  min-height: 76px;
  border-bottom: 1px solid var(--border-primary);
  background: var(--bg-primary);
}
.dv-layer-heading {
  display: flex;
  align-items: center;
  gap: 7px;
  h3 {
    margin: 0;
    font-size: 0.875rem;
    font-weight: 650;
    letter-spacing: -0.3px;
  }
}
.dv-layer-alias {
  font: 10px var(--font-mono);
  color: var(--text-muted);
}
.dv-layer-header {
  p {
    margin: 7px 0 8px;
    color: var(--text-muted);
    font-size: 0.625rem;
  }
}
.dv-layer-facts {
  display: flex;
  justify-content: space-between;
  gap: 4px;
  font: 9px var(--font-mono);
  color: var(--text-muted);
  code {
    color: var(--layer-color);
  }
}
.dv-expand {
  margin-inline-start: auto;
  border: none;
  background: none;
  color: var(--text-muted);
  cursor: pointer;
  font-size: 1rem;
  line-height: 1;
  padding: 2px;
}
.dv-expand:hover {
  color: var(--layer-color);
}
.dv-page-picker {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 7px 9px;
  min-height: 38px;
  border-bottom: 1px solid var(--border-primary);
  select {
    min-width: 0;
    flex: 1;
    padding: 3px 5px;
    border: 0;
    background: var(--bg-secondary);
    font: 10px var(--font-mono);
  }
}
.dv-match-count {
  flex-shrink: 0;
  font: 9px var(--font-mono);
  color: var(--text-muted);
}
.dv-match-count.matched {
  color: var(--layer-color);
}
.dv-layer {
  .davinci-folio {
    border: 0;
    border-radius: 0;
    padding: 10px 0;
    font-size: 0.65625rem;
    line-height: 1.9;
  }
  .davinci-line {
    padding-inline-end: 8px;
  }
  .davinci-ln {
    width: 2.7em;
    padding-inline-end: 8px;
    font-size: 0.5625rem;
  }
  .davinci-mark {
    width: 4px;
    height: 4px;
    margin-inline-start: 4px;
  }
  .davinci-code {
    min-width: 0;
  }
}
.dv-wrap {
  .dv-layer {
    .davinci-line {
      min-width: 0;
    }
    .davinci-code {
      white-space: pre-wrap;
      overflow-wrap: anywhere;
    }
  }
}
.dv-layer {
  .davinci-line.linked {
    background: color-mix(in srgb, var(--layer-color) 10%, var(--code-bg));
    border-left-color: var(--layer-color);
  }
  .davinci-line.selected {
    background: color-mix(in srgb, var(--layer-color) 10%, var(--code-bg));
    border-left-color: var(--layer-color);
  }
  .davinci-line.linkable:hover {
    background: color-mix(in srgb, var(--layer-color) 7%, var(--code-bg));
  }
}
.dv-workbench {
  .page-surface {
    .davinci-line.linked {
      background: transparent;
      border-left-color: transparent;
    }
  }
}
.dv-surface-unit {
  display: inline;
  border: 0;
  border-radius: 2px;
  padding: 0;
  background: transparent;
  color: inherit;
  font: inherit;
  white-space: inherit;
  text-align: inherit;
  cursor: pointer;
}
.dv-surface-unit.linked {
  background: color-mix(in srgb, var(--dv-l1) 20%, var(--code-bg));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--dv-l1) 65%, transparent);
}
.dv-surface-unit:hover {
  background: color-mix(in srgb, var(--dv-l1) 20%, var(--code-bg));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--dv-l1) 65%, transparent);
}
.dv-surface-unit:focus-visible {
  background: color-mix(in srgb, var(--dv-l1) 20%, var(--code-bg));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--dv-l1) 65%, transparent);
}
.dv-layer {
  .davinci-output {
    gap: 0;
    overflow: hidden;
    .davinci-subtabs {
      gap: 2px;
      min-height: 38px;
      padding: 5px;
      border-bottom: 1px solid var(--border-primary);
    }
    .davinci-subtab {
      font-size: 0.5625rem;
      padding: 4px 5px;
    }
    .davinci-output-code.code-highlight {
      font-size: 0.65625rem;
      --code-line-height: 20px;
      border: 0;
      border-radius: 0;
    }
  }
}
.dv-layer-footer {
  display: flex;
  justify-content: space-between;
  gap: 5px;
  padding: 8px 9px;
  border-top: 1px solid var(--border-primary);
  font: 9px var(--font-mono);
  color: var(--text-muted);
  background: var(--bg-secondary);
}
.dv-layer-empty {
  display: flex;
  flex: 1;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 12px;
  text-align: center;
  padding: 20px;
  background-image: radial-gradient(var(--border-primary) 0.7px, transparent 0.7px);
  background-size: 12px 12px;
  color: var(--text-muted);
  font-size: 0.6875rem;
  p {
    margin: 0;
    line-height: 1.7;
  }
}
.dv-empty-glyph {
  font: 38px var(--font-mono);
  opacity: 0.4;
}
.dv-layer-empty {
  .davinci-ghost {
    font-size: 0.625rem;
  }
}
@media (min-width: 1700px) {
  .dv-layer {
    .davinci-folio {
      font-size: 0.75rem;
    }
    .davinci-output {
      .davinci-output-code.code-highlight {
        font-size: 0.75rem;
      }
    }
  }
  .dv-layer-header {
    padding: 16px;
  }
  .dv-layer-heading {
    h3 {
      font-size: 1rem;
    }
  }
  .dv-layer-header {
    p {
      font-size: 0.6875rem;
    }
  }
  .dv-layer-facts {
    font-size: 0.625rem;
  }
}
</style>
