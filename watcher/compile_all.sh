#!/bin/sh
set -e

find app \( -path "*/resource/ts/*.ts" -o -path "*/resource/style/*.css" \) \( -name "*.ts" -o -name "*.css" \) -type f | while IFS= read -r file; do
	sh compile.sh "$file"
	done
