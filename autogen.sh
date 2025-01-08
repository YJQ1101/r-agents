#!/bin/bash -e

BASE_AGENTS_DIR="src/agents"
OUTPUT_AGENTS_FILE="agents.yaml"

echo "agents:" > "$OUTPUT_AGENTS_FILE"

find "$BASE_AGENTS_DIR" -type f -name "*.yaml" | while read -r file; do
    relative_path="${file#${BASE_AGENTS_DIR}/}"
    
    folder_name=$(basename "$(dirname "$relative_path")")

    echo "  $folder_name: $BASE_AGENTS_DIR/$relative_path" >> "$OUTPUT_AGENTS_FILE"
done

echo "agents file: $OUTPUT_AGENTS_FILE"

BASE_TOOLS_DIR="src/tools"
OUTPUT_TOOLS_FILE="tools.yaml"

echo "tools:" > "$OUTPUT_TOOLS_FILE"
find "$BASE_TOOLS_DIR" -type f -name "*.yaml" | while read -r file; do
    relative_path="${file#${BASE_TOOLS_DIR}/}"
    
    folder_name=$(basename "$(dirname "$relative_path")")

    echo "  $folder_name: $BASE_TOOLS_DIR/$relative_path" >> "$OUTPUT_TOOLS_FILE"
done
echo "tools file: $OUTPUT_TOOLS_FILE"

BASE_RAGS_DIR="src/rags"
OUTPUT_RAGS_FILE="rags.yaml"

echo "rags:" > "$OUTPUT_RAGS_FILE"

find "$BASE_RAGS_DIR" -type f -name "*.yaml" | while read -r file; do
    relative_path="${file#${BASE_RAGS_DIR}/}"
    
    folder_name=$(basename "$(dirname "$relative_path")")

    echo "  $folder_name: $BASE_RAGS_DIR/$relative_path" >> "$OUTPUT_RAGS_FILE"
done

echo "rags file: $OUTPUT_RAGS_FILE"