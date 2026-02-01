// Утилиты для отображения имени и инициалов пользователя

const UUID_REGEX =
	/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/**
 * Возвращает отображаемое имя пользователя.
 * Если username — UUID (например sub из ZID), показываем email.
 */
export function getDisplayName(user: {
	username?: string | null;
	email?: string;
} | null): string {
	if (!user) return 'Пользователь';
	if (user.username && !UUID_REGEX.test(user.username)) return user.username;
	return user.email || 'Пользователь';
}

/**
 * Извлекает инициалы из email адреса
 */
export function getInitialsFromEmail(email: string): string {
	if (!email || !email.includes('@')) {
		return 'U';
	}
	const localPart = email.split('@')[0];
	if (!localPart) {
		return 'U';
	}
	const letters = localPart.replace(/[^a-zA-Zа-яА-Я]/g, '');
	if (letters.length === 0) {
		const cleanPart = localPart.substring(0, 2);
		return cleanPart ? cleanPart.toUpperCase() : 'U';
	}
	return letters.substring(0, 2).toUpperCase();
}
