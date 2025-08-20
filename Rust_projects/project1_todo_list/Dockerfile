# Многоэтапная сборка для оптимизации размера образа
FROM rust:1.82-slim as builder

# Устанавливаем системные зависимости
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Создаем рабочую директорию
WORKDIR /app

# Копируем исходный код и манифесты сразу (упрощаем и делаем сборку детерминированной)
COPY Cargo.toml ./
COPY src ./src

# Собираем приложение
RUN cargo build --release

# Финальный образ
FROM debian:bookworm-slim

# Устанавливаем runtime зависимости
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Создаем пользователя для безопасности
RUN useradd -r -s /bin/false todobot

# Создаем рабочую директорию
WORKDIR /app

# Копируем бинарник из builder stage
COPY --from=builder /app/target/release/telegram_todo_bot /app/telegram_todo_bot

# Создаем директорию для данных
RUN mkdir -p /app/data && chown todobot:todobot /app/data

# Переключаемся на непривилегированного пользователя
USER todobot

# Открываем порт (если понадобится в будущем)
EXPOSE 8080

# Запускаем приложение
CMD ["./telegram_todo_bot"]