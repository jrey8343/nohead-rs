#!/bin/sh

if [ ! -f "$DATABASE_PATH" ]; then
  echo "Database does not exist. Running setup..."
  sqlx database setup --source /app/db/migrations -D ${DATABASE_URL}
else
  echo "Database exists. Running migrations..."
  sqlx migrate run --source /app/db/migrations -D ${DATABASE_URL}
fi
