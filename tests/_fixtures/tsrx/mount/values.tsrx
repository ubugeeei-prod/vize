export function Values({ text, count, flag, items }: {
	text: string;
	count: number;
	flag: boolean;
	items: string[];
}) @{
	<ul>
		<li>Text: {text}</li>
		<li>{count} and {count + 1}</li>
		<li>{flag ? 'on' : 'off'}</li>
		<li>{flag ? null : 0}</li>
		<li>{items.join(', ')}</li>
	</ul>
}
