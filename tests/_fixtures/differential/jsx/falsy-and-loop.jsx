const App = () => <main>{rows.map(row => <div key={row.id} data-id={row.id}>{row.value && <span>X</span>}</div>)}</main>;
