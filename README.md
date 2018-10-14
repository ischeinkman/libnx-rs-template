# libnx-rs-template

Simple template to get started with [lib-nx-rs](https://github.com/ischeinkman/libnx-rs).

## Setup

The simplest setup is to install Docker and clone the repository; if Docker is used, then everything is handled for you.

If you would like to use this template without Docker, you must install ```devkitpro```, ```cargo``` nightly, ```rust-src```, and ```xargo``` and make sure that rust is set as "nightly" by default. You also need gcc.

## Using the template

### Developing

Using ```libnx-rs``` is exactly like using standard Rust; the entire ```libstd``` has been ported. However, there are a couple of extra things to remember
when you develop with ```libnx-rs```:

* Since the Switch is a custom target, we have a custom target JSON file to build against and pass in a lot of extra arguments to the linker via a ```.cargo/config``` file, both of which are included in this template.

* Since technically Rust doesn't support the Switch, you need to use Xargo to build a valid sysroot. If you are using this template then all this involves is running the ```makew``` script, but if you are trying to get things set up from scratch it may become a more involved process.

* ```libnx-rs-std``` is not the standard Rust ```libstd```. Things may behave in strange ways that you don't expect.

### Building

There are 3 ways to package and build an ```nro``` from your code:

* The *simplest* way is to build the docker container and then run it as an executable, passing the working directory as a volume; ie ```docker build --rm -f Dockerfile -t rusted-switch:latest``` once and then  ```docker run --rm -v $PWD:/workdir rusted-switch``` to recompile. This process will not be able to take advantage of Rust's build cache or incremental compilation, and will require downloading all the dependencies each time.

* The *fastest* way is to install the dependencies manually and then just run ```./makew```. This will take advantage of the cache and local directories, but involves a long setup process.

* The *recommended* way is to create a docker *container* that all compilation will be done in. Just like in the first way, an image is built via ```docker build --rm -f Dockerfile -t rusted-switch:latest```, but unlike in the first way, it is run via the command ```docker run -v $PWD:/workdir -it rusted-switch:latest /bin/bash```, dropping you into a terminal inside the container at the ```workdir``` directory, which is the current code directory. If you want to compile the code, just run ```./makew``` from that container. This has all the benefits of the "fast" way without having to install all the different dependencies and is easier to keep up-to-date as things change.

## Credits and Thanks

* [Igor1201's rusted-switch](https://github.com/Igor1201/rusted-switch) which provided the foundation for this template.

* [Roblabla and his mighty Megaton Hammer](https://github.com/MegatonHammer/megaton-hammer) for allowing me to take over his Discord server.