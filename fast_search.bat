@echo off

REM script not tested on Windows


REM Script to run FastSearch commands more easily
REM Usage: fast_search.bat [command] [arguments]

REM Change to the engine directory
cd "%~dp0\packages\engine"

REM Execute the command
if "%1"=="index" (
    echo Indexing files in directory: %2
    cargo run index %2
) else if "%1"=="search" (
    echo Searching for: %3 in index: %2
    cargo run search %2 "%3"
) else if "%1"=="serve" (
    echo Starting server with index: %2
    if "%3"=="" (
        cargo run serve %2
    ) else (
        cargo run serve %2 %3
    )
) else if "%1"=="clean" (
    echo Cleaning up index files
    del /q index.json index.db 2>nul
    cargo clean
) else (
    echo FastSearch CLI Helper
    echo Usage: fast_search.bat [command] [arguments]
    echo.
    echo Commands:
    echo   index [directory]          - Index the specified directory and save to index.db
    echo   search [index] [query]     - Search for query in the specified index file
    echo   serve [index] [address]    - Start web server with the specified index (default: 127.0.0.1:6969)
    echo   clean                      - Remove index files and clean cargo build
    echo   help                       - Display this help message
)