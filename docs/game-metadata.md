# Game metadata and cover art

Set metadata in each game's `Cargo.toml`. Use the `[package.metadata.game]` table. The build reads this table with `cargo metadata`.

```toml
[package.metadata.game]
title = "Snake"
description = "A sample description."
author = "avitrano"
players = 1
cover = "src/assets/title.pxs"
```

## Fields

- `title`: Name that EmulationStation displays. If this field is absent, the build uses the launcher name.
- `description`: Short game description.
- `author`: The build writes this value to EmulationStation's `developer` field.
- `players`: Number of players.
- `cover`: Path to a `.pxs` file. The path starts at the game's `Cargo.toml`. This field is optional.

## Build and update

The build converts the `.pxs` cover to `cover.png`. It stores the PNG in the staged game folder. The converter reads sprite data through `formats`. It reads colors from `api::Color`. Pixel value `0` stays transparent. This keeps cover colors aligned with the game palette. The source `.pxs` file stays editable.

The build creates `gamelist-metadata.xml`. Each game entry uses the launcher's path, such as `./Snake.sh`. Each image path starts at the Ports folder, such as `./Snake/cover.png`.

`update.sh` does not install `gamelist-metadata.xml`. It uses this file to update the device's `gamelist.xml`. It matches each entry by launcher path. It updates the display fields only. It leaves play statistics and other entries unchanged. Before it replaces the file, it saves a backup as `gamelist.xml.bak`. It then replaces the file in one step to prevent a partial write.
