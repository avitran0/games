#!/usr/bin/env python3
"""Convert the bundled Mindustry pixel art to Factory's indexed sprite format.

Requires Pillow. The PNG source art and GPL-3.0 license are bundled in
factory/src/assets/mindustry_source/.
"""
from functools import lru_cache
from pathlib import Path
import re
import struct
from PIL import Image, ImageEnhance

ROOT = Path(__file__).resolve().parents[1] / "factory/src/assets"
SOURCE = ROOT / "mindustry_source"

color_source = (Path(__file__).resolve().parents[1] / "api/src/formats/color.rs").read_text()
color_source = color_source.split("define_palette! {", 1)[1].split("\n}", 1)[0]
PALETTE = [(0, 0, 0)] + [
    tuple(int(component, 16) for component in rgb)
    for rgb in re.findall(r"=> \[(0x[0-9a-f]+), (0x[0-9a-f]+), (0x[0-9a-f]+)\]", color_source)
]


@lru_cache(maxsize=None)
def nearest_palette_index(red, green, blue):
    return min(
        range(1, len(PALETTE)),
        key=lambda candidate: sum(
            (component - target) ** 2
            for component, target in zip((red, green, blue), PALETTE[candidate])
        ),
    )


def converted_pixels(image, size):
    image = image.convert("RGBA").resize((size, size), Image.Resampling.NEAREST)
    indices = []
    for red, green, blue, alpha in image.getdata():
        if alpha < 128:
            indices.append(0)
            continue
        indices.append(nearest_palette_index(red, green, blue))
    return bytes(indices)


def composite_layer(canvas, layer, position=(0, 0), opacity=1.0):
    """Composite a complete Mindustry sprite layer at its aligned position."""
    x, y = position
    if (
        x < 0
        or y < 0
        or x + layer.width > canvas.width
        or y + layer.height > canvas.height
    ):
        raise ValueError(
            f"layer {layer.size} at {position} does not fit canvas {canvas.size}"
        )
    if opacity != 1.0:
        layer = layer.copy()
        layer.putalpha(
            layer.getchannel("A").point(lambda value: round(value * opacity))
        )
    canvas.alpha_composite(layer, dest=position)


def write_animated(name, frames):
    width, height = frames[0].size
    if width != height or width % 8 or not 8 <= width <= 96:
        raise ValueError(f"{name}: unsupported sprite dimensions {width}x{height}")
    output = bytearray(struct.pack("<4sHHHHH", b"ANIM", 1, width, height, len(frames), 1))
    for frame in frames:
        if frame.size != (width, height):
            raise ValueError(f"{name}: inconsistent animation frame sizes")
        output.extend(converted_pixels(frame, width))
    tag = b"default"
    output.extend(struct.pack("<H", len(tag)) + tag)
    output.extend(struct.pack("<HHB", 0, len(frames) - 1, 0))
    (ROOT / "blocks" / f"{name}.pxa").write_bytes(output)


def write_static(name, image, size=16, directory="item"):
    output = struct.pack("<4sHHH", b"SPRT", 1, size, size)
    output += converted_pixels(image, size)
    (ROOT / directory / f"{name}.pxs").write_bytes(output)


def load(path):
    return Image.open(SOURCE / path).convert("RGBA")


# Mindustry's five conveyor blend variants, each with four authentic belt frames.
conveyor_dir = Path("blocks/distribution/conveyors")
for variant in range(5):
    frames = [
        load(conveyor_dir / f"conveyor-{variant}-{frame}.png").resize(
            (16, 16), Image.Resampling.NEAREST
        )
        for frame in range(4)
    ]
    write_animated(f"conveyor-{variant}", frames)

# Composite original block layers into appropriately sized, animated PXA sprites.
drill_base = load("blocks/drills/mechanical-drill.png")
drill_rotor = load("blocks/drills/mechanical-drill-rotator.png")
drill_frames = []
for angle in (0, 90, 180, 270):
    frame = drill_base.copy()
    composite_layer(
        frame, drill_rotor.rotate(angle, resample=Image.Resampling.NEAREST)
    )
    drill_frames.append(frame.resize((32, 32), Image.Resampling.NEAREST))
write_animated("drill", drill_frames)

smelter_base = load("blocks/production/silicon-smelter.png")
smelter_top = load("blocks/production/silicon-smelter-top.png")
smelter_frames = []
for level in (0.78, 0.96, 1.14, 0.96):
    frame = smelter_base.copy()
    composite_layer(frame, ImageEnhance.Brightness(smelter_top).enhance(level))
    smelter_frames.append(frame.resize((32, 32), Image.Resampling.NEAREST))
write_animated("smelter", smelter_frames)

solar = load("blocks/power/solar-panel.png").resize((16, 16), Image.Resampling.NEAREST)
write_animated(
    "solar_panel",
    [ImageEnhance.Brightness(solar).enhance(level) for level in (0.94, 1.0, 1.06, 1.0)],
)

container = load("blocks/storage/container.png").resize((32, 32), Image.Resampling.NEAREST)
write_animated(
    "storage",
    [ImageEnhance.Brightness(container).enhance(level) for level in (0.96, 1.0, 1.04, 1.0)],
)

core_base = load("blocks/storage/core-shard.png")
thruster1 = load("blocks/storage/core-shard-thruster1.png")
thruster2 = load("blocks/storage/core-shard-thruster2.png")
# The 3-tile core and 4-tile-wide thruster layers have different atlas sizes.
# Center both on the 4x4 game-block canvas rather than compositing at (0, 0),
# which clips the exhausts entirely off the core sprite.
core_canvas_size = (128, 128)
core_position = ((128 - core_base.width) // 2, (128 - core_base.height) // 2)
thruster_position = ((128 - thruster1.width) // 2, (128 - thruster1.height) // 2)
core_frames = []
for first, second in ((0.8, 0.3), (0.3, 0.8), (0.0, 1.0), (0.3, 0.8)):
    frame = Image.new("RGBA", core_canvas_size)
    composite_layer(frame, core_base, core_position)
    composite_layer(frame, thruster1, thruster_position, first)
    composite_layer(frame, thruster2, thruster_position, second)
    core_frames.append(frame.resize((64, 64), Image.Resampling.NEAREST))
write_animated("core", core_frames)

# Mindustry has no iron; use its lead/steel ore art as the closest match.
for output_name, source_name in (
    ("iron", "item-lead.png"),
    ("iron_plate", "item-surge-alloy.png"),
    ("coal", "item-coal.png"),
    ("titanium", "item-titanium.png"),
):
    write_static(output_name, load(Path("items") / source_name))

# Ore flecks are terrain overlays, not the carried-item icons above.
for ore_name, source_name in (("iron", "lead"), ("coal", "coal"), ("titanium", "titanium")):
    for variant in range(1, 4):
        texture = load(Path("blocks/environment/ores") / f"ore-{source_name}{variant}.png")
        write_static(f"{ore_name}-{variant - 1}", texture, directory="ores")

# Bring the pre-existing cursor mask art up to the new 16-pixel tile unit.
cursor_sizes = {"single": 16, "double": 32, "triple": 48, "quad": 64}
for name, target_size in cursor_sizes.items():
    path = ROOT / "cursor" / f"{name}.pxs"
    data = path.read_bytes()
    magic, version, width, height = struct.unpack_from("<4sHHH", data)
    if (width, height) == (target_size, target_size):
        continue
    if width * 2 != target_size or height * 2 != target_size:
        raise ValueError(f"{name} cursor has unexpected size {width}x{height}")
    pixels = data[10:]
    enlarged = bytes(
        pixels[(y // 2) * width + (x // 2)]
        for y in range(target_size)
        for x in range(target_size)
    )
    path.write_bytes(struct.pack("<4sHHH", magic, version, target_size, target_size) + enlarged)

# Terrain is a 16x16 Mindustry floor tile, matching the new block pixel unit.
terrain_tiles = [
    load(Path("blocks/environment") / filename).resize((16, 16), Image.Resampling.NEAREST)
    for filename in ("stone1.png", "sand-floor1.png", "shallow-water.png")
]
terrain_header = (ROOT / "terrain.pxt").read_bytes()[:22]
terrain_data = bytearray(terrain_header + struct.pack("<HHH", 16, 16, len(terrain_tiles)))
for tile in terrain_tiles:
    terrain_data.extend(converted_pixels(tile, 16))
(ROOT / "terrain.pxt").write_bytes(terrain_data)
