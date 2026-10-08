# Pixel asset editor

## Run the editor

Run this command from the repository root:

```sh
cargo run -p egui-editor
```

## Asset types

The editor supports these asset types:

- Static sprites (`.pxs`)
- Animated sprites (`.pxa`)
- Bitmap fonts (`.pxf`)
- Tilesets (`.pxt`)
- Tilemaps (`.pxm`)

## File operations

Use the File menu to create, open, save, or close an asset.

Open and Save As use system file dialogs. Save replaces the open file.

## Main window

The editor has a left panel, a central canvas, and a status bar.

The left panel contains the tools and view controls. Animated sprites also show a timeline at the bottom.

The editor does not have a right tool panel.

## Canvas controls

Use the mouse wheel to zoom the canvas.

Drag with the middle mouse button to pan the canvas. You can also hold Space and drag with the left mouse button.

Use the View controls to fit the canvas in the view. Use the same controls to select a light or dark grid.

Choose **Draw** to paint. Left-click to use the selected color. Right-click to erase.

Choose **Select**. Drag on the canvas to select a rectangle. Drag inside it to move it. The editor keeps the selection inside the sprite. Press `Esc` or click **Deselect** to clear it.

## Palette

The palette groups colors into 16-color bands. Each band groups similar hues. Each band has muted and vivid shade ramps.

On a color canvas, move the pointer over a pixel and press `P` to select its color. The editor does not change the pixel.

Press `P` over a transparent pixel to select Transparent. Paint with Transparent to clear pixels.

Hover over a palette color to see its name and hex code.

## Animation controls

Click a frame in the timeline to select it.

Drag a tag handle to change its frame range. Use the Animation tags panel to change a tag name or direction.

## Tilemaps

A tilemap uses a separate tileset. Both assets must have the same UUID.

The tilemap editor has Tilemap and Tileset tabs. Use the Tileset tab to edit the linked tiles.

The left panel shows tile previews in the Tilemap tab. The strip at the bottom shows all tiles.

Click a tile in the strip to select it. Use the Add tile, Duplicate tile, and Remove tile buttons to change the tileset.

Save writes the map and any changed tileset.

## API

The editor uses document types from `api::formats`.

The `dev` feature provides pixel editing and encoding functions for the editor.
