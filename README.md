# [glfw](https://github.com/PistonDevelopers/glfw-rs/) backend for [egui](https://github.com/emilk/egui/)

[glfw](https://github.com/PistonDevelopers/glfw-rs/) backend for
[egui](https://github.com/emilk/egui/). Should work right out of the
box, see examples for usage details.

## Dependency versions
| Dependency | Supported version |
|:-----------|:------------------|
| egui       | 0.30              |
| glfw       | 0.59              |

## Usage

``` toml
egui_glfw = { version = "0.9.1", git = "https://github.com/ishbosamiya/egui_glfw.git", branch = "v0.9.1-release" }
glfw = "0.59"
```

## Note about MSAA

`egui` does not require MSAA (multisample anti-aliasing) but the
application might need it. It is recommended to use 4 samples for
MSAA.

<details>

<summary>Why only 4 samples?</summary>

Attempting to use more number of samples may result in blurry text
(and shapes).

OpenGL specification states that at least 4 MSAA samples must be
supported but there is no upper bound, it is vendor dependent. So why
not just ask OpenGL for the maximum number of sample supported? Well,
it may report a value that although works may no longer just be MSAA
but might introduce supersampling. See the OpenGL extension
[NV_internalformat_sample_query](https://registry.khronos.org/OpenGL/extensions/NV/NV_internalformat_sample_query.txt). It
allows introduces supersampling to be used along with multisampling at
higher sample counts. This means that the supersampled fragments when
downsampled (usually with a linear filter) may make the text (and
shapes) blurry.

Vulkan does not have this problem (at least as of writing this), so it
is possible to get the true number of MSAA samples supported by the
GPU by initializing Vulkan but that is often too much effort. Since
OpenGL specification requires at least 4 samples to be supported, it
is recommended to use 4 samples for MSAA.

</details>
