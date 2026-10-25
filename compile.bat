@echo off
setlocal EnableExtensions

rem Console Launcher - build the Qt application on Windows.
rem Run from a source checkout or from the ZIP downloaded with "Code".

cd /d "%~dp0"
if not exist "qt-prototype\CMakeLists.txt" (
    echo [ERROR] qt-prototype\CMakeLists.txt was not found.
    echo Extract the source ZIP completely, then run compile.bat from its root.
    exit /b 1
)

where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Rust Cargo was not found in PATH.
    echo Install Rust from https://rustup.rs/ and open a new terminal.
    exit /b 1
)
where cmake >nul 2>&1
if errorlevel 1 (
    echo [ERROR] CMake was not found in PATH.
    echo Install CMake 3.21 or newer and reopen the terminal.
    exit /b 1
)

set "CMAKE_ARGS="
if defined CL_GENERATOR set "CMAKE_ARGS=-G %CL_GENERATOR%"

rem Ninja avoids the need to select a Visual Studio generator manually.
if not defined CL_GENERATOR (
    where ninja >nul 2>&1
    if not errorlevel 1 set "CMAKE_ARGS=-G Ninja"
)

rem Qt Creator and a Qt developer prompt normally provide these variables.
rem The common standalone Qt layout is also detected automatically.
if not defined CMAKE_PREFIX_PATH (
    for %%Q in (
        "C:\Qt\6.8.3\msvc2022_64"
        "C:\Qt\6.8.2\msvc2022_64"
        "C:\Qt\6.8.1\msvc2022_64"
        "C:\Qt\6.8.0\msvc2022_64"
    ) do if exist "%%~Q\lib\cmake\Qt6\Qt6Config.cmake" (
        set "CMAKE_PREFIX_PATH=%%~Q"
        goto :qt_found
    )
)
:qt_found

if not defined SDL2_DIR (
    for %%S in (
        "C:\SDL2-2.32.10\cmake"
        "C:\SDL2-2.30.11\cmake"
        "C:\SDL2\cmake"
    ) do if exist "%%~S\SDL2Config.cmake" (
        set "SDL2_DIR=%%~S"
        goto :sdl_found
    )
)
:sdl_found

if not defined CMAKE_PREFIX_PATH (
    echo [ERROR] Qt 6.8 was not found.
    echo Set CMAKE_PREFIX_PATH to the Qt kit folder, for example:
    echo   set CMAKE_PREFIX_PATH=C:\Qt\6.8.3\msvc2022_64
    exit /b 1
)
if not defined SDL2_DIR (
    echo [ERROR] SDL2 development files were not found.
    echo Set SDL2_DIR to the SDL2 cmake folder, for example:
    echo   set SDL2_DIR=C:\SDL2-2.32.10\cmake
    exit /b 1
)

echo [INFO] Qt:   %CMAKE_PREFIX_PATH%
echo [INFO] SDL2: %SDL2_DIR%
echo [INFO] Build: build-qt

echo [INFO] Building the Rust engine...
cargo build --release -p cl-ffi -p cl-cli
if errorlevel 1 exit /b 1

cmake -S qt-prototype -B build-qt %CMAKE_ARGS% -DCMAKE_BUILD_TYPE=Release -DCMAKE_PREFIX_PATH="%CMAKE_PREFIX_PATH%" -DSDL2_DIR="%SDL2_DIR%" -DCL_ENGINE_PREBUILT_DIR="%CD%\target\release"
if errorlevel 1 exit /b 1

cmake --build build-qt --config Release
if errorlevel 1 exit /b 1

if /i "%~1"=="test" (
    echo [INFO] Running the packaged-interface smoke test...
    if exist "build-qt\Release\console-launcher-qt.exe" (
        "build-qt\Release\console-launcher-qt.exe" --smoke-test
    ) else (
        "build-qt\console-launcher-qt.exe" --smoke-test
    )
    if errorlevel 1 exit /b 1
)

echo.
echo [OK] Build complete.
echo Executable:
if exist "build-qt\Release\console-launcher-qt.exe" echo   build-qt\Release\console-launcher-qt.exe
if exist "build-qt\console-launcher-qt.exe" echo   build-qt\console-launcher-qt.exe
echo.
echo Run "compile.bat test" to build and run the smoke test.
exit /b 0
