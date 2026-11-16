export function Rows({ rows }: { rows: { id: number; text: string }[] }) @{
	@for (const row of rows; index i; key row.id) {
		<li class="row">{i}: {row.text}</li>
	}
}

export function Pairs({ rows }: { rows: { id: number; text: string }[] }) @{
	@for (const { id, text } of rows; key id) {
		<Pair id={id} text={text} />
	}
}

function Pair({ id, text }: { id: number; text: string }) @{
	<p data-id={id}>{text}</p>
}

export function Nested({ rows }: { rows: { id: number; text: string }[] }) @{
	<ul>
		@for (const row of rows; key row.id) {
			<li>{row.text}</li>
		}
	</ul>
}
