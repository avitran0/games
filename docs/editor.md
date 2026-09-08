# Pixel asset editor

Run `cargo run -p egui-editor` from the repository root.

The editor supports these asset types:

- Static sprites (`.pxs`)
- Animated sprites (`.pxa`)
- Bitmap fonts (`.pxf`)
- Tilesets (`.pxt`)
- Tilemaps (`.pxm`)

Use the File menu to create, open, save, or close an asset. Open and Save As use native file dialogs. Save replaces the open file.

The editor uses a compact left panel, a central canvas, and a status bar. Animated sprites also show a frame timeline at the bottom. The editor has no right tool panel.

Use the mouse wheel to zoom. Use middle-drag or Space and left-drag to pan. Use the View controls to fit the canvas or select a light or dark grid. Left click paints with the selected color. Right click erases.

Click a frame in the animation timeline to select it. Drag a tag handle to change its frame range. Use the Animation tags panel to edit a tag name or direction.

Tilemaps use a separate tileset. The tilemap and tileset must have the same UUID. The tilemap editor has Tilemap and Tileset tabs. Use the Tileset tab to edit its linked tiles. The left panel shows tile previews in the Tilemap tab. A tile strip at the bottom shows every tile. Click a tile to select it. Use the Add tile, Duplicate tile, and Remove tile buttons beside the strip heading to change the tileset. Save writes the map and any changed tileset.

The editor uses document types from `api::formats`. The `dev` feature provides pixel editing and encoding functions for editor use.
