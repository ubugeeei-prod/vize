function Box<T>({ value }: { value: T }) @{
	<p>{String(value)}</p>
}

export function App() @{
	<Box<number> value={1}>text</Box<number>>
}
