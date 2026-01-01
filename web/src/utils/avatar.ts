/**
 * Извлекает инициалы из email адреса
 * @param email - Email адрес пользователя
 * @returns Строка с инициалами (максимум 2 символа)
 */
export function getInitialsFromEmail(email: string): string {
  if (!email || !email.includes('@')) {
    return "U";
  }
  
  // Разделяем email на локальную часть и домен
  const localPart = email.split('@')[0];
  
  if (!localPart) {
    return "U";
  }
  
  // Убираем все не-буквенные символы и берём первые две буквы
  const letters = localPart.replace(/[^a-zA-Zа-яА-Я]/g, '');
  
  if (letters.length === 0) {
    // Если нет букв, берём первые два символа из локальной части (но не @)
    const cleanPart = localPart.substring(0, 2);
    return cleanPart ? cleanPart.toUpperCase() : "U";
  }
  
  // Берём первые две буквы
  return letters.substring(0, 2).toUpperCase();
}

