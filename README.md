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

## Configuration

Place your Live2D models and a `config.toml` in `~/.config/rusty-ships/` (or
wherever the config folder on your system is generated, refer to `confy`
documentation[^confy]). Config and its default parameters are as follows:

```toml
[window]
size = [800, 600]
title = "Rusty Ships"
fit = "Cover"
[window.bg]
variant = "Color"
color = [0.0, 0.0, 0.0, 0.0]
image = ""

[model]
name = []
path = "assets"
[model.motions]
open = []
idle = ""
usr1 = ""
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

#### Background

Two values for `variant` supported: `Image` and `Color`. Color is defined by
`color` setting, which is an array of float RGBA values (four entries, 0.0 to
1.0). Image is set as a relative path `image` from configuration directory.

Both `image` and `color` are optional. Setting `variant: 'Image'` will fail
with no `image`. Setting `variant: 'Color'` with no `color` with be clear
transparent background equivalet to the following:

```toml
color = [0.0, 0.0, 0.0, 0.0]
```

> [!NOTE]
> Background image is not affected by `fit` setting and always works as with
> `Cover` setting.

### Model

#### Name, Path

Specifying `path` to resources (may include many models) and a list of `name`s
of particular models to use inside that directory.

Model index is the `XSCREENSAVER_SAVER_INDEX` environment variable to comply
with Xsecurelock's spec.

> [!IMPORTANT]
> Configuration assumes that the `name` is the name of directory inside of
> `path`, and that the same name is used for `model3.json` file contained
> inside of that folder.

#### Motions

Configuration of motions that will be played at the program start (`open`, can
be an array), a motion fallback (`idle`) when motion queue is empty, and a
motion set when SIGUSR1 is received (`usr1`, Xsecurelock uses it when
resetting saver).

Specified as a 2-size array, as Live2D motions have classes. So, for example,
if your model has the following motion file references:

```json
{
  "FileReferences": {
    "Motions": {
      "Idle": [
        {
          "Name": "Idle",
          "File": "motions/Idle.motion3.json",
          "Weight": 1
        }
      ],
      "Tap": [
        {
          "Name": "Anim",
          "File": "motions/Anim_1.motion3.json",
          "Weight": 1
        }
      ]
    }
  }
}
```

Then configuring these motions should be done as follows:

```toml
[model.motions]
open = [
    ["Tap", "Anim_1"],
]
idle = ["Idle", "Idle"]
```

> [!NOTE]
> Currently filenames are used for motion identification, not the `Name` field

## Usage

Launch the app with `rusty-ships`. That's it, it does not accept command line
argumets because I did not want it to. Window with your model should open,
unless you forgot to set it in configuration.

Playback can be controlled via `rusty-ships-ctl`. The following argumets are
accepted:

* `set [class] <motion>`: Queues the motion from the class with that name, if
  available. Class can be omitted, in that case it will be treated as `""`
* `pause`, `play`, `toggle`: self-explanatory
* `exit`: tells the program that you want to quit

Xsecurelock[^xsl] is supported. Appropriate mode is launched based on a
presence of `XSCREENSAVER_WINDOW` environment variable. Symlink app to the
appropriate location with appropriate name (e.g. `saver_ships`), and it should
work automagically.

[^live2d]: <https://www.live2d.com/en/>
[^alviewer]: <https://l2d.algwiki.moe/>
[^cubism-rs]: [Veykril/cubism-rs](https://github.com/Veykril/cubism-rs)
[^confy]: <https://crates.io/crates/confy>
[^xsl]: <https://github.com/google/xsecurelock>

