# Decisions

- **Scope:** a playable native maze chase for Tom's retro Omarchy Arcade.
  Name: Omarchy Scram (original working name: Omarchy Munch). The first milestone ships one original maze,
  four deterministic pursuers, power, scoring, lives, progression and recovery.
- **Stack:** Rust 2021, eframe/egui 0.31.1, matching the collection's current
  native implementation. No browser or scripting runtime in the app.
- **Artwork:** original geometric player, pursuer crowns, maze layout and
  launcher icon. Omarchy character comes from the palette and Arcade frame.
  No official brand asset or commercial resource is bundled.
- **Visual direction:** a restrained desktop arcade cabinet: charcoal playing
  field, sage circuit walls, warm player, monospace counters, minimal chrome.
  The functional signature is the power countdown directly beneath the maze,
  which replaces flashing vulnerability warnings with precise information.
- **Input:** buffered junction turns and immediate reversal. Fixed 120 Hz
  simulation; rendering is independent of the rules. No catch-up beyond 100 ms
  after a stalled render frame. Focus loss and menu use pause the run.
- **Pursuit:** scatter/chase phases plus chase, look-ahead, flank and retreat
  targets. Returning pursuers navigate to the centre and wait before release.
  These are original rules, not an emulation of the commercial game's ROM.
- **Storage:** one versioned snapshot, atomic updates, exclusive process lock,
  previous-run archive, damaged-save retention, read-only future-version mode.
  Relaxed and standard records are separate.
- **Audio:** short original synthesized events, off by default, asynchronous
  playback through paplay. No per-dot process spawn or background music.
- **Delivery:** a Linux x86_64 preview and complete source with a PKGBUILD.
  The initial downloads preceded repository creation. Tom subsequently supplied
  `tcballard/omarchy-scram` as the canonical repository.
- **Remaining acceptance:** real Omarchy Wayland focus/scaling, native audio,
  pacman build/install/upgrade and play-feel tuning with Tom.

- **0.2 character direction:** Tom selected concept 1, Latch, and asked to make
  character replacement easy. The default uses original Image Generation
  artwork: a hinged ivory jaw, four mechanical pursuers and a matching icon.
  The earlier vector artwork remains selectable as Original.
- **Pack boundary:** schema-1 TOML plus PNGs; only presentation data, no scripts,
  rules or executable hooks. Save schema remains 1 with a defaulted pack key.
  Selection and reload never replace the Game object or records. Errors keep
  the current art; unavailable saved packs fall back to Latch with a notice.
- **Raster transparency:** Image Generation returned RGB checkerboard outputs
  despite alpha requests. A subsequent tool edit supplied a near-magenta
  backdrop. The source PNGs are preserved unchanged; the renderer interprets
  the manifest's magenta colour key with tolerance 70 at texture load. Packs
  may instead supply true RGBA PNGs and omit the key. The launcher is opaque.
- **Artist workflow:** Settings exports a new editable Latch folder without
  replacing prior edits. Folder discovery and reload need no compiler or app
  restart. A CLI validator reports asset and manifest errors without a GUI.
- **Icon identity:** the installed launcher stays Latch for a stable app
  identity. A selected pack's icon is used in the native window and About.

- **0.2.1 name:** Tom selected Scram after rejecting Munch and Latch as game
  names. The game title, native window, launcher, executable, crate, package
  and documentation now use Omarchy Scram. Latch remains an artwork pack.
- **Rename compatibility:** retain the original `omarchy-munch` state and
  pack directories, including the shared writer lock. The portable installer
  replaces the previous launcher and leaves an `omarchy-munch` command alias.
  The Arch package provides/replaces/conflicts with the prior package name.
  Saved sessions and user artwork require no migration or edits.

- **Repository import:** the supplied repository was empty. A README-only
  seed establishes the PR base; the native game, assets, packaging and checks
  are introduced on `feat/native-scram` for review. No release is tagged here.
