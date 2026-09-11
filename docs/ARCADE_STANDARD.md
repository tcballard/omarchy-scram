# Omarchy Arcade conventions applied

Source: the collection standard in
[Omarchy Chess](https://github.com/tcballard/omarchy-chess/blob/feat/native-chess-preview/docs/ARCADE_STANDARD.md).

- Native Rust app, immediate playable field or paused saved run.
- Game, Settings and Help menus. Ctrl+N, Ctrl+comma, Ctrl+M, F1 and Ctrl+Q.
- Sound off initially; saved explicit choice and original cues.
- Follow Omarchy by default, readable fallback, light/dark support, no theme writes.
- Launcher icon in the 128 × 128 desktop slot. The user-selected Latch artwork
  uses a charcoal frame, sage hinge and ivory jaw; it supersedes the initial
  geometric family icon. The high-resolution PNG is scaled by the desktop.
- About: matching icon, name, Omarchy Arcade, version, credits and community status.
- Per-game atomic local state and settings, preserved during package updates.
- Stable executable, desktop entry and icon identities.
- Game-over result and Play Again; retain the preceding run before replacing it.

Scram adds buffered movement, window-focus pause, explicit resume, a non-flashing
power timer and separate relaxed-mode records. Undo does not apply to real-time
maze play.
