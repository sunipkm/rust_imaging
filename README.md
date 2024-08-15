# Digital Camera Imaging Software

This is an attempt to build a standalone imaging service running on
Raspberry Pi that can be remotely controlled with a web browser.
Project is in the early stage of development, far from being usable for now,
however basic proof of concept is already working.

## Goals

#### Web application GUI for
 - Easy composition of the whole image
 - Focusing the image
 - Setting camera cooling parameters
 - Controlling batch imaging

#### Service
 - Running on a PC or Raspberry PI
 - Allowing standalone batch imaging

## Stage of the Project

Project is almost ready for first field test, following features are already implemented:
 - Camera ROI Control
 - Camera exposure Control (manual and auto)
 - Cooling
 - Saving series of FITS files on the disk/memory card
 - Software power off of the service
 - Status LEDs

 ## Images

Example gui for image composition, focusing and camera parameters (demo mode with
generated fractal image):

![alt text](doc/images/ccdi-composition.jpg)

## Supported platforms

Software now runs on:
 - Linux PC (x86_64)
 - Raspberry PI 3 (armv7)
 - Raspberry PI 4 (aarch64)
 - macOS (aarch64: FLI only, x86_64: FLI and ZWO)

## Technologies

Project uses following drivers / technologies
 - [CameraUnit](https://crates.io/crates/cameraunit) interface
 - Server uses `warp` and `tokio`
 - Client is written in `yew` and is compiled into WebAssembly

## Installing Dependencies to build Web Service

In order to build the web service, following debian packages are needed
```sh
$ sudo apt install libusb-1.0-0-dev llvm-dev libclang-dev clang libcfitsio-dev
```

Also, do no forget to update the Cargo registry to get the latest, SemVer compatible packages:
```sh
$ cargo update
```

Web service in the repository already contains client WASM binaries compiled
previously, so it is only needed to build the web service on the target
platform.

To run the web service, run:

```sh
cd ccdi-web-service
cargo run --release
```

## Installing Dependencies for Web Client Development

To install the trunk server (dev server for hosting and reloading web service
upon change), type:

`cargo install --locked trunk`

To run the client in the dev server, run:

```sh
$ cd ccdi-web-client
$ trunk serve --release --open
```

To build the client:
```sh
$ cd ccdi-web-client
$ trunk build --release
```

To build the service:
```sh
$ cd ccdi-web-server
$ ./copy-client-binaries.sh # builds the client and copies over necessary files
$ cargo build --release # build the server
```
Optionally, create a symbolic link to the built server and execute (for passing command line arguments):
```sh
$ ln -s ../target/release/ccdi-web-service server
$ ./server --addr 8080 # custom address
```

# Notes

- Tool to view FITS images: QFitsView - `sudo apt install qfitsview`
- Cross compiling x86_64 on aarch64 macOS:
  1. Install `homebrew` and `pkg-config`.
  1. Install dependencies (`libusb-1.0-0`, `libcfitsio`, possibly from source, and compile for x86-64, by prepending relevant commands with `arch -x86_64`) in `/path/to/x86_64libs`.
  1. Build using `PKG_CONFIG_PATH=/path/to/x86_64libs/lib/pkgconfig PKG_CONFIG_SYSROOT_DIR=/ cargo build --release --target x86_64-apple-darwin`


