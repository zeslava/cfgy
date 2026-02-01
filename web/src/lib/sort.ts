/**
 * Сортирует массив по строковому полю с учётом русской локали.
 */
export function sortByString<T>(items: T[], getValue: (item: T) => string): T[] {
	return [...items].sort((a, b) =>
		getValue(a).localeCompare(getValue(b), 'ru', { sensitivity: 'base' })
	);
}
