#!/usr/bin/env bash
set -euo pipefail

REPO="cesarecaoduro/rusted-geom"
LABELS_FILE=".github/labels.yml"

if ! command -v gh &> /dev/null; then
  echo "Error: GitHub CLI (gh) is required. Install from https://cli.github.com"
  exit 1
fi

if ! command -v yq &> /dev/null; then
  echo "Parsing labels with grep/sed (yq not found)..."

  name="" color="" desc=""
  while IFS= read -r line; do
    if [[ "$line" =~ ^-\ name:\ (.+) ]]; then
      name="${BASH_REMATCH[1]}"
      name="${name#\"}" ; name="${name%\"}"
    elif [[ "$line" =~ ^\ \ color:\ (.+) ]]; then
      color="${BASH_REMATCH[1]}"
      color="${color#\"}" ; color="${color%\"}"
    elif [[ "$line" =~ ^\ \ description:\ (.+) ]]; then
      desc="${BASH_REMATCH[1]}"
      desc="${desc#\"}" ; desc="${desc%\"}"

      echo "  -> $name (#$color)"
      gh label create "$name" --color "$color" --description "$desc" --repo "$REPO" --force 2>/dev/null || true
      name="" ; color="" ; desc=""
    fi
  done < "$LABELS_FILE"
else
  count=$(yq '.| length' "$LABELS_FILE")
  for ((i=0; i<count; i++)); do
    name=$(yq -r ".[$i].name" "$LABELS_FILE")
    color=$(yq -r ".[$i].color" "$LABELS_FILE")
    desc=$(yq -r ".[$i].description" "$LABELS_FILE")

    echo "  -> $name (#$color)"
    gh label create "$name" --color "$color" --description "$desc" --repo "$REPO" --force 2>/dev/null || true
  done
fi

echo "Labels synced."
