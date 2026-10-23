// Horizon — default template.
// The SDK gives this script one object, `Launcher` (see THEMES docs).
// Structure: state → render functions → navigation. Copy and adapt freely.

const fr = (navigator.language || "fr").toLowerCase().startsWith("fr");
const T = fr
  ? { games: "jeux", play: "Jouer", back: "Retour", fav: "Favori", menu: "Menu", noGames: "Aucun jeu trouvé.<br>Ouvre le menu (Tab / Start) pour ajouter un dossier de jeux.", missing: "émulateur introuvable", running: "Partie en cours…", runningHint: "Le lanceur revient quand l'émulateur se ferme.", measured: "Mesure de fluidité", settled: "Réglage trouvé", tuning: "Optimisation en cours", slow: "Machine en dessous du nécessaire", fix: "A : choisir l'émulateur", systemOnly: "Optimisations système" }
  : { games: "games", play: "Play", back: "Back", fav: "Favourite", menu: "Menu", noGames: "No game found.<br>Open the menu (Tab / Start) to add a game folder.", missing: "emulator not found", running: "Game running…", runningHint: "The launcher comes back when the emulator closes.", measured: "Smoothness check", settled: "Settings found", tuning: "Optimising", slow: "Machine below requirements", fix: "A: choose the emulator", systemOnly: "System optimisations" };

const $ = (id) => document.getElementById(id);
const esc = Launcher.escapeHtml;
const MAKER = { Nintendo: "var(--nintendo)", Sony: "var(--sony)", Sega: "var(--sega)" };

const state = {
  systems: [],
  bySystem: {},
  sys: 0,
  game: {}, // selected game index per system id
  focus: "systems",
  infoToken: 0,
  info: null, // gameInfo of the selected game
};

async function loadLibrary() {
  const lib = await Launcher.getLibrary();
  state.systems = lib.systems;
  state.bySystem = {};
  for (const g of lib.games) (state.bySystem[g.system] ||= []).push(g);
  state.sys = Math.min(state.sys, Math.max(0, state.systems.length - 1));
  render();
}

const currentSystem = () => state.systems[state.sys];
const currentGames = () => (currentSystem() ? state.bySystem[currentSystem().info.id] || [] : []);
const currentGame = () => currentGames()[state.game[currentSystem()?.info.id] || 0];

function renderSystems() {
  const nav = $("hz-systems");
  nav.innerHTML = state.systems
    .map((s, i) => `
      <div class="hz-sys ${i === state.sys ? "current" : ""}" style="--sys:${MAKER[s.info.manufacturer] || "var(--accent)"}">
        <div class="bar"></div>
        <div class="name">${esc(s.info.name)}</div>
        <div class="meta">${s.games} ${T.games} · ${s.emulator_found ? esc(s.emulator_name) : `<span class="warn">${T.missing}</span>`}</div>
      </div>`)
    .join("");
  const cur = nav.children[state.sys];
  if (cur) cur.scrollIntoView({ inline: "center", block: "nearest" });
  const sys = currentSystem();
  document.querySelector(".hz").style.setProperty("--sys", sys ? MAKER[sys.info.manufacturer] || "var(--accent)" : "var(--accent)");
}

function renderGames() {
  const list = $("hz-games");
  const games = currentGames();
  if (!state.systems.length) {
    list.innerHTML = `<li class="hz-empty">${T.noGames}</li>`;
    return;
  }
  const sel = state.game[currentSystem().info.id] || 0;
  // Show a window of items around the selection so long lists stay fast.
  const size = 11;
  const start = Math.max(0, Math.min(sel - Math.floor(size / 2), games.length - size));
  list.innerHTML = games
    .slice(start, start + size)
    .map((g, k) => `<li class="hz-game ${start + k === sel ? "current" : ""}">${esc(g.title)}${g.favorite ? '<span class="fav">★</span>' : ""}</li>`)
    .join("");
}

function initials(title) {
  return title.split(/\s+/).filter(Boolean).slice(0, 2).map((w) => w[0]).join("").toUpperCase();
}

async function renderDetail() {
  const box = $("hz-detail");
  const g = currentGame();
  if (!g) { box.innerHTML = ""; return; }
  const cover = g.coverUrl
    ? `<div class="hz-cover" style="background-image:url('${g.coverUrl}')"></div>`
    : `<div class="hz-cover placeholder"><span class="initials">${esc(initials(g.title))}</span></div>`;
  box.innerHTML = `${cover}<div class="hz-title">${esc(g.title)}</div><div class="hz-chips" id="hz-chips"></div><ul class="hz-notes" id="hz-notes"></ul>`;

  // Emulator and quality info: fetched after a short pause so scrolling stays smooth.
  state.info = null;
  const token = ++state.infoToken;
  await new Promise((r) => setTimeout(r, 140));
  if (token !== state.infoToken) return;
  const info = await Launcher.gameInfo(g.id).catch((e) => ({ ready: false, error: String(e) }));
  if (token !== state.infoToken) return;
  state.info = info;
  const chips = $("hz-chips");
  const notes = $("hz-notes");
  if (!info.ready) {
    chips.innerHTML = `<span class="hz-chip warn">${esc(info.error || T.missing)}</span>` +
      (info.code === "emulator_missing" || info.code === "no_emulator" ? `<span class="hz-chip">${T.fix}</span>` : "");
    return;
  }
  chips.innerHTML = [
    `<span class="hz-chip">${esc(info.emulator)}</span>`,
    info.tunesSettings ? `<span class="hz-chip accent">${esc(info.stepLabel)}</span>` : `<span class="hz-chip">${T.systemOnly}</span>`,
    info.belowRequirements
      ? `<span class="hz-chip warn">${T.slow}</span>`
      : `<span class="hz-chip">${info.settled ? T.settled : T.tuning}</span>`,
    info.measure ? `<span class="hz-chip">${T.measured}</span>` : "",
  ].join("");
  notes.innerHTML = (info.notes || []).map((n) => `<li>${esc(n)}</li>`).join("");
}

function renderHelp() {
  $("hz-help").innerHTML = `
    <span><kbd>A</kbd>${T.play}</span>
    <span><kbd>B</kbd>${T.back}</span>
    <span><kbd>Y</kbd>${T.fav}</span>
    <span><kbd>Start</kbd>${T.menu}</span>`;
}

function render() {
  document.querySelector(".hz").className = `hz focus-${state.focus}`;
  renderSystems();
  renderGames();
  renderDetail();
}

function moveSystem(d) {
  if (!state.systems.length) return;
  state.sys = (state.sys + d + state.systems.length) % state.systems.length;
  render();
}

function moveGame(d) {
  const games = currentGames();
  if (!games.length) return;
  const id = currentSystem().info.id;
  const cur = state.game[id] || 0;
  const next = cur + d;
  if (next < 0 && d === -1) { state.focus = "systems"; render(); return; }
  state.game[id] = Math.max(0, Math.min(games.length - 1, next));
  render();
}

async function play() {
  const g = currentGame();
  if (!g) return;
  // Emulator missing: go straight to the place where it is chosen.
  if (state.info && !state.info.ready) {
    return Launcher.openSettings(state.info.emulatorId ? { emulator: state.info.emulatorId } : { page: "systems" });
  }
  try {
    await Launcher.play(g.id);
  } catch (e) {
    console.error(e);
  }
  renderDetail();
}

Launcher.onNav(async (a) => {
  switch (a) {
    case "left": return moveSystem(-1);
    case "right": return moveSystem(1);
    case "down":
      if (state.focus === "systems") { state.focus = "games"; return render(); }
      return moveGame(1);
    case "up": return state.focus === "games" ? moveGame(-1) : undefined;
    case "pageLeft": return state.focus === "games" ? moveGame(-10) : moveSystem(-1);
    case "pageRight": return state.focus === "games" ? moveGame(10) : moveSystem(1);
    case "confirm":
      if (state.focus === "systems") { state.focus = "games"; return render(); }
      return play();
    case "back":
      if (state.focus === "games") { state.focus = "systems"; return render(); }
      return;
    case "favorite": {
      const g = currentGame();
      if (g) { g.favorite = await Launcher.toggleFavorite(g.id); renderGames(); }
      return;
    }
    case "menu": return Launcher.openSettings();
  }
});

Launcher.onSession((s) => {
  const box = $("hz-session");
  box.hidden = s.state !== "started";
  box.innerHTML = `${T.running}<small>${T.runningHint}</small>`;
});

Launcher.onPad((p) => { $("hz-pad").textContent = p.connected ? `🎮 ${p.name}` : ""; });

document.addEventListener("launcher:library-changed", loadLibrary);

setInterval(() => {
  $("hz-clock").textContent = new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}, 1000);

renderHelp();
loadLibrary();
