# Build for ROCKNIX H700

From this games checkout on Fedora, run:

```sh
./rocknix/build.sh
```

The helper uses Podman and ROCKNIX's official build container, builds this
**local checkout** with ROCKNIX's H700 linker, sysroot, and SDL3 package, and
stages the ports in `out/rocknix-ports/`. It does not deploy anything; copy
`Pong/`, `Pong.sh`, `Snake/`, and `Snake.sh` from that directory to
`/storage/roms/ports/` on the device yourself.

The helper pins the ROCKNIX source to build ID `c445081` (the commit reported
by your H700). The checkout/build cache defaults to
`$XDG_CACHE_HOME/rocknix-distribution` (or `$HOME/.cache/rocknix-distribution`).
Override it, for example, to use temporary storage:

```sh
ROCKNIX_BUILD_DIR=/tmp/rocknix-distribution ./rocknix/build.sh
```

The first run clones ROCKNIX's `next` branch, checks out build ID `c445081`,
and builds the required toolchain and package dependencies, so it takes longer
and uses substantial disk space.
Subsequent runs reuse that checkout and cache. The helper builds `HEAD` from the local Git checkout; commit changes you want
included first. The package collects binary targets from Cargo's workspace
default members; when adding a game, add its crate to both
`workspace.members` and `workspace.default-members` in the root `Cargo.toml` and
it will be included automatically.
