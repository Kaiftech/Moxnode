@echo off
cd /d "%~dp0"
if not exist moxnode.exe (
  call build.bat
  if errorlevel 1 exit /b 1
)
echo Starting Moxnode - infinite loop. Ctrl+C to stop.
moxnode.exe %*
