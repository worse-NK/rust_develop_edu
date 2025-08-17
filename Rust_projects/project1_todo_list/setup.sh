#!/bin/bash

# Скрипт для первоначальной настройки проекта

echo "🚀 Настройка Telegram Todo Bot"
echo "================================"

# Проверяем существование .env файла
if [ ! -f ".env" ]; then
    echo "📝 Создаю .env файл..."
    cp .env.example .env
    echo "✅ Файл .env создан"
    echo ""
    echo "⚠️  ВАЖНО: Отредактируйте файл .env и добавьте ваш токен бота!"
    echo "   1. Получите токен от @BotFather в Telegram"
    echo "   2. Откройте файл .env"
    echo "   3. Замените 'your_bot_token_here' на ваш токен"
    echo ""
else
    echo "✅ Файл .env уже существует"
fi

# Проверяем установку Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust не установлен. Установите Rust с https://rustup.rs/"
    exit 1
fi

echo "🔧 Проверяю зависимости..."
cargo check

if [ $? -eq 0 ]; then
    echo "✅ Все зависимости в порядке"
    echo ""
    echo "🎉 Настройка завершена!"
    echo ""
    echo "Для запуска бота выполните:"
    echo "  cargo run"
else
    echo "❌ Ошибка при проверке зависимостей"
    exit 1
fi