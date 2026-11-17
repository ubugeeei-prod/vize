export function List({ items }: { items: string[] }) @{
	<ul>
		@for (const item of items; key item) {
			<li>{item}</li>
		} @empty {
			<li class="none">none</li>
		}
	</ul>
}

export function Cards({ rows }: { rows: { id: number; text: string }[] }) @{
	<>
		<h1>{rows.length}</h1>
		@for (const row: { id: number; text: string } of rows; key row.id) {
			<Card text={row.text} />
		} @empty {
			<Card text="nothing" />
		}
	</>
}

function Card({ text }: { text: string }) @{
	<p class="card">{text}</p>
}
