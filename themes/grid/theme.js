// Grid — minimal template: the smallest useful theme, ~80 lines.
const $ = (id) => document.getElementById(id);
const esc = Launcher.escapeHtml;
const fr = (navigator.language || "fr").toLowerCase().startsWith("fr");
const EMPTY = fr ? "Aucun jeu. Menu (Tab / Start) → ajouter un dossier." : "No games. Menu (Tab / Start) → add a folder.";

let systems = [], bySystem = {}, sys = 0, sel = 0;

async function load() {
  const lib = await Launcher.getLibrary();
  systems = lib.systems;
  bySystem = {};
  for (const g of lib.games) (bySystem[g.system] ||= []).push(g);
  render();
}

const games = () => (systems[sys] ? bySystem[systems[sys].info.id] || [] : []);
const columns = () => getComputedStyle($("gr-grid")).gridTemplateColumns.split(" ").length;

function render() {
  $("gr-tabs").innerHTML = systems.map((s, i) => `<span class="gr-tab ${i === sys ? "current" : ""}">${esc(s.info.name)}</span>`).join("");
  const list = games();
  sel = Math.min(sel, Math.max(0, list.length - 1));
  // Keep the selected row visible: show 3 rows around it.
  const cols = columns();
  const firstRow = Math.max(0, Math.floor(sel / cols) - 1);
  const shown = list.slice(firstRow * cols, firstRow * cols + cols * 3);
  $("gr-grid").innerHTML = list.length
    ? shown.map((g, k) => {
        const i = firstRow * cols + k;
        const bg = g.coverUrl ? `style="background-image:url('${g.coverUrl}')"` : "";
        return `<div class="gr-card ${g.coverUrl ? "has-cover" : ""} ${i === sel ? "current" : ""}" ${bg}><span>${esc(g.title)}</span>${g.favorite ? '<span class="star">★</span>' : ""}</div>`;
      }).join("")
    : `<div class="gr-empty">${EMPTY}</div>`;
  const g = list[sel];
  $("gr-bar").innerHTML = g ? `<span><b>${esc(g.title)}</b> · ${esc(systems[sys].info.name)}</span><span id="gr-q"></span>` : "";
  if (g) Launcher.gameInfo(g.id).then((i) => { const q = $("gr-q"); if (q) q.textContent = i.ready ? `${i.emulator} · ${i.stepLabel}` : i.error; });
}

function move(d) {
  const n = games().length;
  if (n) { sel = Math.max(0, Math.min(n - 1, sel + d)); render(); }
}

function switchSystem(d) {
  if (!systems.length) return;
  sys = (sys + d + systems.length) % systems.length;
  sel = 0;
  render();
}

Launcher.onNav(async (a) => {
  if (a === "left") move(-1);
  else if (a === "right") move(1);
  else if (a === "up") move(-columns());
  else if (a === "down") move(columns());
  else if (a === "pageLeft") switchSystem(-1);
  else if (a === "pageRight") switchSystem(1);
  else if (a === "confirm" && games()[sel]) {
    const info = await Launcher.gameInfo(games()[sel].id).catch(() => null);
    if (info && !info.ready) Launcher.openSettings(info.emulatorId ? { emulator: info.emulatorId } : { page: "systems" });
    else await Launcher.play(games()[sel].id).catch(console.error);
  }
  else if (a === "favorite" && games()[sel]) { games()[sel].favorite = await Launcher.toggleFavorite(games()[sel].id); render(); }
  else if (a === "menu" || a === "back") Launcher.openSettings();
});

document.addEventListener("launcher:library-changed", load);
window.addEventListener("resize", render);
load();
