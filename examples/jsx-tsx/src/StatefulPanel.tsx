import { computed, ref } from "vue";

type Row = {
  id: string;
  label: string;
  status: "ready" | "blocked";
};

type PanelProps = {
  rows: Row[];
  initialActiveId?: string;
  accentColor?: string;
  dense?: boolean;
};

type PanelEmits = {
  select: [id: string];
};

type PanelSlots = {
  footer?: () => unknown;
};

const StatefulPanel = (
  { rows, initialActiveId = rows[0]?.id, accentColor = "#42b883", dense = false }: PanelProps,
  { emit, slots }: Ctx<PanelEmits, PanelSlots>,
) => {
  const activeId = ref(initialActiveId);
  const activeRow = computed(() => rows.find((row) => row.id === activeId.value));

  const selectRow = (id: string) => {
    activeId.value = id;
    emit("select", id);
  };

  return (
    <section
      class={{ panel: true, dense }}
      style={{
        "--panel-accent": accentColor,
      }}
    >
      <header class="panel-header">
        <h2>{activeRow.value?.label ?? "Select a row"}</h2>
      </header>

      <ul class="panel-list">
        {rows.map((row, index) => (
          <li
            key={row.id}
            class={{
              active: row.id === activeId.value,
              blocked: row.status === "blocked",
            }}
            data-index={index}
          >
            <button type="button" onClick={() => selectRow(row.id)}>
              <span>{row.label}</span>
              {row.id === activeId.value ? <strong>Active</strong> : <em>{index + 1}</em>}
            </button>
          </li>
        ))}
      </ul>

      <footer>{slots.footer?.()}</footer>

      <style scoped>{`
        .panel {
          border: 1px solid var(--panel-accent);
          padding: 12px;
        }

        .panel-list {
          display: grid;
          gap: 8px;
          margin: 0;
          padding: 0;
        }

        .active {
          color: var(--panel-accent);
        }
      `}</style>
    </section>
  );
};

export default StatefulPanel;
