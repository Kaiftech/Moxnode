@echo off
cd /d "%~dp0"
echo Building moxnode (release + rayon)...
cargo build --release
if errorlevel 1 exit /b 1
copy /Y target\release\moxnode.exe moxnode.exe >nul
echo.
echo Ready: moxnode.exe
echo   moxnode.exe           - loop forever
echo   moxnode.exe --fast    - rayon swarm
echo   moxnode.exe --once    - one tick
