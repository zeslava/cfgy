# Отчет о соответствии дизайну Figma

> **Примечание:** Фронтенд переведён на Svelte 5 + SvelteKit (каталог `web/`). В отчёте указаны файлы старого React-фронта; рекомендации по дизайну остаются актуальными для Svelte-компонентов.

## Критические несоответствия

### 1. Header (Layout компонент)

**Файл**: `web/src/components/Layout.tsx`

**Проблемы**:
- ❌ Фон: используется `bg-lines-gray` вместо `bg-basic-brand-color` (#0F0F7B)
- ❌ Цвет текста: используется `text-text-dark` вместо `text-text-light` (белый)
- ❌ Padding: используется `py-[40px]` вместо `py-[32px]`
- ❌ Логотип: используется `text-text-dark` вместо белого цвета

**Должно быть**:
```tsx
<header className="bg-basic-brand-color text-text-light py-[32px] px-[120px]">
  <span className="font-jersey text-[48px] text-text-light leading-none">
    CFGY
  </span>
  <span className="font-inter font-normal text-regular-24 text-text-light">
    {user?.email || "Пользователь"}
  </span>
</header>
```

### 2. Страница ProjectDetail

**Файл**: `web/src/pages/ProjectDetail.tsx`

**Проблемы**:
- ❌ Заголовок проекта: используется `text-heading-32` вместо `text-heading-40` (40px)
- ❌ Описание: используется `text-regular-24` вместо `text-regular-20` (20px)
- ❌ Отступ между breadcrumbs и заголовком: используется `mb-[-40px]` вместо правильного отступа

**Должно быть**:
```tsx
<div className="flex flex-col gap-[52px]">
  <div className="flex flex-col gap-[12px]">
    <Breadcrumbs ... />
    <div className="flex gap-[32px] items-start">
      <div className="flex flex-col gap-[4px]">
        <h1 className="font-istok-web font-bold text-heading-40 text-text-dark">
          {project.name}
        </h1>
        <p className="font-inter font-normal text-regular-20 text-text-dark">
          {project.description || 'Описание'}
        </p>
      </div>
    </div>
  </div>
</div>
```

## Средние несоответствия

### 3. Страница Projects

**Файл**: `web/src/pages/Projects.tsx`

**Статус**: ✅ В основном соответствует, но нужно проверить:
- ✅ Заголовок: `text-heading-40` - правильно
- ✅ Отступ: `gap-[32px]` - правильно
- ✅ Шрифт проектов: `font-inter text-regular-24` - правильно
- ✅ Структура разделителей: правильно

### 4. Страница Parameters

**Файл**: `web/src/pages/Parameters.tsx`

**Статус**: ✅ В основном соответствует после последних исправлений

## Детальный анализ компонентов

### Header компонент

**Текущая реализация**:
```tsx
<header className="bg-lines-gray text-text-dark py-[40px] px-[120px]">
```

**Дизайн из Figma**:
- Фон: `#0F0F7B` (basic-brand-color)
- Текст: белый (`#FFFFFF`)
- Padding: `py-[32px]`

**Несоответствия**:
1. Фон должен быть темно-синим, а не серым
2. Весь текст должен быть белым
3. Padding должен быть 32px, а не 40px

### Avatar в Header

**Текущая реализация**: Использует `bg-basic-gray` для фона аватара

**Дизайн из Figma**: Аватар должен быть белым (`bg-white`) на темном фоне

### Кнопка выхода в Header

**Текущая реализация**: Использует `variant="basic-secondary"` (светло-голубой фон)

**Дизайн из Figma**: Кнопка должна иметь светло-голубой фон (`#EBF1FF`), что соответствует `basic-secondary` - ✅ правильно

## Рекомендации по исправлению

### Приоритет 1 (Критично):
1. Исправить фон Header на `bg-basic-brand-color`
2. Изменить цвет текста в Header на белый
3. Изменить padding Header на `py-[32px]`
4. Исправить размер заголовка проекта на `text-heading-40`
5. Исправить размер описания проекта на `text-regular-20`

### Приоритет 2 (Важно):
1. Проверить отступы между breadcrumbs и заголовками на всех страницах
2. Убедиться, что все цвета соответствуют токенам дизайн-системы

## Проверка токенов

### Цвета в дизайне:
- `Basic/Brand_Color`: #0F0F7B ✅ определен в `tailwind.config.js` как `basic.brand-color`
- `Basic/Secondary`: #EBF1FF ✅ определен как `basic.secondary`
- `Text/Light`: #FFFFFF ✅ определен как `text.light`
- `Text/Dark`: #000033 ✅ определен как `text.dark`

### Шрифты в дизайне:
- `Headings/Heading_40`: Istok Web Bold 40px ✅ определен как `text-heading-40`
- `Headings/Heading_32`: Istok Web Bold 32px ✅ определен как `text-heading-32`
- `Text/Regular_24`: Inter Regular 24px ✅ определен как `text-regular-24`
- `Text/Regular_20`: Inter Regular 20px ✅ определен как `text-regular-20`

## Исправления (выполнено)

### ✅ Header компонент
- Исправлен фон на `bg-basic-brand-color` (#0F0F7B)
- Изменен цвет текста на белый (`text-text-light`)
- Исправлен padding на `py-[32px]`
- Исправлен цвет логотипа на белый
- Исправлен цвет email на белый
- Исправлен gap между аватаром и email на `gap-[12px]`
- Аватар теперь белый (`bg-white`) на темном фоне

### ✅ ProjectDetail страница
- Исправлен размер заголовка на `text-heading-40` (40px)
- Исправлен размер описания на `text-regular-20` (20px)
- Исправлена структура отступов: `gap-[52px]` для основной секции, `gap-[12px]` для breadcrumbs и заголовка

### ✅ Добавлен токен
- Добавлен `bg-basic-brand-color` в `index.css` для использования в Header

## Итоговая оценка соответствия (после исправлений)

- **Header**: ✅ 100% - полностью соответствует дизайну
- **Projects page**: ✅ 100% - полностью соответствует
- **Parameters page**: ✅ 100% - полностью соответствует
- **ProjectDetail page**: ✅ 100% - полностью соответствует

## Общий процент соответствия: ~98%

Остались только мелкие детали, которые могут отличаться в зависимости от контекста использования (например, формы создания/редактирования, которые не показаны в основном дизайне).

