# Pixel asset formats

## Common rules

The first 4 bytes are the file ID. The next 2 bytes give the file version. The current version is `1`.

All other integers use little-endian byte order. Store the least-significant byte first.

Files do not use compression or padding. Each file must end after its last field.

The file extension identifies the asset type. The decoder checks the file ID and version. It does not check the extension.

Image data uses row-major order. Start at the top-left pixel. Read each row from left to right, then read the next row.

## Open a file in ImHex

1. Open the asset file in ImHex.
2. Open the Pattern Editor.
3. Load the pattern for the asset type.
4. Run the pattern.

The patterns use ImHex standard libraries. Do not set an include path.

| Asset | Extension | File ID | Pattern |
| --- | --- | --- | --- |
| Static sprite | `.pxs` | `SPRT` | [sprite.hexpat](imhex/sprite.hexpat) |
| Animated sprite | `.pxa` | `ANIM` | [animated_sprite.hexpat](imhex/animated_sprite.hexpat) |
| Bitmap font | `.pxf` | `FONT` | [font.hexpat](imhex/font.hexpat) |
| Tileset | `.pxt` | `TSET` | [tileset.hexpat](imhex/tileset.hexpat) |
| Tilemap | `.pxm` | `TMAP` | [tilemap.hexpat](imhex/tilemap.hexpat) |

The patterns show the file fields. They check the ID, version, dimensions, and file length. They also check fields that do not need another asset file.

## Pixels and dimensions

Sprite and tile widths and heights must be multiples of 8. Each dimension must be 8 to 96 pixels.

Each sprite and tile pixel uses one byte:

- `0` means transparent.
- Values `1` through `255` select colors in the Aurora palette.

The file does not contain the palette. `api/src/formats/color.rs` defines the Aurora colors and their order. The palette uses 16-color bands. The first band contains grays. Each other band groups similar hues into two shade ramps. The first ramp contains muted colors. The second ramp contains vivid colors. Each ramp goes from dark to light. The last band has 15 colors. `assets/palette.pal` uses the same order for palette tools.

Font height must be 1 to 64 pixels. Each glyph has its own width. Glyph width must be greater than zero. Each glyph bitmap uses `width * height` bytes. Zero means clear. A nonzero value means set.

## Static sprite: `SPRT`

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `char[4]` | File ID: `SPRT` |
| 4 | `u16` | Version: `1` |
| 6 | `u16` | Width in pixels |
| 8 | `u16` | Height in pixels |
| 10 | `u8[width * height]` | Pixel data |

The file length is `10 + width * height` bytes. The file contains one image.

`SpriteDocument` stores the decoded image. `Assets::load_sprite` loads the file as a runtime sprite.

## Animated sprite: `ANIM`

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `char[4]` | File ID: `ANIM` |
| 4 | `u16` | Version: `1` |
| 6 | `u16` | Frame width in pixels |
| 8 | `u16` | Frame height in pixels |
| 10 | `u16` | Frame count |
| 12 | `u16` | Tag count |
| 14 | Frame data | One image for each frame |
| After frame data | Tag data | One entry for each tag |

Each frame uses `width * height` bytes. All frames have the same size.

Each tag has these fields:

| Type | Field |
| --- | --- |
| `u16` | Name length in UTF-8 bytes |
| `u8[name_length]` | Name bytes in UTF-8. No zero byte follows the name. |
| `u16` | First frame index |
| `u16` | Last frame index |
| `u8` | Direction |

Frame indexes start at zero. The range includes the first and last frame.

| Value | Direction |
| ---: | --- |
| 0 | Forward |
| 1 | Reverse |
| 2 | Ping-pong |
| 3 | Reverse ping-pong |

The file does not contain frame durations. The runtime sets the playback speed.

`AnimatedSpriteDocument` stores the frames and tags. `Assets::load_animated_sprite` loads the file as a runtime animation.

## Bitmap font: `FONT`

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `char[4]` | File ID: `FONT` |
| 4 | `u16` | Version: `1` |
| 6 | `u16` | Font height in pixels |
| 8 | `u16` | Glyph count |
| 10 | Glyph data | One entry for each glyph |

Each glyph uses `8 + width * height` bytes. The offsets below start at the glyph.

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `u32` | Unicode scalar value |
| 4 | `u16` | Advance in pixels |
| 6 | `u16` | Bitmap width in pixels |
| 8 | `u8[width * height]` | Bitmap, in row-major order |

Each codepoint must be a Unicode scalar value. A file must not contain the same codepoint twice. Each bitmap width must be greater than zero.

Bitmap width and advance are separate values. Use a small advance for narrow spacing. Use a large bitmap width for wide glyphs.

`FontDocument` stores the decoded glyphs. `Assets::load_font` loads the file as a runtime font.

## Tileset: `TSET`

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `char[4]` | File ID: `TSET` |
| 4 | `u16` | Version: `1` |
| 6 | `u8[16]` | Tileset UUID |
| 22 | `u16` | Tile width in pixels |
| 24 | `u16` | Tile height in pixels |
| 26 | `u16` | Tile count |
| 28 | Tile data | One image for each tile |

Each tile uses `tile_width * tile_height` bytes. All tiles have the same size. A tileset must contain at least one tile.

Tile IDs start at `1`. The first tile in the file has ID `1`. ID `0` means that a tilemap cell is empty. The file does not contain an empty tile.

`TilesetDocument` stores the UUID and tiles. `Assets::load_tileset` loads the file and registers the UUID.

## Tilemap: `TMAP`

A tilemap uses the UUID of a separate tileset. The tilemap file does not contain the tileset.

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `char[4]` | File ID: `TMAP` |
| 4 | `u16` | Version: `1` |
| 6 | `u16` | Width in cells |
| 8 | `u16` | Height in cells |
| 10 | `u8[16]` | UUID of the tileset |
| 26 | Cell data | One entry for each cell |

Each cell uses 3 bytes. The cells use row-major order. No padding follows a cell.

| Cell offset | Type | Field |
| ---: | --- | --- |
| 0 | `u16` | Tile ID. Zero means empty. |
| 2 | `u8` | Flip flags |

| Bit | Mask | Operation |
| ---: | --- | --- |
| 0 | `0x01` | Flip horizontally |
| 1 | `0x02` | Flip vertically |
| 2 | `0x04` | Transpose diagonally |
| 3 to 7 | `0xF8` | Reserved. Set these bits to zero. |

The renderer applies the diagonal transpose first. It then applies the horizontal and vertical flips.

The file length is `26 + width * height * 3` bytes. The map decoder checks the size and flip flags. It does not read the tileset.

Use the matching UUID and valid tile IDs. Load the tileset before the map. `Assets::load_tilemap` reports an error if it cannot find the tileset UUID.

## Source files

Aseprite files are source files. The game does not load them as pixel assets. Export or convert them to the current `.px*` formats before you use them.
