# Конвенции

## Язык
Комментарии, doc-комментарии (`///`) и **тексты ошибок** — на русском.
Идентификаторы — английские. Описания задач в Taskfile — русские.

## Rust
- Именование: структуры/трейты PascalCase (трейт с ролевым суффиксом: `UserRepository`,
  `CryptoService`), функции snake_case, константы SCREAMING_SNAKE_CASE, модули snake_case.
- Ошибки: `anyhow::Result<T>` + `.context("<русский текст>")`. Никакого `unwrap()` вне тестов
  без комментария. Валидация входных данных — в `application/`, не в handlers.
- `async_trait` для async-методов трейтов.
- `Uuid` для id, `chrono::DateTime<Utc>` для времени.
- Doc-комментарии объясняют «почему», не «что».

## Добавление серверной фичи (порядок)
1. трейт в `ports/repositories.rs` (или `ports/crypto.rs`)
2. сущность в `ports/entities.rs` при необходимости
3. бизнес-логика в `application/`
4. реализация в `adapters/persistence/postgres.rs`
5. handler в `adapters/api/handlers.rs` + маршрут в `adapters/api/router.rs`
6. пара миграций при изменении схемы

## Миграции
- Имя: `NNN_description.up.sql` + `NNN_description.down.sql`.
- В up-миграциях всегда `IF NOT EXISTS`.
- down-файл обязателен, даже тривиальный.

## Auth (три механизма)
- Веб: JWT-сессия в cookie `cfgy_session`.
- CLI/CI: Bearer API-токены.
- SSO: OIDC Authorization Code (`OIDC_*`, `ZID_*` — fallback), state/next-cookie с TTL 15 минут.
  Пользователь привязывается только по (provider, sub).
Реализация — `adapters/api/auth.rs` + `application/auth.rs`.

## Web
Svelte 5 runes, функциональный стиль; логика — в сторах/модулях `web/src/lib/`,
не в компонентах. Без inline-стилей кроме единичных случаев. См. `mem:web/core`.

## Git
Коммиты: `(feat|fix|refactor|chore): краткое сообщение` (русский), без co-author-строк
и без буллет-списков в теле.
