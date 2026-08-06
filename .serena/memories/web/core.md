# Web (SvelteKit SPA)

Svelte 5 + SvelteKit 2 + TS. `adapter-static` с `fallback: 'index.html'` — **чистая SPA,
без SSR**: не использовать серверные `+page.server.ts` / `load` с серверными зависимостями.
Сборка в `web/build/`, отдаётся Rust-сервером (tower-http `fs`).

## Структура
```
src/routes/
  +layout.svelte, +layout.ts
  login/, register/, forgot-password/     # публичные
  (app)/                                  # группа за авторизацией (+layout.svelte)
    profile/
    projects/+page.svelte
    projects/[id]/+page.svelte
    projects/[id]/configurations/[configId]/parameters/+page.svelte
src/lib/
  api.ts            # весь HTTP-доступ к бэкенду — единая точка
  auth.ts           # состояние сессии / гварды
  avatar.ts, sort.ts, toast.ts, documentTitle.ts
  components/       # Avatar, Button, ConfirmDialog, Header, Input, Modal, ToastContainer
```

## Правила
- Сетевые вызовы — **только** через `lib/api.ts`; не звать `fetch` из компонентов.
- Уведомления — через `lib/toast.ts` + `ToastContainer`, не свои алерты.
- Подтверждения деструктивных действий — `ConfirmDialog`.
- Иерархия домена в URL отражает модель: project → configuration → parameters.
- Dev: `task web` (:5173) против сервера на :8080.
