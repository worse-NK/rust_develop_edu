#!/bin/bash

# Скрипт для резервного копирования данных бота

BACKUP_DIR="backups"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/backup_$DATE.tar.gz"

echo "💾 Создание резервной копии данных..."

# Создаем директорию для бэкапов
mkdir -p $BACKUP_DIR

# Создаем архив с данными (без .env)
tar -czf $BACKUP_FILE data/

echo "✅ Резервная копия создана: $BACKUP_FILE"

# Удаляем старые бэкапы (оставляем последние 7)
find $BACKUP_DIR -name "backup_*.tar.gz" -type f -mtime +7 -delete

echo "🧹 Старые бэкапы очищены"