cmake_minimum_required(VERSION 3.21)
project(ConsoleLauncherQt VERSION 0.1.0 LANGUAGES CXX)

# Third interface prototype: C++17 + Qt Quick, themes in QML loaded from disk,
# controller input through SDL2, engine = the Rust `clctl` tool (cl-engine).

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 6.4 REQUIRED COMPONENTS Core Gui Qml Quick)
find_package(SDL2 REQUIRED)

add_executable(console-launcher-qt
    src/main.cpp
    src/backend.cpp
    src/backend.h
    src/gamepad.cpp
    src/gamepad.h
    src/keyfilter.cpp
    src/keyfilter.h
)

if(WIN32)
    set_target_properties(console-launcher-qt PROPERTIES WIN32_EXECUTABLE ON)
    target_sources(console-launcher-qt PRIVATE src/app.rc)
endif()

target_compile_definitions(console-launcher-qt PRIVATE SDL_MAIN_HANDLED QT_NO_KEYWORDS)
target_link_libraries(console-launcher-qt PRIVATE
    Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick
    SDL2::SDL2
)

# Themes are plain folders next to the executable, editable without rebuilding.
add_custom_command(TARGET console-launcher-qt POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_directory
            ${CMAKE_CURRENT_SOURCE_DIR}/themes $<TARGET_FILE_DIR:console-launcher-qt>/themes)
