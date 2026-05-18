@echo off
echo ====================================
echo   Claude Code v2.1.142
echo   DeepSeek V4 Pro + 自动权限 (auto)
echo ====================================
echo.

set CLAUDE_EXE=%APPDATA%\npm\node_modules\@anthropic-ai\claude-code-win32-x64\claude.exe
set CC_SWITCH=node "%APPDATA%\npm\node_modules\@aravhawk\cc-switch\dist\index.js"

if not exist "%CLAUDE_EXE%" (
    echo [错误] claude.exe 未找到！
    echo 请先运行: npm install -g @anthropic-ai/claude-code-win32-x64
    pause
    exit /b 1
)

echo [OK] Claude Code: %CLAUDE_EXE%
echo [OK] CC Switch:  deepseek-v4-pro
echo.

rem 同步 CC Switch profile 到 settings.json
%CC_SWITCH% deepseek-v4-pro 2>nul

echo 正在启动...
echo.

"%CLAUDE_EXE%" --permission-mode auto
