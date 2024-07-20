# [glfw](https://github.com/PistonDevelopers/glfw-rs/) backend for [egui](https://github.com/emilk/egui/)

[glfw](https://github.com/PistonDevelopers/glfw-rs/) backend for
[egui](https://github.com/emilk/egui/). Should work right out of the
box, see examples for usage details.

## Dependency versions
| Dependency | Supported version | Comments                                       |
|:-----------|:------------------|------------------------------------------------|
| egui       | 0.28              |                                                |
| glfw       | 0.55              | See [glfw version](#glfw-version) for details. |

## Usage

``` toml
egui_glfw = { version = "0.7.0", git = "https://github.com/ishbosamiya/egui_glfw.git", branch = "v0.7.0-release" }
glfw = "0.55"
```

## glfw version

[Dependency versions](#dependency-versions) lists a known working
version of `glfw`. The listed version is expected to work but any
other version is also likely to work.

`glfw` in this crate is imported as `glfw = "0.*"`, the expectation is
that `glfw` a fairly stable library, so any version should work. This
gives the user of this crate more flexibility for the version of
`glfw` that is actually used.
