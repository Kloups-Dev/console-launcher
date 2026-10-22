//! Controller input, read in Rust with gilrs (SDL game controller database,
//! XInput on Windows, evdev on Linux) and sent to the theme as abstract
//! navigation actions. Themes never deal with buttons, so any controller
//! gilrs recognises works with any theme.

use gilrs::{Axis, Button, EventType, Gilrs};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Nav {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    Menu,
    Favorite,
    PageLeft,
    PageRight,
}

const STICK_THRESHOLD: f32 = 0.55;
const REPEAT_DELAY: Duration = Duration::from_millis(380);
const REPEAT_RATE: Duration = Duration::from_millis(95);

fn button_nav(b: Button) -> Option<Nav> {
    Some(match b {
        Button::DPadUp => Nav::Up,
        Button::DPadDown => Nav::Down,
        Button::DPadLeft => Nav::Left,
        Button::DPadRight => Nav::Right,
        Button::South => Nav::Confirm,
        Button::East => Nav::Back,
        Button::North => Nav::Favorite,
        Button::Start | Button::Select => Nav::Menu,
        Button::LeftTrigger => Nav::PageLeft,
        Button::RightTrigger => Nav::PageRight,
        _ => return None,
    })
}

fn stick_nav(x: f32, y: f32) -> Option<Nav> {
    if x.abs() < STICK_THRESHOLD && y.abs() < STICK_THRESHOLD {
        return None;
    }
    Some(if x.abs() > y.abs() {
        if x > 0.0 {
            Nav::Right
        } else {
            Nav::Left
        }
    } else if y > 0.0 {
        Nav::Up
    } else {
        Nav::Down
    })
}

#[derive(Debug, Clone, Serialize)]
struct PadInfo {
    name: String,
    connected: bool,
}

/// Starts the input thread. `paused` is true while a game runs, so the
/// launcher does not react to buttons pressed in the game.
pub fn spawn(app: AppHandle, paused: Arc<AtomicBool>) {
    std::thread::Builder::new()
        .name("gamepad".into())
        .spawn(move || {
            let mut gilrs = match Gilrs::new() {
                Ok(g) => g,
                Err(e) => {
                    log::warn!("controller support unavailable: {e}");
                    return;
                }
            };
            for (_, pad) in gilrs.gamepads() {
                let _ = app.emit(
                    "pad",
                    PadInfo {
                        name: pad.name().to_owned(),
                        connected: true,
                    },
                );
            }
            // Held direction, for auto-repeat.
            let mut held: Option<(Nav, Instant, Instant)> = None;
            let mut stick = (0.0f32, 0.0f32);
            loop {
                while let Some(ev) = gilrs.next_event() {
                    let active = !paused.load(Ordering::Relaxed);
                    match ev.event {
                        EventType::ButtonPressed(b, _) => {
                            if let Some(n) = button_nav(b) {
                                if active {
                                    let _ = app.emit("nav", n);
                                }
                                if matches!(n, Nav::Up | Nav::Down | Nav::Left | Nav::Right) {
                                    let now = Instant::now();
                                    held = Some((n, now, now));
                                }
                            }
                        }
                        EventType::ButtonReleased(b, _) => {
                            if button_nav(b).is_some_and(|n| held.is_some_and(|h| h.0 == n)) {
                                held = None;
                            }
                        }
                        EventType::AxisChanged(axis, v, _) => {
                            match axis {
                                Axis::LeftStickX => stick.0 = v,
                                Axis::LeftStickY => stick.1 = v,
                                _ => continue,
                            }
                            let dir = stick_nav(stick.0, stick.1);
                            if dir != held.map(|h| h.0) {
                                held = dir.map(|d| {
                                    if active {
                                        let _ = app.emit("nav", d);
                                    }
                                    let now = Instant::now();
                                    (d, now, now)
                                });
                            }
                        }
                        EventType::Connected => {
                            let pad = gilrs.gamepad(ev.id);
                            let _ = app.emit(
                                "pad",
                                PadInfo {
                                    name: pad.name().to_owned(),
                                    connected: true,
                                },
                            );
                        }
                        EventType::Disconnected => {
                            let _ = app.emit(
                                "pad",
                                PadInfo {
                                    name: String::new(),
                                    connected: false,
                                },
                            );
                            held = None;
                        }
                        _ => {}
                    }
                }
                if let Some((n, since, last)) = held {
                    let now = Instant::now();
                    if now - since >= REPEAT_DELAY && now - last >= REPEAT_RATE {
                        if !paused.load(Ordering::Relaxed) {
                            let _ = app.emit("nav", n);
                        }
                        held = Some((n, since, now));
                    }
                }
                // While a game runs, poll 25 times less often: the launcher
                // ignores input then and should cost almost nothing.
                let idle = if paused.load(Ordering::Relaxed) {
                    200
                } else {
                    8
                };
                std::thread::sleep(Duration::from_millis(idle));
            }
        })
        .expect("spawn gamepad thread");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stick_directions() {
        assert_eq!(stick_nav(0.9, 0.1), Some(Nav::Right));
        assert_eq!(stick_nav(-0.2, -0.8), Some(Nav::Down));
        assert_eq!(stick_nav(0.3, 0.3), None);
    }
}
