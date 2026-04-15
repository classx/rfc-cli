# rfc-cli

CLI-инструмент для управления RFC (Request for Comments) документами в проекте.

RFC — это формальные спецификации, которые описывают проектные решения **до** начала реализации. Это позволяет:

- фиксировать архитектурные решения и их обоснование
- обсуждать дизайн до написания кода
- давать AI-ассистентам точный контекст вместо расплывчатых запросов
- сохранять историю решений и отвергнутых альтернатив

## Установка

```sh
cargo build --release
```

Бинарный файл будет в `target/release/rfc-cli`.

Для удобства можно скопировать в PATH:

```sh
cp target/release/rfc-cli ~/.local/bin/
```

## Быстрый старт

```sh
# Инициализация структуры RFC в проекте
rfc-cli init

# Создание нового RFC
rfc-cli new "кэширование запросов"

# Просмотр списка
rfc-cli list

# Работа над RFC
rfc-cli edit 1
rfc-cli set 1 review

# Проверка и диагностика
rfc-cli check
rfc-cli doctor
```

## Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `RFC_HOME` | Корневой каталог проекта | Текущая директория |
| `EDITOR` | Редактор для команды `edit` | — (обязателен для `edit`) |

## Команды

### `init` — инициализация

Создаёт каталог `docs/rfcs/` и пустой индексный файл `.index.json`. Идемпотентна — повторный вызов безопасен.

```sh
rfc-cli init
```

### `new <title>` — создание RFC

Создаёт новый RFC из шаблона с автоматической нумерацией (0001, 0002, ...).

```sh
rfc-cli new "кэширование запросов"
# → Created docs/rfcs/0007.md
```

Новый RFC создаётся в статусе `draft` с шаблоном, включающим обязательные секции: Проблема, Задача, Дизайн, Альтернативы, Голосование, Миграция.

### `list` — список RFC

Выводит таблицу всех RFC, отсортированных по номеру.

```sh
rfc-cli list
#  #      Status      Title
#  0001   implemented RFC-0001: структура RFC cli
#  0002   implemented RFC-0002: реализация команд init и new
#  ...

# Фильтр по статусу
rfc-cli list --status draft
```

| Флаг | Описание |
|------|----------|
| `--status <status>` | Показать только RFC с указанным статусом |

### `view <number>` — просмотр

Выводит полное содержимое RFC в терминал. Номер можно указывать без ведущих нулей.

```sh
rfc-cli view 1
# эквивалентно: rfc-cli view 0001
```

### `status <number>` — текущий статус

Выводит статус RFC из индекса (быстро, без чтения файла).

```sh
rfc-cli status 3
# RFC-0003: implemented
```

### `edit <number>` — редактирование

Открывает RFC в `$EDITOR`. Блокирует редактирование `accepted` и `implemented` RFC без `--force`.

```sh
rfc-cli edit 1

# Принудительное редактирование принятого RFC (с предупреждением)
rfc-cli edit 1 --force
```

| Флаг | Описание |
|------|----------|
| `--force` | Разрешить редактирование accepted/implemented RFC |

### `set <number> <status>` — изменение статуса

Меняет статус RFC с валидацией допустимых переходов. Автоматически обновляет frontmatter, индекс и `content_hash`.

```sh
rfc-cli set 1 review
rfc-cli set 1 accepted
rfc-cli set 1 implemented

# Замена другим RFC
rfc-cli set 3 superseded --by 7
```

| Флаг | Описание |
|------|----------|
| `--by <number>` | Номер замещающего RFC (обязателен для `superseded`) |

Допустимые переходы:

```
draft → review → accepted → implemented
                    ↓            ↓
               deprecated    superseded
```

Также допускаются: `review → draft`, `draft → deprecated`, `accepted → deprecated`, `implemented → deprecated`.

### `link <number> <path>` — привязка к коду

Связывает RFC с файлом исходного кода. Путь сохраняется относительно корня проекта.

```sh
rfc-cli link 2 src/commands/init.rs
# RFC-0002: linked src/commands/init.rs ✅

# Повторная привязка не дублирует
rfc-cli link 2 src/commands/init.rs
# RFC-0002: link already exists: src/commands/init.rs
```

| Флаг | Описание |
|------|----------|
| `--force` | Разрешить изменение accepted/implemented RFC (пересчитает `content_hash`) |

### `unlink <number> <path>` — удаление привязки

Удаляет связь RFC с файлом.

```sh
rfc-cli unlink 2 src/commands/init.rs
# RFC-0002: unlinked src/commands/init.rs ✅
```

| Флаг | Описание |
|------|----------|
| `--force` | Разрешить изменение accepted/implemented RFC |

### `deps <number>` — дерево зависимостей

Показывает прямые или обратные зависимости RFC.

```sh
# Прямые зависимости: от кого зависит RFC-0005
rfc-cli deps 5
# RFC-0005 depends on:
#   - RFC-0001 (структура RFC cli) [implemented]
#   - RFC-0003 (реализация команд list, view, status и edit) [implemented]

# Обратные зависимости: кто зависит от RFC-0001
rfc-cli deps 1 --reverse
# RFC-0001 is depended on by:
#   - RFC-0002 (реализация команд init и new) [implemented]
#   - RFC-0003 (реализация команд list, view, status и edit) [implemented]
#   - ...
```

| Флаг | Описание |
|------|----------|
| `--reverse` | Показать обратные зависимости (кто зависит от данного RFC) |

### `check [<number>]` — валидация формата

Проверяет корректность RFC: frontmatter, обязательные секции, ссылки, зависимости, целостность `content_hash`. Без аргумента проверяет все RFC.

```sh
rfc-cli check        # все RFC
rfc-cli check 3      # только RFC-0003
```

Проверки:
- Корректность YAML frontmatter
- Непустые обязательные поля (`title`, `status`)
- Валидный статус
- Наличие обязательных секций (`## Problem`, `## Goal`, `## Design`, `## Alternatives`)
- Соответствие номера в имени файла и заголовке
- Существование зависимостей
- Существование файлов из `links`
- Целостность `content_hash` для accepted/implemented RFC

Код выхода: `0` — всё ОК, `1` — есть ошибки.

### `doctor` — диагностика здоровья проекта

Анализирует смысловую согласованность между RFC и кодовой базой. В отличие от `check` (формат), `doctor` ищет логические проблемы.

```sh
rfc-cli doctor
rfc-cli doctor --stale-days 14
```

| Флаг | Описание |
|------|----------|
| `--stale-days <N>` | Порог дней для «зависшего черновика» (по умолчанию 30) |

Диагностические проверки:

| Проверка | Уровень | Описание |
|----------|---------|----------|
| Дрифт кода | ❌ ошибка | Файл из `links` изменился после принятия RFC |
| Нет реализации | ⚠️ предупреждение | `accepted` RFC без ссылок в `links` |
| Мёртвые ссылки | ❌ ошибка | Файл из `links` не существует на диске |
| Зависший черновик | ⚠️ предупреждение | `draft` RFC не обновлялся дольше N дней |
| Незакрытые зависимости | ⚠️ предупреждение | `accepted` RFC зависит от не-accepted/implemented |
| Циклические зависимости | ❌ ошибка | Цикл в графе зависимостей |

Код выхода: `1` при наличии ❌ ошибок, `0` при только ⚠️ предупреждениях или чистом проекте.

Пример вывода:

```
RFC-0003 (cache):
  ❌ code drift: src/cache/mod.rs modified after RFC acceptance
  ❌ dead link: src/cache/old_store.rs (file not found)

RFC-0005 (logging):
  ⚠️  no linked files (status: accepted)

Summary: 3 error(s), 1 warning(s) across 2 RFC(s).
```

### `reindex` — пересборка индекса

Полностью перестраивает `.index.json` из RFC-файлов на диске. Используйте, если индекс повреждён или рассинхронизирован.

```sh
rfc-cli reindex
# Reindexed 6 RFCs.
```

## Краткая справка

| Команда | Описание |
|---------|----------|
| `init` | Создаёт `docs/rfcs/` и индексный файл |
| `new <title>` | Создаёт RFC из шаблона |
| `list [--status S]` | Таблица RFC |
| `view <N>` | Содержимое RFC |
| `status <N>` | Статус RFC |
| `edit <N> [--force]` | Открыть в `$EDITOR` |
| `set <N> <S> [--by N]` | Изменить статус |
| `link <N> <path> [--force]` | Привязать файл |
| `unlink <N> <path> [--force]` | Убрать привязку |
| `deps <N> [--reverse]` | Дерево зависимостей |
| `check [N]` | Валидация формата |
| `doctor [--stale-days N]` | Диагностика здоровья |
| `reindex` | Пересборка индекса |

## RFC-процесс

### Правила

1. **Нельзя писать код без RFC** — любое нетривиальное изменение начинается с RFC
2. **RFC можно нарушить только через новый RFC** — прямое редактирование принятых RFC запрещено
3. **Статусы обязательны** — каждый RFC проходит через определённый жизненный цикл

### Жизненный цикл

```
draft → review → accepted → implemented
                    ↓            ↓
               deprecated    superseded
```

| Статус | Значение |
|--------|----------|
| `draft` | Черновик, идёт написание |
| `review` | Документ готов к обсуждению |
| `accepted` | Решение утверждено, можно реализовывать |
| `implemented` | Реализация завершена |
| `superseded` | Заменён новым RFC |
| `deprecated` | Устарел или отменён без замены |

### Формат RFC-документа

Каждый RFC — это Markdown-файл в `docs/rfcs/` с YAML frontmatter:

```yaml
---
title: "RFC-0001: название"
status: draft
dependencies: [RFC-0003, RFC-0005]
superseded_by: null
links:
  - src/commands/init.rs
  - src/rfclib/rfc.rs
---

## Problem
## Goal
## Design
## Alternatives
## Voting
## Migration
```

| Поле | Тип | Описание |
|------|-----|----------|
| `title` | string | Название в формате `RFC-NNNN: описание` |
| `status` | string | Текущий статус |
| `dependencies` | list | RFC-зависимости, например `[RFC-0001]` |
| `superseded_by` | string/null | Номер замещающего RFC |
| `links` | list | Пути к связанным файлам исходного кода |

### Индексный файл

Метаданные всех RFC кэшируются в `docs/rfcs/.index.json`. Индекс обновляется автоматически при каждом вызове CLI (по `mtime`). Если индекс повреждён — восстановите через `rfc-cli reindex`.

Для `accepted` и `implemented` RFC в индексе хранится `content_hash` (SHA-256) — защита от несанкционированного изменения принятых документов.

## Разработка

```sh
# Сборка
make build          # debug
make release        # release

# Тесты
make test           # запуск всех тестов (95 шт.)
cargo test <name>   # запуск конкретного теста

# Проверка проекта
cargo run -- check
cargo run -- doctor
```

### Структура проекта

```
src/
├── main.rs              # точка входа, маршрутизация команд
├── cli.rs               # определение CLI (clap derive)
├── commands/
│   ├── mod.rs
│   ├── init.rs          # rfc-cli init
│   ├── new.rs           # rfc-cli new
│   ├── list.rs          # rfc-cli list
│   ├── view.rs          # rfc-cli view
│   ├── status.rs        # rfc-cli status
│   ├── edit.rs          # rfc-cli edit
│   ├── set.rs           # rfc-cli set
│   ├── link.rs          # rfc-cli link
│   ├── unlink.rs        # rfc-cli unlink
│   ├── deps.rs          # rfc-cli deps
│   ├── check.rs         # rfc-cli check
│   ├── doctor.rs        # rfc-cli doctor
│   └── reindex.rs       # rfc-cli reindex
└── rfclib/
    ├── mod.rs
    ├── rfc.rs           # парсинг frontmatter, нормализация, обновление полей
    ├── index.rs         # загрузка/сохранение индекса, хэширование, пересборка
    └── project.rs       # определение корня проекта (RFC_HOME)

tests/
└── integration_test.rs  # 95 интеграционных тестов

docs/rfcs/
├── .index.json          # индексный файл (генерируемый)
├── 0001.md              # RFC-0001: структура RFC cli
├── 0002.md              # RFC-0002: реализация команд init и new
├── 0003.md              # RFC-0003: реализация команд list, view, status и edit
├── 0004.md              # RFC-0004: реализация команд set, check и reindex
├── 0005.md              # RFC-0005: реализация команд link, unlink и deps
└── 0006.md              # RFC-0006: реализация команды doctor
```

## Лицензия

См. файл [LICENSE](LICENSE).