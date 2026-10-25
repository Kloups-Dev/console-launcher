//! Explicit, reversible audio/video trade-offs, not latency measurements.
use cl_core::config::ExperienceMode;
use std::collections::BTreeMap;

pub fn settings(adapter: &str, mode: ExperienceMode) -> BTreeMap<String, String> {
    let responsive = mode == ExperienceMode::Responsive;
    let entries: Vec<(&str, String)> = match adapter {
        "dolphin" => vec![
            ("GFX.Hardware.VSync", (!responsive).to_string()),
            ("GFX.Settings.ShaderCache", "True".into()),
            ("GFX.Settings.WaitForShadersBeforeStarting", "True".into()),
            ("GFX.Settings.ShaderCompilationMode", "2".into()),
            ("GFX.Hacks.VISkip", "False".into()),
            ("Dolphin.Core.EmulationSpeed", "1.0".into()),
            (
                "Dolphin.Core.AudioBufferSize",
                match mode {
                    ExperienceMode::Responsive => "24",
                    ExperienceMode::Balanced => "40",
                    ExperienceMode::Quality => "80",
                }
                .into(),
            ),
        ],
        "retroarch" => vec![
            (
                "audio_latency",
                match mode {
                    ExperienceMode::Responsive => "32",
                    ExperienceMode::Balanced => "48",
                    ExperienceMode::Quality => "64",
                }
                .into(),
            ),
            ("audio_sync", "true".into()),
            ("audio_rate_control", "true".into()),
            ("video_vsync", (!responsive).to_string()),
            ("video_max_swapchain_images", "2".into()),
            ("video_hard_sync", responsive.to_string()),
            ("video_hard_sync_frames", "0".into()),
            ("run_ahead_enabled", "false".into()),
            ("rewind_enable", "false".into()),
        ],
        _ => vec![],
    };
    entries.into_iter().map(|(k, v)| (k.into(), v)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profiles_trade_buffering_without_speed_hacks_or_missing_frames() {
        for mode in [
            ExperienceMode::Responsive,
            ExperienceMode::Balanced,
            ExperienceMode::Quality,
        ] {
            let dolphin = settings("dolphin", mode);
            assert_eq!(dolphin["Dolphin.Core.EmulationSpeed"], "1.0");
            assert_eq!(dolphin["GFX.Hacks.VISkip"], "False");
            assert_eq!(dolphin["GFX.Settings.ShaderCompilationMode"], "2");
            let retro = settings("retroarch", mode);
            assert_eq!(retro["audio_sync"], "true");
            assert_eq!(retro["run_ahead_enabled"], "false");
        }
        assert_eq!(
            settings("dolphin", ExperienceMode::Balanced)["Dolphin.Core.AudioBufferSize"],
            "40"
        );
        assert_eq!(
            settings("retroarch", ExperienceMode::Responsive)["audio_latency"],
            "32"
        );
        assert!(settings("unknown", ExperienceMode::Balanced).is_empty());
    }
}
