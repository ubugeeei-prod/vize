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

export function Values({ items }: { items: string[] }) @{
	@for (const item of items) {
		<li class="value">{item}</li>
	}
}

export function Indexed({ items }: { items: string[] }) @{
	@for (const item of items; index i) {
		<li>{i}: {item}</li>
	}
}

export function Owned({ rows }: { rows: { id: number; text: string }[] }) @{
	@for (const row of rows) {
		<Pair key={row.id} id={row.id} text={row.text} />
	}
}

export function Terms({ rows }: { rows: { id: number; text: string }[] }) @{
	@for (const row of rows; key row.id) {
		<>
			<dt>{row.id}</dt>
			<dd class="term">{row.text}</dd>
		</>
	}
}

export function Labels({ items }: { items: string[] }) @{
	@for (const item of items; index i) {
		<>
			{i}
			<b>{item}</b>
		</>
	}
}
