#!/usr/bin/env sh
set -eu

max_lines="${MAX_LINES:-300}"
failed=0

for file in "$@"; do
  lines="$(wc -l < "$file" | tr -d ' ')"

  if [ "$lines" -gt "$max_lines" ]; then
    echo "$file has $lines lines; limit is $max_lines"
    failed=1
  fi
done

exit "$failed"

