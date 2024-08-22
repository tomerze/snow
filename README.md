# Snow

A single executable linux container template. 
With no container runtime installation required.

Based on Alpine Linux.

# How to use

## Build the container

`cd container`

Edit the `Dockerfile` to your liking.

Build using `./dockerfile_to_squashfs.sh Dockerfile alpine-snow.squashfs`

This will produce a Squashfs image that will be used as the rootfs of your container.
This image is extracted from Docker but Docker will not be used to run the container.

## Run the container

```sh
cd snow
cargo build --release
sudo RUST_LOG=INFO target/release/snow [arguments for zsh]
```

Snow runs `/bin/zsh` inside the container and forwards all arguments to it.
So to use it as an application container simply use the `-c` option of Zsh.

