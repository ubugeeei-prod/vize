import { ref } from 'vue';

export function Items({ items }: { items: string[] }) @{
	<>
		<h1>{items.length}</h1>
		{items.map((item, i) => <li class={i % 2 ? 'odd' : 'even'}>{i}: {item}</li>)}
	</>
}

export function Rows({ rows }: { rows: { id: number; text: string }[] }) @{
	const picked = ref(0);

	<>
		<button onClick={() => (picked.value += 1)}>next</button>
		{rows.map(function (row) {
			return <Row key={row.id} text={row.text} on={row.id === picked.value} />;
		})}
	</>
}

function Row({ text, on }: { text: string; on: boolean }) @{
	<p class={on ? 'on' : 'off'}>{text}</p>
}
