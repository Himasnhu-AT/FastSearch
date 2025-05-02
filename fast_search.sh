#!/bin/bash

# Script to run FastSearch commands more easily
# Usage: ./fast_search.sh [command] [arguments]

# Change to the engine directory
cd "$(dirname "$0")/packages/engine"

# Execute the command
case $1 in
    index)
        echo "Indexing files in directory: $2"
        cargo run index $2
        ;;
    search)
        echo "Searching for: $3 in index: $2"
        cargo run search $2 "$3"
        ;;
    serve)
        echo "Starting server with index: $2"
        if [ -z "$3" ]; then
            cargo run serve $2
        else
            cargo run serve $2 "$3"
        fi
        ;;
    clean)
        echo "Cleaning up index files"
        rm -f index.json index.db
        cargo clean
        ;;
    help|*)
        echo "FastSearch CLI Helper"
        echo "Usage: ./fast_search.sh [command] [arguments]"
        echo ""
        echo "Commands:"
        echo "  index [directory]          - Index the specified directory and save to index.db"
        echo "  search [index] [query]     - Search for query in the specified index file"
        echo "  serve [index] [address]    - Start web server with the specified index (default: 127.0.0.1:6969)"
        echo "  clean                      - Remove index files and clean cargo build"
        echo "  help                       - Display this help message"
        ;;
esac