# Mindustry art sources

The PNGs in this directory are copied from the official Mindustry repository's
`core/assets-raw/sprites` at commit `17012d61d9e4c3cbd9820d87e81c24495486ac53`.
Mindustry is copyright Anuken and contributors and is licensed under GPL-3.0;
the accompanying `LICENSE` is the upstream license text.

`tools/convert_mindustry_sprites.py` converts these files to the Factory indexed
sprite format, rescales block art to 16 pixels per in-game tile, and composites
separate animation layers. The conveyor frames and blend variants are upstream
Mindustry art. The terrain tiles use `stone1`, `sand-floor1`, and
`shallow-water`; deposit overlays use the lead, coal and titanium ore textures.
Mindustry has no iron item, so the lead sprite is used for the
Factory iron-ore icon and surge-alloy for the iron-plate icon. Existing cursor
masks are nearest-neighbour doubled to match the new tile size.

Upstream: <https://github.com/Anuken/Mindustry>
