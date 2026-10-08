#!/usr/bin/env python3

import argparse
import math
import sys
from pathlib import Path
from typing import Sequence

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError as error:
    raise SystemExit("Install Pillow with: python -m pip install Pillow") from error

DEFAULT_CHARACTERS = "".join(chr(codepoint) for codepoint in range(32, 127))
MAX_CHARACTERS = 4096
MAX_OUTPUT_PIXELS = 25_000_000
PADDING = 2
THRESHOLD = 128


def measure(font: ImageFont.FreeTypeFont, characters: str):
    ascent, descent = font.getmetrics()
    width = 0
    top = ascent
    bottom = descent
    glyphs = []

    for character in characters:
        try:
            bbox = font.getbbox(character, anchor="ls")
            advance = font.getlength(character)
        except (OSError, ValueError) as error:
            raise ValueError(f"cannot measure U+{ord(character):04X}: {error}") from error

        left, glyph_top, right, glyph_bottom = bbox
        left_edge = min(0, left)
        right_edge = max(advance, right)
        width = max(width, math.ceil(right_edge - left_edge))
        top = max(top, -glyph_top, 0)
        bottom = max(bottom, glyph_bottom, 0)
        glyphs.append((character, advance, bbox))

    return glyphs, width + 2 * PADDING, top + bottom + 2 * PADDING, top


def make_image(font: ImageFont.FreeTypeFont, characters: str, columns: int, antialias: bool):
    glyphs, cell_width, cell_height, top = measure(font, characters)
    rows = math.ceil(len(glyphs) / columns)
    size = columns * cell_width, rows * cell_height
    if size[0] * size[1] > MAX_OUTPUT_PIXELS:
        raise ValueError(f"output is too large ({size[0]}x{size[1]} pixels)")

    mask = Image.new("L", size)
    draw = ImageDraw.Draw(mask)
    content_width = cell_width - 2 * PADDING

    for index, (character, advance, bbox) in enumerate(glyphs):
        row, column = divmod(index, columns)
        left, _, right, _ = bbox
        left_edge = min(0, left)
        right_edge = max(advance, right)
        glyph_width = right_edge - left_edge
        x = column * cell_width + PADDING + (content_width - glyph_width) / 2 - left_edge
        baseline = row * cell_height + PADDING + top
        draw.text((x, baseline), character, font=font, fill=255, anchor="ls")

    if not antialias:
        mask = mask.point(lambda value: 255 if value >= THRESHOLD else 0, mode="1")
    image = Image.new("RGB", size, "white")
    image.paste("black", mask=mask)
    return image, rows


def main(argv: Sequence[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Convert a font into a PNG glyph sheet.")
    parser.add_argument("font", type=Path, help="input TTF, OTF, or supported WOFF2 font")
    parser.add_argument("output", type=Path, nargs="?", help="output PNG path")
    parser.add_argument("--size", type=int, default=28, help="font size in pixels (default: 28)")
    parser.add_argument("--columns", type=int, default=16, help="glyphs per row (default: 16)")
    parser.add_argument("--characters", help="characters to render (default: printable ASCII)")
    parser.add_argument("--antialias", action="store_true", help="preserve smooth glyph edges")
    args = parser.parse_args(argv)

    font_path = args.font.expanduser()
    if not font_path.is_file():
        parser.error(f"font file does not exist: {font_path}")
    if not 1 <= args.size <= 512:
        parser.error("--size must be from 1 to 512")
    if not 1 <= args.columns <= 256:
        parser.error("--columns must be from 1 to 256")

    characters = DEFAULT_CHARACTERS if args.characters is None else args.characters
    if not characters or any(not character.isprintable() for character in characters):
        parser.error("characters must be non-empty and contain no control characters")
    if len(characters) > MAX_CHARACTERS:
        parser.error(f"too many characters (maximum {MAX_CHARACTERS})")

    try:
        font = ImageFont.truetype(str(font_path), args.size)
    except (OSError, ValueError) as error:
        parser.error(f"cannot load font {font_path}: {error}")

    columns = min(args.columns, len(characters))
    output_path = args.output or font_path.with_name(f"{font_path.stem}-bitmap.png")
    output_path = output_path.expanduser()
    if output_path.suffix.lower() != ".png":
        parser.error("output must use the .png extension")
    if output_path.resolve() == font_path.resolve():
        parser.error("output path must not overwrite the input font")

    try:
        image, rows = make_image(font, characters, columns, args.antialias)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        image.save(output_path)
    except (OSError, ValueError) as error:
        parser.error(f"cannot create PNG: {error}")

    print(f"Wrote {output_path} ({image.width}x{image.height}; {len(characters)} glyphs in {rows} rows)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except KeyboardInterrupt:
        print("Interrupted.", file=sys.stderr)
        raise SystemExit(130)
