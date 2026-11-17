export function Shadow({ label, items }: { label: string; items: string[] }) @{
	const upper = label.toUpperCase();
	const pick = (label: string) => label + upper;
	function twice(label: string) {
		return label + label;
	}
	let total = 0;
	for (const label of items) {
		total += label.length;
	}
	{
		const label = 'block';
		total += label.length;
	}

	<>
		<h1 title={label}>{pick('x')} {twice('y')} {total}</h1>
		<p>{items.map((label) => label.toUpperCase()).join(',')}</p>
		@for (const label of items; key label) {
			<li>{label}</li>
		}
	</>
}
