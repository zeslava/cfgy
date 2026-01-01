/**
 * Утилиты для сортировки списков
 */

/**
 * Сортирует массив объектов по строковому полю с учетом русской локали
 * @param items - массив элементов для сортировки
 * @param getValue - функция получения значения для сортировки
 * @returns отсортированный массив
 */
export function sortByString<T>(
  items: T[],
  getValue: (item: T) => string
): T[] {
  return [...items].sort((a, b) =>
    getValue(a).localeCompare(getValue(b), "ru", { sensitivity: "base" })
  );
}

/**
 * Сортирует массив по строковому значению (для простых строк)
 * @param items - массив строк
 * @returns отсортированный массив
 */
export function sortStrings(items: string[]): string[] {
  return [...items].sort((a, b) =>
    a.localeCompare(b, "ru", { sensitivity: "base" })
  );
}
