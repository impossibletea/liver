# Rusty Ships

## About

Live2D[^live2d] model viewer with support for animation.

Inspired by Azur Lane Live2D viewer[^alviewer].

Made possible with incredible wrapper[^cubism-rs] for Cubism SDK.

> [!NOTE]
> The whole library is copied to this project instead of a dependency in a
> Cargo.toml due to two necessary changes that had to be made in order for
> models I use to work:
> * Motions in my models are under the `""` key in `model3.json`, so the
>   wrapper did not parse them in `motion_references`
> * Restricted Beziers in `motion3.json` files

## Configuration

Place your Live2D models and a `config.toml` in `~/.config/rusty-ships/` (or
wherever the config folder on your system is generated, refer to `confy`
documentation[^confy]). Config and its default parameters are as follows:

```toml
[window]
size = [800, 600]
title = "Rusty Ships"
fit = "Cover"

[model]
name = ""
path = "assets"
[model.motions]
open = []
idle = ""
```

### Window

#### Size

Logical size of initially created window. "Logical" means that your DPI is
taken into account. I.e. if size is 800x600 with DPI 2 (192 in X11 terms), the
actual size in pixels will be 1600x1200.

#### Title

The only usage for it now (as it seems to me) is to add window to exceptions
of your window manager by title.

#### Fit

Fit mode represents how the canvas will fit in the window. Currently two modes
are supported:
* Cover: scaled and clipped, takes all the window
* Contain: scaled so that all the canvas is shown

### Model

#### Name, Path

Specifying `path` to resources (may include many models) and `name` of a
particular model to use inside that directory.

> [!IMPORTANT]
> Configuration assumes that the `name` is the name of directory inside of
> `path`, and that the same name is used for `model3.json` file contained
> inside of that folder.

#### Motions

Configuration of motions that will be played at the program start (`open`, can
be an array) and a motion fallback (`idle`) when motion queue is empty.

## Building

First refer to `cubism-rs` [documentation](res/cubism-rs/README.md) to place
Live2DCubismCore library appropriately. I added a helper [file](.cargo/config)
to not provide environment variable every time you wish to run `cargo
<action>` command.

After that it is just as usual for rust projects:

```bash
cargo install --path .
rusty-ships
ed ~/.config/rusty-ships/config.toml
rusty-ships
# Window should appear
```

[^live2d]: <https://www.live2d.com/en/>
[^alviewer]: <https://l2d.algwiki.moe/>
[^cubism-rs]: [Veykril/cubism-rs](https://github.com/Veykril/cubism-rs)
[^confy]: <https://crates.io/crates/confy>

