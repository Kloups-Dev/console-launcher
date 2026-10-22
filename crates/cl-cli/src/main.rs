//! `clctl`: drives the engine from a terminal, to test it without the interface.

use anyhow::{anyhow, bail, Context, Result};
use cl_core::paths::Paths;
use cl_core::Game;
use cl_engine::Engine;
use cl_updater::{decide, sources, Decision, ManagedEmulator};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clctl", version, about = "Console Launcher command-line tool")]
struct Cli {
    /// Data folder (default: the standard per-user folder, or CONSOLE_LAUNCHER_HOME).
    #[arg(long, global = true)]
    home: Option<std::path::PathBuf>,
    /// JSON output.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Detect hardware and run the CPU micro-benchmark.
    Probe,
    /// Add a game folder to the settings.
    AddRoms { dir: std::path::PathBuf },
    /// List systems and games found.
    Scan,
    /// Show what would be launched for a game (title search).
    Plan { query: String },
    /// Launch a game, measure if needed, update its profile.
    Play { query: String },
    /// Analyse a frame-time capture (MangoHud / PresentMon CSV).
    Analyse {
        file: std::path::PathBuf,
        /// Record the result in this game's profile.
        #[arg(long)]
        game: Option<String>,
    },
    /// Force a quality rung for a game (0 = lightest).
    SetStep { query: String, step: usize },
    /// Emulators: detection and update status.
    Emulators,
    /// Check, download and install the latest official release of a managed emulator.
    Update {
        id: String,
        /// Only report, do not install.
        #[arg(long)]
        check: bool,
    },
    /// Go back to the previously installed version of a managed emulator.
    Rollback { id: String },
}

fn find_game(engine: &Engine, query: &str) -> Result<Game> {
    let q = query.to_lowercase();
    let games = engine.games();
    let mut hits: Vec<_> = games
        .iter()
        .filter(|g| g.id == query || g.title.to_lowercase().contains(&q))
        .collect();
    match hits.len() {
        0 => bail!("no game matches '{query}'"),
        1 => Ok(hits.remove(0).clone()),
        n => {
            let names: Vec<_> = hits
                .iter()
                .take(10)
                .map(|g| format!("  {} [{}] {}", g.id, g.system, g.title))
                .collect();
            bail!(
                "{n} games match '{query}', be more precise or use the id:\n{}",
                names.join("\n")
            )
        }
    }
}

fn print<T: serde::Serialize>(json: bool, value: &T, text: impl FnOnce()) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(value).unwrap_or_default()
        );
    } else {
        text();
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let cli = Cli::parse();
    let paths = cli
        .home
        .clone()
        .map(Paths::at)
        .unwrap_or_else(Paths::discover);
    let mut engine = Engine::load(paths).context("loading settings")?;
    let json = cli.json;

    match cli.cmd {
        Cmd::Probe => {
            let hw = engine.hardware(true).clone();
            let summary = engine.hw_summary();
            print(json, &(&hw, &summary), || {
                println!("OS        {}", hw.os);
                println!(
                    "CPU       {} ({} cores / {} threads, {} MHz, AVX2: {})",
                    hw.cpu.brand,
                    hw.cpu.physical_cores,
                    hw.cpu.logical_cores,
                    hw.cpu.max_mhz,
                    hw.cpu.avx2
                );
                println!("RAM       {} MB", hw.ram_mb);
                if hw.gpus.is_empty() {
                    println!("GPU       none detected through Vulkan");
                }
                for g in &hw.gpus {
                    println!(
                        "GPU       {} ({:?}, {}, {} MB, Vulkan {})",
                        g.name, g.kind, g.vendor, g.vram_mb, g.vulkan
                    );
                }
                if let Some(b) = &hw.bench {
                    println!(
                        "CPU test  score {} (best pass {:.0} ms)",
                        b.score, b.best_ms
                    );
                }
            });
        }
        Cmd::AddRoms { dir } => {
            let dir = dir
                .canonicalize()
                .with_context(|| format!("{}", dir.display()))?;
            if !engine.config.rom_dirs.contains(&dir) {
                engine.config.rom_dirs.push(dir.clone());
                engine.save_config()?;
            }
            println!("Game folders: {:?}", engine.config.rom_dirs);
        }
        Cmd::Scan => {
            let games = engine.games();
            let systems = engine.systems(&games);
            print(json, &(&systems, &games), || {
                for s in &systems {
                    println!(
                        "{:<28} {:>4} games   emulator: {} {}",
                        s.info.name,
                        s.games,
                        s.emulator,
                        if s.emulator_found {
                            "(found)"
                        } else {
                            "(NOT FOUND)"
                        }
                    );
                    for g in games.iter().filter(|g| g.system == s.info.id) {
                        println!(
                            "    {} {}{}",
                            g.id,
                            g.title,
                            if g.favorite { " ★" } else { "" }
                        );
                    }
                }
                if systems.is_empty() {
                    println!("No game found. Add a folder with: clctl add-roms <folder>");
                }
            });
        }
        Cmd::Plan { query } => {
            let game = find_game(&engine, &query)?;
            let (p, t) = engine.prepare(&game)?;
            print(json, &(&p, &t), || {
                println!("Game      {} [{}]", p.game.title, p.game.system);
                println!("Emulator  {}", p.emulator);
                println!(
                    "Quality   {} (rung {} of {:?}){}",
                    p.step_label,
                    p.step,
                    p.ladder,
                    if t.state.done {
                        ", settled"
                    } else {
                        ", still tuning"
                    }
                );
                println!("Measure   {}", if p.measure { "yes" } else { "no" });
                println!(
                    "Command   {} {}",
                    p.plan.program.display(),
                    p.plan.args.join(" ")
                );
                for n in &p.notes {
                    println!("Note      {n}");
                }
            });
        }
        Cmd::Play { query } => {
            let game = find_game(&engine, &query)?;
            let r = engine.play(&game)?;
            print(json, &r, || {
                println!(
                    "Played {} s, exit code {:?}",
                    r.session.played_seconds, r.session.exit_code
                );
                if let (Some(s), Some(v)) = (&r.stats, &r.verdict) {
                    println!(
                        "Measured {:.1} fps avg, {:.1} fps 1% low, {:.2} % stutter → {:?}",
                        s.avg_fps, s.low1_fps, s.stutter_pct, v
                    );
                }
                println!(
                    "Next launch: {}{}",
                    r.next_label,
                    if r.tuning_done {
                        " (settled)"
                    } else {
                        " (still tuning)"
                    }
                );
                for n in &r.notes {
                    println!("Note: {n}");
                }
            });
        }
        Cmd::Analyse { file, game } => {
            let ft = cl_perf::frametime::parse_file(&file)?;
            let stats = cl_perf::frametime::analyse(&ft)?;
            let v = cl_perf::frametime::verdict(&stats);
            print(json, &(&stats, v), || {
                println!("{} frames over {:.1} s", stats.frames, stats.seconds);
                println!(
                    "Average {:.1} fps, 1% low {:.1} fps (target {:.0})",
                    stats.avg_fps, stats.low1_fps, stats.target_fps
                );
                println!(
                    "Stutters {} ({:.2} %) → {:?}",
                    stats.stutters, stats.stutter_pct, v
                );
            });
            if let Some(q) = game {
                let g = find_game(&engine, &q)?;
                let t = engine.record_capture(&g, stats)?;
                println!(
                    "Profile updated: next rung {} ({})",
                    t.state.current,
                    if t.state.done {
                        "settled"
                    } else {
                        "still tuning"
                    }
                );
            }
        }
        Cmd::SetStep { query, step } => {
            let game = find_game(&engine, &query)?;
            let t = engine.set_step(&game, step)?;
            println!("{}: rung {} fixed by the user", game.title, t.state.current);
        }
        Cmd::Emulators => {
            for a in engine.adapters() {
                let (_, cfg, exe) = engine.find_emulator(a.id())?;
                let m = ManagedEmulator::new(&engine.paths.emulators_dir(), a.id());
                let st = m.state();
                println!("{} ({}, {})", a.name(), a.license(), a.homepage());
                println!(
                    "    mode {:?}, updates {:?}{}",
                    cfg.mode,
                    cfg.updates,
                    cfg.pinned
                        .as_ref()
                        .map(|p| format!(", pinned {p}"))
                        .unwrap_or_default()
                );
                println!(
                    "    executable: {}",
                    exe.map(|p| p.display().to_string())
                        .unwrap_or_else(|| "not found".into())
                );
                if st.active.is_some() {
                    println!(
                        "    managed versions: {:?} (active {:?})",
                        st.installed, st.active
                    );
                }
                println!(
                    "    systems: {}",
                    a.systems()
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        Cmd::Update { id, check } => {
            let source =
                sources::source_for(&id).ok_or_else(|| anyhow!("no source known for '{id}'"))?;
            if let sources::Source::Manual { page } = source {
                println!("{id}: automatic download not available yet. Official page: {page}");
                return Ok(());
            }
            let release = sources::latest(&source)?;
            let m = ManagedEmulator::new(&engine.paths.emulators_dir(), &id);
            let cfg = engine.config.emulator(&id);
            let decision = decide(
                &m.state(),
                &cfg,
                release.as_ref().map(|r| r.version.as_str()),
            );
            println!("{id}: {decision:?}");
            if let (Decision::Install { version }, false) = (&decision, check) {
                let rel = release.filter(|r| &r.version == version).ok_or_else(|| {
                    anyhow!("version {version} is not the latest release; download it from the official page")
                })?;
                println!(
                    "Downloading {} ({} MB)…",
                    rel.asset_name,
                    rel.size / 1_048_576
                );
                let file = sources::download(&rel, &m.root.join("downloads"))?;
                let adapter = engine
                    .adapter(&id)
                    .ok_or_else(|| anyhow!("unknown emulator"))?;
                let exe_name = adapter.exe_names().into_iter().next().unwrap_or_default();
                m.install_archive(version, &file, &exe_name)?;
                let _ = std::fs::remove_file(file);
                println!("Installed and active: {version}. Previous version kept for rollback.");
                if cfg.mode != cl_core::config::EmulatorMode::Managed {
                    println!("Set [emulators.{id}] mode = \"managed\" in config.toml to use it.");
                }
            }
        }
        Cmd::Rollback { id } => {
            let m = ManagedEmulator::new(&engine.paths.emulators_dir(), &id);
            println!("{id}: version {} is active again", m.rollback()?);
        }
    }
    Ok(())
}
