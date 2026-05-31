<<<<<<< HEAD
# rs_blog
=======
# rs_blog

Учебный Rust-проект для блога с архитектурой из нескольких крейтов, включая сервер, клиентскую библиотеку, CLI и WASM-интерфейс.

## Описание

Проект реализует блоговый бэкенд с авторизацией, хранением данных в PostgreSQL и поддержкой HTTP + gRPC.
Фронтенд-часть представлена в виде WASM-приложения, а CLI-клиент позволяет управлять пользователями и постами из терминала.

## Архитектура

Корневой пакет — `Cargo.toml` с рабочей областью (`workspace`), в которой находятся четыре крейта:

- `blog-server`
  - основной сервер.
  - предоставляет HTTP API на `http://localhost:3000/api`.
  - предоставляет gRPC API на `0.0.0.0:50051`.
  - работает с PostgreSQL через `sqlx`.
  - реализует регистрацию, вход, CRUD для постов и JWT-аутентификацию.

- `blog-client`
  - библиотека клиента.
  - содержит HTTP и gRPC транспорт.
  - используется как основа для CLI.

- `blog-cli`
  - терминальный клиент.
  - оборачивает `blog-client` и предоставляет команды: `register`, `login`, `create`, `get`, `update`, `delete`, `list`.
  - хранит JWT-токен в файле `.blog_token`.

- `blog-wasm`
  - WebAssembly-приложение.
  - реализует простой браузерный UI для регистрации, входа и работы с постами.
  - по умолчанию взаимодействует с сервером `http://localhost:3000/api`.

## Установка зависимостей и окружения

### Требования

- Rust `stable` (версия 2024 или новее)
- Cargo
- Docker и Docker Compose
- PostgreSQL не обязателен локально, можно запустить контейнером
- Опционально для WASM: `wasm-pack`

### Запуск PostgreSQL через Docker Compose

В корне проекта есть `docker-compose.yaml`.

```bash
cd /home/nepich/Projs/Learning/Rust/rs_blog
docker compose up -d
```

Docker Compose создаст сервис:

- `db` — PostgreSQL
  - пользователь: `myuser`
  - пароль: `mypassword`
  - база: `blogdb`

### Переменные окружения

Сервер требует две переменные:

- `DATABASE_URL`
- `JWT_SECRET`

Пример `.env` в корне проекта:

```dotenv
DATABASE_URL=postgres://myuser:mypassword@localhost:5432/blogdb
JWT_SECRET=replace-with-secure-random-value
```

### Генерация JWT-ключа

Рекомендуется использовать криптографически стойкий секрет.

```bash
# OpenSSL
export JWT_SECRET=$(openssl rand -hex 32)

# Python
export JWT_SECRET=$(python3 -c 'import secrets; print(secrets.token_urlsafe(32))')
```

### Пример `.env`

```dotenv
DATABASE_URL=postgres://myuser:mypassword@localhost:5432/blogdb
JWT_SECRET=YOUR_GENERATED_JWT_SECRET
```

## Сборка компонентов

### Сборка всего проекта

```bash
cargo build --workspace
```

### Сборка сервера

```bash
cargo build -p blog-server
```

### Сборка CLI

```bash
cargo build -p blog-cli
```

### Сборка WASM

```bash
cargo build --target wasm32-unknown-unknown -p blog-wasm
```

Для удобной сборки WASM рекомендуется установить `wasm-pack`:

```bash
cargo install wasm-pack
cd blog-wasm
wasm-pack build --target web
```

## Запуск компонентов

### Запуск сервера

```bash
cd /home/nepich/Projs/Learning/Rust/rs_blog
cargo run -p blog-server
```

Сервер стартует на:

- HTTP: `http://localhost:3000/api`
- gRPC: `http://localhost:50051`

### Запуск CLI

CLI-команды выполняются через Cargo:

```bash
cd /home/nepich/Projs/Learning/Rust/rs_blog
cargo run -p blog-cli -- register --username alice --email alice@example.com --password secret123
```

Для gRPC транспорта добавьте флаг `--grpc`:

```bash
cargo run -p blog-cli -- --grpc login --username alice --password secret123
```

Если сервер слушает нестандартный адрес, используйте `--server`:

```bash
cargo run -p blog-cli -- --server http://localhost:3000/api login --username alice --password secret123
```

### Запуск WASM UI

1. Соберите WASM-пакет: `wasm-pack build --target web`
2. Запустите статический сервер в папке `blog-wasm`:

```bash
cd /home/nepich/Projs/Learning/Rust/rs_blog/blog-wasm
python3 -m http.server 8000
```

3. Откройте браузер:

```text
http://localhost:8000/index.html
```

### Запуск gRPC UI (docker-compose)

```bash
docker compose up -d grpcui
```

Обычно доступно на `http://localhost:8080` и позволяет исследовать gRPC API.

> Если `host.docker.internal` в контейнере не резолвится, используйте адрес хоста напрямую или настройте `grpcui` команду на `http://127.0.0.1:50051`.

## Примеры сценариев

### 1) Регистрация пользователя через curl

```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","password":"secret123"}'
```

Ожидаемый ответ: JSON с `token` и информацией о пользователе.

### 2) Вход через curl

```bash
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","password":"secret123"}'
```

### 3) Создание поста через curl

```bash
curl -X POST http://localhost:3000/api/posts \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -d '{"title":"Первый пост","content":"Это тестовый пост"}'
```

### 4) Список постов через curl

```bash
curl "http://localhost:3000/api/posts?limit=10&offset=0"
```

### 5) CRUD в CLI

```bash
cd /home/nepich/Projs/Learning/Rust/rs_blog
cargo run -p blog-cli -- register --username alice --email alice@example.com --password secret123
cargo run -p blog-cli -- login --username alice --password secret123
cargo run -p blog-cli -- create --title "Мой пост" --content "Контент поста"
cargo run -p blog-cli -- list --limit 5 --offset 0
cargo run -p blog-cli -- get --id 1
cargo run -p blog-cli -- update --id 1 --title "Обновлённый заголовок"
cargo run -p blog-cli -- delete --id 1
```

### 6) Работа в браузере через WASM

1. Запустите сервер: `cargo run -p blog-server`
2. Соберите WASM: `cd blog-wasm && wasm-pack build --target web`
3. Запустите статический сервер: `python3 -m http.server 8000`
4. Откройте `http://localhost:8000/index.html`
5. Введите URL сервера: `http://localhost:3000/api`
6. Зарегистрируйтесь и создайте пост.

## Структура крейтов

- `blog-server`
  - `src/main.rs` — запуск HTTP и gRPC сервисов.
  - `src/presentation` — HTTP-обработчики, middleware и gRPC-сервис.
  - `src/application` — бизнес-логика для авторизации и блога.
  - `src/data` — репозитории PostgreSQL.
  - `src/infrastructure` — подключение к БД, JWT, логирование.
  - `src/domain` — доменные модели и ошибки.

- `blog-client`
  - `src/http_client.rs` — HTTP клиент для вызовов API.
  - `src/grpc_client.rs` — gRPC клиент на основе `tonic`.
  - `src/lib.rs` — общий клиент и модели.

- `blog-cli`
  - `src/main.rs` — консольный интерфейс и парсинг команд.

- `blog-wasm`
  - `src/lib.rs` — WASM-логика для браузера.
  - `index.html` — простая фронтенд-страница.

## Полезные команды

```bash
# Показать зависимости workspace
cargo metadata --no-deps --format-version=1

# Сборка только WASM
cargo build --target wasm32-unknown-unknown -p blog-wasm

# Запустить только CLI для тестирования
cargo run -p blog-cli -- list --limit 10
```

## Заметки

- HTTP API сервера доступен на `http://localhost:3000/api`.
- gRPC сервис доступен на `http://localhost:50051`.
- CLI поддерживает оба транспорта: HTTP и gRPC.
- WASM-приложение использует локальное хранилище браузера для JWT.
>>>>>>> fc205e9 (Init)
