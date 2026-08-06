#!/bin/sh
filepath=$1

if [ ! -s "$filepath" ]; then
    exit 0
fi

if [ "$MINIFY" = "true" ]; then
	# tsとcssを受け取る
	if ! echo "$filepath" | grep -E -q '\.(ts|css)$'; then
    	exit 0
	fi
	ext="${filepath##*.}"
	if [ "$ext" = "ts" ]; then
		outfile=$(echo "$filepath" | sed 's|/resource/ts/|/resource/script-min/|' | sed 's/\.ts$/.js/')
		mkdir -p "$(dirname "$outfile")"
		esbuild "$filepath" --minify --outfile="$outfile" --tsconfig=/app/tsconfig.json --log-level=warning
	elif [ "$ext" = "css" ]; then
		outfile=$(echo "$filepath" | sed 's|/resource/style/|/resource/style-min/|')
		mkdir -p "$(dirname "$outfile")"
		esbuild "$filepath" --minify --outfile="$outfile" --log-level=warning
	fi
else
	# tsのみ受け取る
	if ! echo "$filepath" | grep -E -q '\.ts$'; then
    	exit 0
	fi
	outfile=$(echo "$filepath" | sed 's|/resource/ts/|/resource/script/|' | sed 's/\.ts$/.js/')
	mkdir -p "$(dirname "$outfile")"
	esbuild "$filepath" --outfile="$outfile" --tsconfig=/app/tsconfig.json --log-level=warning
fi
