cmake_minimum_required(VERSION 3.21)
project(ConsoleLauncher VERSION 0.3.2 LANGUAGES CXX)

# Console Launcher — C++17 + Qt Quick interface. The functional QML shell is
# compiled into the executable; controllers go through SDL2, and the Rust
# engine (crate cl-ffi) is a shared library loaded by the application.

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

# Latest Qt long-term support series.
set(CL_QT_MIN_VERSION 6.8 CACHE STRING "Minimum Qt version")
find_package(Qt6 ${CL_QT_MIN_VERSION} REQUIRED COMPONENTS Core Gui Qml Quick QuickDialogs2)
find_package(SDL2 REQUIRED)
qt_standard_project_setup(REQUIRES 6.8)

# ---- Rust engine --------------------------------------------------------
# Built with cargo from the workspace root. Set CL_ENGINE_PREBUILT_DIR to a
# folder holding an already built library to skip cargo.
set(CL_WORKSPACE ${CMAKE_CURRENT_SOURCE_DIR}/..)
set(CL_ENGINE_PREBUILT_DIR "" CACHE PATH "Folder of a prebuilt engine library")
if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    set(CL_CARGO_PROFILE debug)
    set(CL_CARGO_FLAG "")
else()
    set(CL_CARGO_PROFILE release)
    set(CL_CARGO_FLAG --release)
endif()
if(CL_ENGINE_PREBUILT_DIR)
    set(CL_ENGINE_DIR ${CL_ENGINE_PREBUILT_DIR})
else()
    set(CL_ENGINE_DIR ${CMAKE_BINARY_DIR}/cargo/${CL_CARGO_PROFILE})
endif()
if(WIN32)
    set(CL_ENGINE_LIB ${CL_ENGINE_DIR}/console_launcher_engine.dll)
    set(CL_ENGINE_IMPLIB ${CL_ENGINE_DIR}/console_launcher_engine.dll.lib)
else()
    set(CL_ENGINE_LIB ${CL_ENGINE_DIR}/libconsole_launcher_engine.so)
endif()

if(NOT CL_ENGINE_PREBUILT_DIR)
    find_program(CARGO cargo REQUIRED)
    add_custom_target(cl_engine_build ALL
        COMMAND ${CMAKE_COMMAND} -E env CARGO_TARGET_DIR=${CMAKE_BINARY_DIR}/cargo
                ${CARGO} build ${CL_CARGO_FLAG} -p cl-ffi -p cl-cli
        WORKING_DIRECTORY ${CL_WORKSPACE}
        BYPRODUCTS ${CL_ENGINE_LIB} ${CL_ENGINE_IMPLIB}
        COMMENT "Building the Rust engine"
        USES_TERMINAL)
endif()

add_library(cl_engine SHARED IMPORTED)
set_target_properties(cl_engine PROPERTIES
    IMPORTED_LOCATION ${CL_ENGINE_LIB}
    INTERFACE_INCLUDE_DIRECTORIES ${CL_WORKSPACE}/crates/cl-ffi/include)
if(WIN32)
    set_target_properties(cl_engine PROPERTIES IMPORTED_IMPLIB ${CL_ENGINE_IMPLIB})
endif()

# ---- Application --------------------------------------------------------
qt_add_executable(console-launcher-qt
    src/main.cpp
    src/backend.cpp
    src/backend.h
    src/engine.cpp
    src/engine.h
    src/gamepad.cpp
    src/gamepad.h
    src/keyfilter.cpp
    src/keyfilter.h
    src/perfmonitor.cpp
    src/perfmonitor.h
    src/framestream.h
    src/systemtelemetry.cpp
    src/systemtelemetry.h
)
if(NOT CL_ENGINE_PREBUILT_DIR)
    add_dependencies(console-launcher-qt cl_engine_build)
endif()

# Trusted QML components compiled into the application.
qt_add_qml_module(console-launcher-qt
    URI ConsoleLauncher
    VERSION 1.0
    QML_FILES
        ui/DesktopApp.qml
        ui/SettingsPanel.qml
        ui/SettingsRow.qml
        ui/ControlVisualizer.qml
        ui/ControlPad.qml
        ui/XboxControlPad.qml
        ui/GameCubeControlPad.qml
        ui/PS2ControlPad.qml
        ui/PS5ControlPad.qml
        ui/PerformanceOverlay.qml
)

if(WIN32)
    set_target_properties(console-launcher-qt PROPERTIES WIN32_EXECUTABLE ON)
    target_sources(console-launcher-qt PRIVATE src/app.rc)
    target_link_libraries(console-launcher-qt PRIVATE user32 psapi pdh)
else()
    # The engine library sits next to the executable.
    set_target_properties(console-launcher-qt PROPERTIES
        BUILD_RPATH "$ORIGIN" INSTALL_RPATH "$ORIGIN")
endif()

target_compile_definitions(console-launcher-qt PRIVATE
    SDL_MAIN_HANDLED QT_NO_KEYWORDS
    CL_VERSION="${PROJECT_VERSION}")
target_link_libraries(console-launcher-qt PRIVATE
    Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick
    SDL2::SDL2
    cl_engine
)

# The engine library is copied next to the executable. User-provided QML is
# deliberately not copied or executed; future themes will be data-only packs.
add_custom_command(TARGET console-launcher-qt POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
            ${CL_ENGINE_LIB} $<TARGET_FILE_DIR:console-launcher-qt>)

option(CL_PERFORMANCE_TESTS "Build rolling frame stream tests" OFF)
if(CL_PERFORMANCE_TESTS)
    enable_testing()
    add_executable(performance-stream-tests tests/perfstream_test.cpp)
    target_include_directories(performance-stream-tests PRIVATE src)
    target_link_libraries(performance-stream-tests PRIVATE Qt6::Core)
    add_test(NAME performance-stream COMMAND performance-stream-tests)
    if(WIN32)
        add_executable(performance-monitor-tests tests/perfmonitor_test.cpp src/perfmonitor.cpp src/perfmonitor.h src/framestream.h src/systemtelemetry.cpp src/systemtelemetry.h)
        target_include_directories(performance-monitor-tests PRIVATE src)
        target_link_libraries(performance-monitor-tests PRIVATE Qt6::Gui user32 psapi pdh)
        add_test(NAME performance-monitor COMMAND performance-monitor-tests)
    endif()
endif()
