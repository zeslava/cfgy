const TITLE_SUFFIX = ' — CFGY';

/**
 * Устанавливает document.title для страницы.
 */
export function setDocumentTitle(title: string): void {
	if (typeof document === 'undefined') return;
	document.title = title ? `${title}${TITLE_SUFFIX}` : 'CFGY';
}
