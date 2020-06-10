# raytracing

A ray tracer written in Rust which implements Blinn-Phong shading and physically-based rendering using a Metallic-Roughness workflow

### Building/Running

*Make sure Rust and `cargo` are installed.*

- For a live visualization of the ray tracer, run `cargo run -- scenes/scene.json`
- To output to a file, run `cargo run -- -o image.png scenes/scene.json`

*Additional scene files are in the [scenes](./scenes) folder*

### Renders

All renders are in the [renders](./renders/renders.md) folder.

`scenes/scene.json` (800 x 800 pixels, 6,615,349 rays, 2.370s on i7 8650U):

![scene.json](./renders/scene.png)

----

#### raytrace usage

The following options may be passed through `cargo` like so: `cargo run -- [FLAGS] [OPTIONS] <scene>`

```
ray tracer
A ray tracer written in Rust

USAGE:
    raytrace [FLAGS] [OPTIONS] <scene>

FLAGS:
    -h, --help           Prints help information
        --no-progress    Hide progress bar
        --no-random      Render to window sequentially instead of randomly
                         If --output is specified, --no-random has no effect
    -V, --version        Prints version information

OPTIONS:
    -o, --output <output>    Output rendered image to file
                             If omitted, image is rendered to a window

ARGS:
    <scene>    input scene as a json file
```
