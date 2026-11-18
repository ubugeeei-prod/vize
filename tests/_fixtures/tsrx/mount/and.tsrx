const values: Record<string, unknown> = {
	zero: 0,
	empty: '',
	nan: NaN,
	none: null,
	missing: undefined,
	no: false,
	big: 0n,
	text: 'x',
	yes: true,
};

export function Count({ n, label }: { n: number; label: string }) @{
	<p>{n && <b class="count">{label}</b>}</p>
}

export function Values({ kind }: { kind: string }) @{
	<div>
		<span>before</span>
		{values[kind] && <em>set</em>}
		<span>after</span>
	</div>
}

export function Root({ n }: { n: number }) @{
	<>{n && <i>{n}</i>}</>
}

export function Pair({ a, b }: { a: number; b: string }) @{
	<ul>
		{a && <li class="a">{a}</li>}
		{b && <li class="b">{b}</li>}
	</ul>
}

function Badge({ text }: { text: string }) @{
	<mark>{text}</mark>
}

export function Tagged({ count, title }: { count: number; title: string }) @{
	<section>{count && <Badge text={title} />}</section>
}

export function Chained({ n, m }: { n: number; m: string }) @{
	<p>{n && m && <b>{m}</b>}</p>
}

export function Drawn({ n }: { n: number }) @{
	<svg>{n && <circle r={n} />}</svg>
}
