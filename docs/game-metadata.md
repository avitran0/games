# Game metadata and cover art

Set metadata in each game's `Cargo.toml`. Use the `[package.metadata.game]` table. The build reads this table with `cargo metadata`.

```toml
[package.metadata.game]
title = "Snake"
description = "A sample description."
author = "avitrano"
players = 1
logo = "src/assets/title.pxs"
```

## Fields

- `title`: Name that EmulationStation displays. If this field is absent, the build uses the launcher name.
- `description`: Short game description.
- `author`: The build writes this value to EmulationStation's `developer` field.
- `players`: Number of players.
- `cover`: Optional `.pxs` image. The build writes it to EmulationStation's `image` field.
- `logo`: Optional `.pxs` image. The build writes it to both `image` and `marquee`. A theme must show marquee art to display the logo. If both `cover` and `logo` are set, the cover sets `image` and the logo sets `marquee`.

Set `cover` and `logo` to paths from the game's `Cargo.toml` folder. The build converts each `.pxs` file to PNG. It stores the PNG in the staged game folder. The converter reads sprite data through `formats` and reads colors from `api::Color`. Pixel value `0` stays transparent. The source `.pxs` files stay editable.

## Build and update

The build creates `gamelist-metadata.xml`. Each game entry uses the launcher's path, such as `./Snake.sh`. Each art path starts at the Ports folder, such as `./Snake/logo.png`.

`update.sh` does not install `gamelist-metadata.xml`. It uses this file to update the device's `gamelist.xml`. It matches each entry by launcher path. It updates the display fields only. It leaves play statistics and other entries unchanged. Before it replaces the file, it saves a backup as `gamelist.xml.bak`. It then replaces the file in one step to prevent a partial write.
