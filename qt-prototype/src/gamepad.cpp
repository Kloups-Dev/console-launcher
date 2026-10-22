#include "gamepad.h"

#include <SDL.h>

namespace {
constexpr int kPollMs = 8;
constexpr int kRepeatDelayMs = 380;
constexpr int kRepeatRateMs = 95;
constexpr float kStickThreshold = 0.55f;

QString buttonAction(Uint8 b) {
    switch (b) {
    case SDL_CONTROLLER_BUTTON_DPAD_UP: return "up";
    case SDL_CONTROLLER_BUTTON_DPAD_DOWN: return "down";
    case SDL_CONTROLLER_BUTTON_DPAD_LEFT: return "left";
    case SDL_CONTROLLER_BUTTON_DPAD_RIGHT: return "right";
    case SDL_CONTROLLER_BUTTON_A: return "confirm";
    case SDL_CONTROLLER_BUTTON_B: return "back";
    case SDL_CONTROLLER_BUTTON_Y: return "favorite";
    case SDL_CONTROLLER_BUTTON_START:
    case SDL_CONTROLLER_BUTTON_BACK: return "menu";
    case SDL_CONTROLLER_BUTTON_LEFTSHOULDER: return "pageLeft";
    case SDL_CONTROLLER_BUTTON_RIGHTSHOULDER: return "pageRight";
    default: return {};
    }
}

bool isDirection(const QString &a) {
    return a == "up" || a == "down" || a == "left" || a == "right";
}
} // namespace

Gamepad::Gamepad(QObject *parent) : QObject(parent) {
    m_timer.setInterval(kPollMs);
    m_timer.setTimerType(Qt::PreciseTimer);
    connect(&m_timer, &QTimer::timeout, this, &Gamepad::poll);
    start();
}

Gamepad::~Gamepad() { stop(); }

void Gamepad::start() {
    if (m_started)
        return;
    SDL_SetHint(SDL_HINT_JOYSTICK_ALLOW_BACKGROUND_EVENTS, "1");
    if (SDL_Init(SDL_INIT_GAMECONTROLLER) != 0)
        return; // no controller support; keyboard still works
    m_started = true;
    for (int i = 0; i < SDL_NumJoysticks(); ++i)
        if (SDL_IsGameController(i))
            if (SDL_GameController *c = SDL_GameControllerOpen(i))
                Q_EMIT padName(QString::fromUtf8(SDL_GameControllerName(c)));
    m_timer.start();
}

void Gamepad::stop() {
    m_timer.stop();
    if (m_started) {
        SDL_QuitSubSystem(SDL_INIT_GAMECONTROLLER);
        m_started = false;
    }
    m_held.clear();
}

void Gamepad::setPaused(bool paused) {
    if (paused)
        stop();
    else
        start();
}

void Gamepad::press(const QString &dir) {
    if (dir == m_held)
        return;
    m_held = dir;
    if (!dir.isEmpty()) {
        Q_EMIT nav(dir);
        m_heldSince.start();
        m_lastRepeat = 0;
    }
}

void Gamepad::poll() {
    SDL_Event e;
    while (SDL_PollEvent(&e)) {
        switch (e.type) {
        case SDL_CONTROLLERDEVICEADDED:
            if (SDL_GameController *c = SDL_GameControllerOpen(e.cdevice.which))
                Q_EMIT padName(QString::fromUtf8(SDL_GameControllerName(c)));
            break;
        case SDL_CONTROLLERDEVICEREMOVED:
            Q_EMIT padName(QString());
            m_held.clear();
            break;
        case SDL_CONTROLLERBUTTONDOWN: {
            const QString a = buttonAction(e.cbutton.button);
            if (a.isEmpty())
                break;
            if (isDirection(a)) {
                m_held.clear();
                press(a);
            } else {
                Q_EMIT nav(a);
            }
            break;
        }
        case SDL_CONTROLLERBUTTONUP:
            if (buttonAction(e.cbutton.button) == m_held)
                m_held.clear();
            break;
        case SDL_CONTROLLERAXISMOTION: {
            const float v = e.caxis.value / 32767.0f;
            if (e.caxis.axis == SDL_CONTROLLER_AXIS_LEFTX)
                m_x = v;
            else if (e.caxis.axis == SDL_CONTROLLER_AXIS_LEFTY)
                m_y = v;
            else
                break;
            QString dir;
            if (qAbs(m_x) >= kStickThreshold || qAbs(m_y) >= kStickThreshold)
                dir = qAbs(m_x) > qAbs(m_y) ? (m_x > 0 ? "right" : "left")
                                            : (m_y > 0 ? "down" : "up");
            press(dir);
            break;
        }
        default:
            break;
        }
    }
    if (!m_held.isEmpty()) {
        const qint64 t = m_heldSince.elapsed();
        if (t >= kRepeatDelayMs && t - m_lastRepeat >= kRepeatRateMs) {
            m_lastRepeat = t;
            Q_EMIT nav(m_held);
        }
    }
}
