#!/bin/bash
# Generate a detailed clippy lint report

echo "=== Clippy Lint Report ==="
echo "Generated: $(date)"
echo ""

# Run clippy and save to temp file
CLIPPY_OUTPUT=$(mktemp)
cargo clippy --all-targets --all-features --message-format=json 2>&1 > "$CLIPPY_OUTPUT"

echo "## Summary by Lint Type:"
echo ""

# Count warnings by lint type
jq -r 'select(.reason == "compiler-message" and .message.level == "error") | .message.code.code' "$CLIPPY_OUTPUT" 2>/dev/null \
  | grep "clippy::" \
  | sort \
  | uniq -c \
  | sort -rn

echo ""
echo "## Detailed Locations:"
echo ""

# Show file:line for each warning type
for lint in "zombie-processes" "needless-borrows-for-generic-args" "suspicious-open-options" "let-and-return"; do
  echo "### clippy::$lint"
  jq -r "select(.reason == \"compiler-message\" and .message.level == \"error\" and .message.code.code == \"clippy::$lint\") |
         \"\(.target.src_path):\(.message.spans[0].line_start)\"" "$CLIPPY_OUTPUT" 2>/dev/null \
    | sort -u
  echo ""
done

rm "$CLIPPY_OUTPUT"
