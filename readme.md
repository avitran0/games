# Games

## Build for ROCKNIX H700

Use the ROCKNIX distribution build system and its matching H700 toolchain/SDL3
sysroot. The Entware GCC on a retail device targets a different glibc and cannot
link against ROCKNIX's SDL3. See [`rocknix/README.md`](rocknix/README.md) for the
Podman build and deployment steps.
