# Character packs

Latch is the default pack. Packs change the player, its animation, the four
pursuers, their vulnerable artwork, and the icon in the native window/About.
They do not change collision boxes, movement, scores, difficulty or saved runs.
The installed app launcher keeps the Latch icon so Scram is easy to find.

## Make a replacement without rebuilding

1. Open Settings with **Ctrl+,**. Under Characters, choose **Create editable copy**.
2. Choose **Open packs folder**. Open the new `my-characters` folder.
3. Replace `player.png`, `pursuers.png` or `icon.png`. If you keep the same
   canvas sizes and positions, you can leave the frame rectangles unchanged.
4. Edit `name` in `pack.toml` to distinguish your pack. Set its author and licence.
5. Choose **Reload packs**, then choose your pack in the Characters selector.
   Subsequent edits appear when you press Reload packs again.

Your run stays paused while Settings is open. Close Settings and press Space
to resume. The chosen pack is remembered between launches. You can switch back
to Latch or Original at any point.

Each export creates a fresh folder; it never replaces an earlier editable copy.
Create editable copy always starts from Latch. To fork another custom pack,
copy its entire folder in your file manager and change its name in `pack.toml`.

## Install a shared pack

Put the folder containing `pack.toml` directly inside:

```text
~/.local/share/omarchy-munch/packs/my-pack/
```

When `XDG_DATA_HOME` is set to an absolute path, the base is
`$XDG_DATA_HOME/omarchy-munch/packs` instead. Folder names accept letters,
numbers, hyphens and underscores, up to 64 characters. The selector uses the
name from the manifest. It scans up to 128 directory entries; symlink folders
are skipped. No network downloads or scripts are run by the loader.

The `omarchy-munch` data directory is intentionally retained from the earlier
preview so existing packs keep working after the Scram rename.

Click Reload packs after installing. If a reload fails, the previous artwork
stays visible and the error is shown in Settings. If a saved pack is missing or
invalid on launch, Scram selects Latch and displays a notice. Fix or reinstall
the custom pack, reload and select it again.

## Files and layout

| File | Latch canvas | Contents |
| --- | --- | --- |
| `pack.toml` | UTF-8 TOML | Names, frame rectangles and state mapping |
| `player.png` | 1536 × 1024 | Closed, half-open and open poses, facing right |
| `pursuers.png` | 1536 × 1024 | Four normal poses above four vulnerable poses |
| `icon.png` | 1254 × 1254 | Opaque charcoal launcher tile |

The renderer extracts square rectangles at load/render time; the PNGs are
unchanged. Any canvas up to 2048 × 2048 is supported, and atlases need not use a
uniform grid. Latch's rectangles are supplied in its manifest.

Here is a complete example for a simple transparent atlas: a 256 × 64 player
sheet with four poses and a 256 × 128 pursuer sheet with two rows of four poses.

```toml
schema = 1
name = "My characters"
author = "Your name"
license = "Your chosen licence"
icon = "icon.png"
fps = 9.0
scale = 1.0
normal = [0, 1, 2, 3]
vulnerable = [4, 5, 6, 7]

[player]
image = "player.png"
frames = [[0, 0, 64, 64], [64, 0, 64, 64], [128, 0, 64, 64], [192, 0, 64, 64]]

[pursuers]
image = "pursuers.png"
frames = [
  [0, 0, 64, 64], [64, 0, 64, 64], [128, 0, 64, 64], [192, 0, 64, 64],
  [0, 64, 64, 64], [64, 64, 64, 64], [128, 64, 64, 64], [192, 64, 64, 64],
]
```

A frame is `[left, top, width, height]` in pixels, starting at the image's top
left. Width and height must be equal and the whole frame must fit the image.
Leave a little transparent padding. Centre each character consistently to
avoid wobble. The player faces right and rotates with movement. Pursuers stay
upright. Indices are zero-based: the normal and vulnerable arrays map the four
AI identities to atlas frames. Returning pursuers use their vulnerable frame
at 58% size. No extra animation or engine hooks are required.

Player frames play in the listed order; repeat a half-open frame for a smooth
closed/half/open/half cycle. Reduce animation uses the first player frame.
Pursuers use one frame per state. Draw eyes and vulnerable expressions boldly:
characters usually occupy only 14–30 pixels on the playfield.

## Transparency and limits

True RGBA PNGs are recommended for new artwork. Omit `key_color` for these.
Latch was generated with an approximate magenta backdrop. Its manifest adds
these fields inside both `[player]` and `[pursuers]`:

```toml
key_color = "#ff00ff"
key_tolerance = 70
```

At texture load, pixels within the tolerance in every RGB channel are treated
as transparent. Existing PNG alpha is preserved elsewhere. This does not edit
the source file. Avoid that colour in the character itself. Lower tolerance
suits exact backgrounds; omit the key entirely when replacing with RGBA art.
A painted checkerboard is not transparency and is not automatically removed.

Limits: 8 MiB per PNG; 2048 pixels per side; 32 KiB manifest; 1–12 player frames;
1–32 pursuer frames; 1–24 fps; scale 0.6–1.2; key tolerance 0–100. Names, authors
and licence labels must be nonempty, at most 100 bytes, with no control
characters. The icon must be square. Fully invisible actor frames are rejected.
Assets must be relative paths inside the pack; parent traversal and escaping
symlinks are rejected. Unknown manifest fields and schema versions are rejected
so typos cannot silently change rendering.

## Check from a terminal

These commands do not open a window or touch game saves:

```sh
omarchy-scram --validate-pack ~/.local/share/omarchy-munch/packs/my-characters
omarchy-scram --export-pack ~/Downloads
```

The first prints the validated pack name and frame counts, or exits with an
error. The second creates an editable Latch folder inside the destination and
prints its path. The game runtime and pack tools are native Rust.
