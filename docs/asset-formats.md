# Pixel asset formats

## Common rules

Each file starts with a four-byte ID. A little-endian `u16` version follows it. The current version is `1`.

All other integers use little-endian byte order. Files have no compression or alignment padding. Files must end after the last field.

The file extension tells the user which asset type the file contains. The decoder checks the file ID and version. It does not check the extension.

All image data uses row-major order. The first pixel is at the top-left corner.

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

The patterns show the file fields. They check the ID, version, dimensions, and file length. The patterns also check fields that they can validate without another asset file.

## Pixels and dimensions

Sprite and tile width and height must be multiples of 8. Each dimension must be from 8 to 64 pixels.

Each sprite and tile pixel is one byte:

- `0` means transparent.
- `1` to `255` select Aurora palette colors.

The file does not store the palette. Aurora is hardcoded in `api/src/formats/color.rs`; `assets/palette.pal` mirrors it for external palette tools.

A font height must be from 1 to 64 pixels. Each glyph has a square bitmap. The bitmap has one byte per pixel. Zero is clear. The runtime treats any nonzero value as set.

## Static sprite: `SPRT`

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `char[4]` | File ID: `SPRT` |
| 4 | `u16` | Version: `1` |
| 6 | `u16` | Width in pixels |
| 8 | `u16` | Height in pixels |
| 10 | `u8[width * height]` | Pixel data |

The file length is `10 + width * height` bytes. The file stores one image.

`SpriteDocument` stores the decoded image. `Assets::load_sprite` loads the file into a runtime sprite.

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

Each frame has `width * height` bytes. All frames have the same size.

Each tag has these fields:

| Type | Field |
| --- | --- |
| `u16` | Name length in UTF-8 bytes |
| `u8[name_length]` | Name bytes. No zero byte follows the name. |
| `u16` | First frame index |
| `u16` | Last frame index |
| `u8` | Direction |

Frame indexes start at zero. The range includes both indexes.

| Value | Direction |
| ---: | --- |
| 0 | Forward |
| 1 | Reverse |
| 2 | Ping-pong |
| 3 | Reverse ping-pong |

The file does not store frame durations. The runtime controls playback speed.

`AnimatedSpriteDocument` stores frames and tags. `Assets::load_animated_sprite` loads the file into a runtime animation.

## Bitmap font: `FONT`

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `char[4]` | File ID: `FONT` |
| 4 | `u16` | Version: `1` |
| 6 | `u16` | Font height in pixels |
| 8 | `u16` | Glyph count |
| 10 | Glyph data | One entry for each glyph |

Each glyph has `6 + height * height` bytes. The offsets below start at the glyph.

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `u32` | Unicode scalar value |
| 4 | `u16` | Advance in pixels |
| 6 | `u8[height * height]` | Square bitmap |

A codepoint must be a Unicode scalar value. A file must not repeat a codepoint. The API finds the visible width from the rightmost set pixel. The advance is separate. A blank glyph can have a nonzero advance.

`FontDocument` stores the decoded glyphs. `Assets::load_font` loads the file into a runtime font.

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

Each tile has `tile_width * tile_height` bytes. All tiles have the same size. A tileset must contain at least one tile.

Tile IDs start at `1`. The first tile in the file has ID `1`. ID `0` means an empty tilemap cell. The file does not store an empty tile.

`TilesetDocument` stores the UUID and tiles. `Assets::load_tileset` loads the file and registers its UUID.

## Tilemap: `TMAP`

A tilemap refers to a separate tileset by UUID. It does not contain a tileset.

| Offset | Type | Field |
| ---: | --- | --- |
| 0 | `char[4]` | File ID: `TMAP` |
| 4 | `u16` | Version: `1` |
| 6 | `u16` | Width in cells |
| 8 | `u16` | Height in cells |
| 10 | `u8[16]` | UUID of the tileset |
| 26 | Cell data | One entry for each cell |

Each cell has three bytes. The cells use row-major order. No padding follows a cell.

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

The file length is `26 + width * height * 3` bytes. The map decoder checks the size and flip flags. It does not read the tileset. Use the matching UUID and valid tile IDs.

Load the tileset before the map. `Assets::load_tilemap` reports an error if the tileset UUID is not loaded.

## Source files

Aseprite files are source files. The game does not load them as pixel asset files. Export or convert them to the current `.px*` formats before use.
