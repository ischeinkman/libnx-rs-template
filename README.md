# libnx-rs-template

Simple template to get started with [lib-nx-rs](https://github.com/ischeinkman/libnx-rs).

Do note that this is made with the idea that it will be obsoleted eventually by [Megaton Hammer](https://github.com/MegatonHammer/megaton-hammer). Unless a pressing reason presents itself, this template and related crates will only be maintained until Megaton Hammer reaches a stable point of usability.

## Setup

The simplest setup is to install Docker and clone the repository; if Docker is used, then everything is handled for you.

If you would like to use this template without Docker, you must install ```devkitpro```, ```cargo``` nightly, ```rust-src```, and ```xargo``` and make sure that rust is set as "nightly" by default. You also need gcc.

## Using the template

### Developing

Just like in normal Rust, the main entrypoint is the ```main``` function in ```lib.rs```. Unlike in normal Rust, however, you are not allowed to change the signature of this function. It needs to be setup with the same signature as the main from ```C``` in order to link properly.

External crates can be linked via ```Cargo.toml```, just like normal. However, since we don't have a ```stdlib``` for switch just yet, you can only use other ```#![no_std]``` crates.

### Building

There are 3 ways to package and build an ```nro``` from your code:

* The *simplest* way is to build the docker container and then run it as an executable, passing the working directory as a volume; ie ```docker build --rm -f Dockerfile -t rusted-switch:latest``` once and then  ```docker run --rm -v $PWD:/workdir rusted-switch``` to recompile. This process will not be able to take advantage of Rust's build cache or incremental compilation, and will require downloading all the dependencies each time.

* The *fastest* way is to install the dependencies manually and then just run ```./makew```. This will take advantage of the cache and local directories, but involves a long setup process.

* The *recommended* way is to create a docker *container* that all compilation will be done in. Just like in the first way, an image is built via ```docker build --rm -f Dockerfile -t rusted-switch:latest```, but unlike in the first way, it is run via the command ```docker run -v $PWD:/workdir -it rusted-switch:latest /bin/bash```, dropping you into a terminal inside the container at the ```workdir``` directory, which is the current code directory. If you want to compile the code, just run ```./makew``` from that container. This has all the benefits of the "fast" way without having to install all the different dependencies and is easier to keep up-to-date as things change.

## Credits and Thanks

* [Igor1201's rusted-switch](https://github.com/Igor1201/rusted-switch) which provided the foundation for this template.

* [Roblabla and his mighty Megaton Hammer](https://github.com/MegatonHammer/megaton-hammer) for eventually obsoleting this.