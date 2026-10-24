# Metadata, artwork and patches

## Goal

The library identifies a local dump independently of its filename, keeps rich
details available offline after the first sync, and provides a precise identity
for per-game optimisation and patches. No remote provider is required to launch
a game.

## Identification

Confidence order: a product/serial id read from the game, a content checksum for
small cartridge images, then a normalised filename only as a suggestion. A
compressed container needs a format-aware reader; its container checksum must
not be presented as the game's checksum. Platform, region and revision remain
part of the identity because patches can be release-specific.

The engine now exposes `GameIdentity` (`serial`, `crc32`, size) and the offline
`metadata/catalogue.json` cache. Raw GameCube/Wii ISO or GCM images, Nintendo DS
files and small cartridge images form the first supported set.

## Providers

Provider adapters merge into the common `GameMetadata` model:

- **Title Screen** is the first connected provider. Its public, keyless API can
  search by checksum, serial or title and returns releases, dates, studios,
  publishers, regions and genres. Results are cached locally. Title Screen data
  is licensed under CC BY 4.0;
- **Libretro Database** for local matching and normalised retro data, first as
  a versioned data pack;
- **GameTDB** for GameCube/Wii, Nintendo DS and Wii U ids, localised details and artwork, with required
  attribution and project contact before distribution;
- **Libretro Thumbnails** for on-demand cached box art, logos and screenshots;
- **IGDB** as a rich optional supplement through a relay service or
  user-supplied credentials—never an embedded application secret;
- **ScreenScraper** as an optional credentialled and rate-limited provider.

Every field retains provenance, remote id, refresh time and confidence. User
corrections always win and are never overwritten.

## Fetching and cache

Sync is explicit, cancellable and resumable. Responses and images use a bounded
cache; network failures preserve the last valid record; artwork loads in the
background; secrets never enter logs or `config.toml`; and a Sources and
Licences page credits every provider.

A selected game without cached details automatically starts one background
lookup after a short delay; **Refresh/Retry details** can run it again manually.
Startup never bulk-downloads the library. For GameCube/Wii, Nintendo DS and Wii U
the lookup fetches the Title Screen record, then the GameTDB front cover matching
the game id and regional edition. The PNG or JPEG is validated, limited to 10 MB and kept in
`metadata/artwork/<system>/`. Scanning, startup and game launch remain fully
local, and Title Screen and GameTDB remain separate sources in the cache. The
interface never renders the remote URL directly: artwork must be validated and
stored locally first. A failed cache operation is shown as a retryable state.

## Patches

A patch targets an exact platform, serial/checksum, region and revision—not a
title alone. Its manifest records source, author, licence, kind (60 FPS,
widescreen, translation, textures), tested emulator versions, hardware cost,
conflicts, rollback and payload checksum.

Official emulator compatibility settings precede the quality ladder. Behaviour
changing patches such as 60 FPS remain opt-in. Patched and unpatched builds keep
separate measured profiles.

## Optimisation link

The identity key joins metadata, artwork, compatibility constraints, available
patches and measured performance—including emulator, driver and patch versions.
Hardware scoring only selects the first rung; real stability measurements choose
the final configuration for that exact game identity.
