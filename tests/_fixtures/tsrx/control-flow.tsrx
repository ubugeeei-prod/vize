export function Status({ count, items, mode }: { count: number; items: { id: number; text: string }[]; mode: string }) @{
	<section>
		@if (count > 2) {
			<p>many</p>
		} @else if (count > 0) {
			<p>some</p>
		} @else {
			<p>none</p>
		}
		<ul>
			@for (const item of items; index i; key item.id) {
				<li>{i}: {item.text}</li>
			} @empty {
				<li>empty</li>
			}
		</ul>
		<ol>
			@for (const item of items) {
				<li>{item.text}</li>
			}
		</ol>
		@switch (mode) {
			@case 'a': {
				<b>a</b>
			}
			@case 'b': {
				<i>b</i>
			}
			@default: {
				<u>other</u>
			}
		}
	</section>
}
