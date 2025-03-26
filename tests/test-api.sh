#!/bin/bash

# Set the server URL
SERVER_URL="http://localhost:8086"

# Display help
function show_help {
    echo "OxiCloud API Testing Script"
    echo "Usage: $0 [options]"
    echo "Options:"
    echo "  -h, --help     Show this help"
    echo "  --list         List files in a folder (use --folder <folder_id> or root if not specified)"
    echo "  --upload       Upload a file (requires --file and optionally --folder)"
    echo "  --download     Download a file (requires --id)"
    echo "  --file <path>  Path to file for upload"
    echo "  --folder <id>  Folder ID (for upload or list operations)"
    echo "  --id <id>      File ID for download operation"
}

# Parse arguments
OPERATION=""
FILE_PATH=""
FOLDER_ID=""
FILE_ID=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --list)
            OPERATION="list"
            shift
            ;;
        --upload)
            OPERATION="upload"
            shift
            ;;
        --download)
            OPERATION="download"
            shift
            ;;
        --file)
            FILE_PATH="$2"
            shift 2
            ;;
        --folder)
            FOLDER_ID="$2"
            shift 2
            ;;
        --id)
            FILE_ID="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Validate arguments
if [[ -z "$OPERATION" ]]; then
    echo "Error: No operation specified."
    show_help
    exit 1
fi

# Execute requested operation
case $OPERATION in
    "list")
        echo "Listing files..."
        if [[ -n "$FOLDER_ID" ]]; then
            echo "Folder ID: $FOLDER_ID"
            curl -s "$SERVER_URL/api/files?folder_id=$FOLDER_ID" | jq .
        else
            echo "Root folder"
            curl -s "$SERVER_URL/api/files" | jq .
        fi
        ;;
    "upload")
        if [[ -z "$FILE_PATH" ]]; then
            echo "Error: File path required for upload."
            exit 1
        fi
        
        if [[ ! -f "$FILE_PATH" ]]; then
            echo "Error: File not found: $FILE_PATH"
            exit 1
        fi
        
        echo "Uploading file: $FILE_PATH"
        if [[ -n "$FOLDER_ID" ]]; then
            echo "To folder: $FOLDER_ID"
            curl -s -X POST \
                -F "file=@$FILE_PATH" \
                -F "folder_id=$FOLDER_ID" \
                "$SERVER_URL/api/files/upload" | jq .
        else
            echo "To root folder"
            curl -s -X POST \
                -F "file=@$FILE_PATH" \
                "$SERVER_URL/api/files/upload" | jq .
        fi
        ;;
    "download")
        if [[ -z "$FILE_ID" ]]; then
            echo "Error: File ID required for download."
            exit 1
        fi
        
        echo "Downloading file: $FILE_ID"
        FILENAME=$(basename "$FILE_ID")
        curl -s -o "$FILENAME" "$SERVER_URL/api/files/$FILE_ID"
        echo "Downloaded to: $FILENAME"
        ;;
esac

echo "Operation completed."