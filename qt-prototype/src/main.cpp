// Console Launcher — Qt Quick prototype.

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>

#include "backend.h"
#include "gamepad.h"
#include "keyfilter.h"

int main(int argc, char *argv[]) {
    QGuiApplication app(argc, argv);
    app.setOrganizationName("ConsoleLauncher");
    app.setApplicationName("console-launcher-qt");

    Backend backend;
    Gamepad pad;
    KeyFilter keys;
    app.installEventFilter(&keys);

    QObject::connect(&pad, &Gamepad::nav, &backend, &Backend::emitNav);
    QObject::connect(&pad, &Gamepad::padName, &backend, &Backend::setPadName);
    QObject::connect(&keys, &KeyFilter::nav, &backend, &Backend::emitNav);
    QObject::connect(&keys, &KeyFilter::fullscreen, &backend, &Backend::toggleFullscreen);
    QObject::connect(&backend, &Backend::sessionActive, &pad, &Gamepad::setPaused);

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty("Launcher", &backend);
    backend.setQmlEngine(&engine);
    backend.loadTheme();
    backend.reload();
    return app.exec();
}
