# App icon

A sun over two peaks on a blue rounded tile. Original geometry written by hand as SVG (three
shapes); every other file here is rendered from it.

## Palette

| Colour | Hex | Used for |
|---|---|---|
| Blue (app colour) | `#2f7bf5` | the tile |
| Sun | `#ffd25a` | the sun |
| White | `#ffffff` | the peaks |

## Geometry

A 512-unit tile (`viewBox="0 0 512 512"`), rounded square with `rx=112`, no border. The macOS
renders pad it onto Apple's 824/1024 icon grid; the Windows and Linux renders use the whole tile.

## Files

- `photocraft.svg`: canonical master (also copied to `photocraft-small.svg` and the hicolor
  scalable icon).
- `photocraft-1024.png`: 1024 px render on the macOS grid.
- `photocraft.icns`: macOS bundle icon (`CFBundleIconFile`).
- `photocraft.ico`: Windows icon, 16–256 px, embedded in the `.exe` by `apps/photocraft/build.rs`.
- `hicolor/<size>/apps/ai.storyteller.photocraft.png` (16–512) and `hicolor/scalable/...svg`:
  Linux icon theme. (The file names are machine ids, not shown to users.)

## Regenerate

Edit `photocraft.svg`, then run `packaging/icons.sh` (needs `resvg`, plus `iconutil` on macOS;
the `.ico` is packed by `cargo xtask ico`).
