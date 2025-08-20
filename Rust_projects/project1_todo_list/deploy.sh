#!/bin/bash

# Скрипт для развертывания бота на сервере

set -e

echo "🚀 Развертывание Telegram Todo Bot"
echo "=================================="

# Проверяем наличие Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker не установлен. Установите Docker и попробуйте снова."
    exit 1
fi

# Проверяем наличие docker-compose
if ! command -v docker-compose &> /dev/null; then
    echo "❌ docker-compose не установлен. Установите docker-compose и попробуйте снова."
    exit 1
fi

# Проверяем наличие .env файла
if [ ! -f ".env" ]; then
    echo "📝 Создаю .env файл из примера..."
    cp .env.example .env
    echo ""
    echo "⚠️  ВАЖНО: Отредактируйте файл .env и добавьте ваш токен бота!"
    echo "   Откройте файл .env и замените 'your_bot_token_here' на реальный токен"
    echo ""
    read -p "Нажмите Enter после редактирования .env файла..."
fi

# Создаем необходимые директории
echo "📁 Создаю директории..."
mkdir -p data logs

# Останавливаем существующие контейнеры
echo "🛑 Останавливаю существующие контейнеры..."
docker-compose down || true

# Собираем образ
echo "🔨 Собираю Docker образ..."
docker-compose build

# Запускаем контейнер
echo "▶️  Запускаю бота..."
docker-compose up -d

# Проверяем статус
echo "📊 Проверяю статус..."
sleep 5
docker-compose ps

# Показываем логи
echo ""
echo "📋 Последние логи:"
docker-compose logs --tail=20

echo ""
echo "✅ Развертывание завершено!"
echo ""
echo "Полезные команды:"
echo "  docker-compose logs -f          # Просмотр логов в реальном времени"
echo "  docker-compose restart          # Перезапуск бота"
echo "  docker-compose down             # Остановка бота"
echo "  docker-compose up -d            # Запуск бота в фоне"