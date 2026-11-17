@echo off
setlocal enabledelayedexpansion
chcp 65001 > nul

cd /d "%~dp0"

set "TARGET_DIR=%~dp0"
if "%TARGET_DIR:~-1%"=="\" set "TARGET_DIR=%TARGET_DIR:~0,-1%"

for %%I in ("%TARGET_DIR%") do set "PROJECT_NAME=%%~nxI"
set "OUTPUT_FILE=%TARGET_DIR%\project_structure.txt"

for /f "usebackq delims=" %%I in (`powershell -NoProfile -Command "Get-Date -Format 'yyyy-MM-dd HH:mm:ss'"`) do set "TIMESTAMP=%%I"

echo ================================================================================ > "%OUTPUT_FILE%"
echo PROJECT STRUCTURE REPORT >> "%OUTPUT_FILE%"
echo ================================================================================ >> "%OUTPUT_FILE%"
echo Project Name : %PROJECT_NAME% >> "%OUTPUT_FILE%"
echo Root Path    : %TARGET_DIR% >> "%OUTPUT_FILE%"
echo Generated At : %TIMESTAMP% >> "%OUTPUT_FILE%"
echo ================================================================================ >> "%OUTPUT_FILE%"
echo. >> "%OUTPUT_FILE%"

tree "%TARGET_DIR%" /F /A >> "%OUTPUT_FILE%"

echo. >> "%OUTPUT_FILE%"
echo ================================================================================ >> "%OUTPUT_FILE%"
echo END OF STRUCTURE REPORT >> "%OUTPUT_FILE%"
echo ================================================================================ >> "%OUTPUT_FILE%"

cls
echo ================================================================================
echo   DIRECTORY TREE GENERATED SUCCESSFULLY
echo ================================================================================
echo.
echo   Project Name : %PROJECT_NAME%
echo   Output Path  : %OUTPUT_FILE%
echo.
echo ================================================================================
pause