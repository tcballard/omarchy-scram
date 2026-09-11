# Verification: 0.2.1 preview

Verified in a Linux x86_64 environment using Rust 1.98.1, an actual X11 native
window under Xvfb, and software OpenGL. The current update renames the game to Omarchy Scram. Latch artwork and
data-only character packs remain; gameplay rules are unchanged.

## Scram rename and upgrade

The optimized Scram executable opens a native window under its new name.
All seven screenshots were refreshed and the gameplay view inspected.
The portable installer was run over a 0.2.0 Munch installation in a path with
spaces. It leaves one Scram launcher, replaces the icon identity, and keeps
`omarchy-munch` as a working command alias. GLib accepts the desktop entry.

A native launch using the default XDG state location preserves the complete
saved game, both records, settings and selected custom pack. The custom PNG
bytes are unchanged. Launching the old executable concurrently is refused by
the shared writer lock. Reinstallation also preserves the session.

`cargo fmt --check`, Clippy with denied warnings, 36 tests and the release build
pass under the renamed crate. Shell syntax checks pass for both installer and
PKGBUILD. Actual pacman package replacement remains an Arch desktop/CI gate.

## Automated rules and state checks

**36 tests pass.** The suite covers:

- Maze dimensions, reachable corridors and all four power diamonds.
- Bidirectional tunnels, wall stops, immediate reversal and buffered turns.
- Collecting each dot once, capture chains, power expiry and pursuer return.
- One life per collision, final death, level clear and extra-life thresholds.
- Invalid geometry, non-finite movement values and deterministic save continuation.
- 60,000 simulation steps with randomized direction inputs.
- Save round trips, prior-run archives, damaged-file preservation, unknown
  future schemas, failed restart recovery and exclusive writer locking.
- Separate relaxed-mode records, light/dark contrast and malformed theme fallback.
- PCM audio headers and sample lengths.
- Latch atlas bounds and colour-key interpretation; true PNG alpha preservation.
- Invalid schema, geometry, frame indices, invisible frames and missing assets.
- Parent traversal and symlink escape rejection, and non-destructive template export.
- Pack switching and failed loads preserving game state and both score records.
- Saves without a pack field and unavailable saved packs recovering to Latch.

`cargo fmt --check`, Clippy with warnings denied, locked tests and the optimized
release build pass. The binary's required glibc symbol versions reach 2.39.

## Native window evidence

Real XTest key and pointer events verify launching, moving left from the
starting position, scoring, Space pause/resume, focus-loss pause, menu pause,
closing a menu without resuming, a persisted sound setting and clean Ctrl+Q exit.
The final release executable passed these checks.

The restored-game check also verifies that Game Over remains visible after
relaunch and focus changes, and that Play Again starts a fresh run while keeping
the previous session. Completed games are not treated as paused active games. The current game-over render also uses the Scram name.

Rendered windows were inspected at 800 × 940 and 560 × 620. The initial check
caught score overlap at the small size, seams between player triangles, and
loss of hue in the light palette. These were corrected before the final
screenshots. The small Help dialog scrolls to expose its complete contents.

- `charcoal.png` and `ivory.png`: gameplay started by native keyboard events.
- `compact.png`: initial ready state at the minimum size.
- `help.png` and `settings.png`: dialogs opened using F1 and Ctrl+comma.
- `power.png`: a configured power-state fixture, then advanced by native input.

The portable user installer was checked for shell syntax, a prefix containing
spaces, desktop-entry parsing, native application launch, reinstallation and
saved-session preservation. The source PKGBUILD was checked for shell syntax. The 0.2 installer also
installs the new PNG and removes the prior app-owned SVG; installed launch,
reinstallation and save preservation pass with the updated bundle.

## Character-pack workflow

Native XTest pointer events verify Latch → Original → Latch, Create editable
copy, a manifest edit, Reload packs, and selection of the new custom pack.
The selected key is written to the save while the complete game state and
both score records stay identical. Removing a selected atlas and reloading
keeps the current artwork; relaunch then selects Latch and preserves the run.
The CLI validator also accepts the shipped pack without opening a window.

Visual inspection caught aliasing when large atlases were drawn at game size;
OpenGL mipmap filtering now smooths them. The selector initially retained a
popup height sized for two packs, hiding a newly exported third pack. Its
cache identity now changes with the number of packs, and the complete native
export/edit/reload/select check passes. All displayed PNG sources remain
unchanged; colour-key interpretation and mipmaps happen in the renderer.

The current screenshots show Latch at 800 × 940, both palettes, the minimum
560 × 620 window, Settings and the power state. Large source art inevitably
loses small facial details in compact mode; silhouettes and the exact power
timer remain the primary gameplay cues. The editable copy includes its guide
and licence. Real Omarchy launcher/compositor acceptance remains outstanding.

## Checks still requiring Omarchy

- A real Omarchy Wayland session: scaling, compositor focus behaviour, launcher
  appearance, live theme switching and extended play-feel testing.
- Actual audio playback on the desktop sound server. Only cue construction and
  nonblocking playback code were checked here.
- Building, installing and upgrading through makepkg/pacman on Arch.
- Screen-reader compatibility; no claim of screen-reader-only playability.

GitHub CI is configured for Rust and Arch package verification. The local
evidence above does not establish remote CI or Arch results; check the current
[Native checks run](https://github.com/tcballard/omarchy-scram/actions/workflows/ci.yml)
for the branch or commit being installed.
