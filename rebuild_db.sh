#!/bin/bash

# Rebuild database script for Solji Indexer
# This script drops and recreates all tables to apply schema changes

echo "🔄 Rebuilding Solji Indexer Database..."

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | xargs)
fi

# Default database URL if not set
DB_URL=${DATABASE_URL:-"mysql://root:password@localhost:3306/solji"}

echo "📊 Database URL: $DB_URL"

# Extract database name from URL
DB_NAME=$(echo $DB_URL | sed 's/.*\/\([^?]*\).*/\1/')

echo "🗄️  Database Name: $DB_NAME"

# MySQL commands to drop and recreate database
MYSQL_COMMANDS="
DROP DATABASE IF EXISTS $DB_NAME;
CREATE DATABASE $DB_NAME CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
"

echo "💥 Dropping and recreating database..."
mysql -e "$MYSQL_COMMANDS"

if [ $? -eq 0 ]; then
    echo "✅ Database rebuilt successfully!"
    echo "🚀 You can now run the indexer to recreate all tables."
else
    echo "❌ Failed to rebuild database!"
    exit 1
fi
