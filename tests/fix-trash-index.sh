#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== OxiCloud Trash Index Fix Script ===${NC}"

# Configuration
TRASH_INDEX_FILE="./storage/.trash/trash_index.json"

# Check if the trash index file exists
if [ ! -f "$TRASH_INDEX_FILE" ]; then
    echo -e "${RED}Trash index file not found at: $TRASH_INDEX_FILE${NC}"
    exit 1
fi

# Backup the trash index file
BACKUP_FILE="${TRASH_INDEX_FILE}.bak"
cp "$TRASH_INDEX_FILE" "$BACKUP_FILE"
echo -e "${GREEN}Created backup at: $BACKUP_FILE${NC}"

# Parse and filter out problematic entries
echo -e "${YELLOW}Analyzing and fixing trash index...${NC}"
TEMP_FILE=$(mktemp)

# Read the current trash index
cat "$TRASH_INDEX_FILE" | jq '.' > "$TEMP_FILE"

# Check if there are any entries
ENTRY_COUNT=$(cat "$TEMP_FILE" | jq 'length')
echo -e "${YELLOW}Found $ENTRY_COUNT entries in trash index${NC}"

if [ "$ENTRY_COUNT" -eq 0 ]; then
    echo -e "${GREEN}Trash index is empty, nothing to fix${NC}"
    rm "$TEMP_FILE"
    exit 0
fi

# Problematic IDs (hardcoded based on error messages)
PROBLEMATIC_IDS=("ee30543b-9268-4fb1-8085-9d140f756187")

# Filter out problematic entries
for ID in "${PROBLEMATIC_IDS[@]}"; do
    echo -e "${YELLOW}Removing entries for original_id: $ID${NC}"
    cat "$TEMP_FILE" | jq "[.[] | select(.original_id != \"$ID\")]" > "${TEMP_FILE}.new"
    mv "${TEMP_FILE}.new" "$TEMP_FILE"
done

# Verify the new contents
NEW_ENTRY_COUNT=$(cat "$TEMP_FILE" | jq 'length')
echo -e "${GREEN}Trash index now contains $NEW_ENTRY_COUNT entries${NC}"

# Write back the fixed index
cat "$TEMP_FILE" > "$TRASH_INDEX_FILE"
rm "$TEMP_FILE"

echo -e "${GREEN}Trash index has been fixed!${NC}"
echo -e "${YELLOW}Original index was backed up to: $BACKUP_FILE${NC}"