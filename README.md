# Advent(2)

These are my solutions to the [Advent(2)](https://osg.tuhh.de/Advent/) advent calendar of Linux syscalls.

For portability, `docker.sh` will build a docker environment for testing the programs, since they rely on linux syscalls. Some binaries rely on specific ARM assembly instructions, so even docker won't help you if you're on an x64 machine (though I guess you could run qemu or something).

## Running

Run the various binaries with `cargo run --bin <binary name>`. Some binaries can be built with extra debugging info by setting `RUSTFLAGS='--cfg debug'`.

## Questions

Unresolved questions to research:

* In day 2 (clone), when calling `clone` with the `CLONE_NEWUSER` flag, the child process has UID 65534 (which I'm assuming is actually -1) before we call `setuid`. Why?
