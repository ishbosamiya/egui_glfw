pub mod drawable;
pub mod gpu_immediate;
pub mod input;
pub mod shader;
pub mod texture;
pub mod util;

use std::usize;

use drawable::Drawable;
use gpu_immediate::{GPUImmediate, GPUVertCompType, GPUVertFetchMode};
use input::Input;
use shader::Shader;
use texture::Texture;

use egui::{ClippedMesh, Output};
use nalgebra_glm as glm;

#[derive(Debug)]
struct MonitorData {
    pos: (u32, u32),
    size: (u32, u32), // width, height in pixels
}

impl MonitorData {
    fn new(pos: (u32, u32), size: (u32, u32)) -> Self {
        Self { pos, size }
    }
}

pub struct EguiBackend {
    egui_ctx: egui::CtxRef,
    input: Input,
    imm: GPUImmediate,
    texture: Option<Texture>,
    shader: Shader,
}

fn get_pixels_per_point(window: &glfw::Window, glfw: &mut glfw::Glfw) -> f32 {
    // reference:
    // https://developer.apple.com/documentation/appkit/nsscreen/1388375-userspacescalefactor
    // pixels per point is the (pixels per inch of the display) / 72.0

    let monitor_data = glfw.with_connected_monitors(|_, monitors| {
        monitors
            .iter()
            .map(|monitor| {
                let pos = monitor.get_pos();
                let pos = (pos.0 as _, pos.1 as _);
                let video_mode = monitor.get_video_mode().unwrap();
                let size = (video_mode.width, video_mode.height);
                MonitorData::new(pos, size)
            })
            .collect::<Vec<_>>()
    });

    let get_current_monitor = || {
        let window_pos = window.get_pos();
        let window_pos = (window_pos.0 as _, window_pos.1 as _);
        monitor_data
            .iter()
            .enumerate()
            .filter(|(_, data)| {
                (data.pos.0..=(data.pos.0 + data.size.0)).contains(&window_pos.0)
                    && (data.pos.1..=(data.pos.1 + data.size.1)).contains(&window_pos.1)
            })
            .collect::<Vec<_>>()
    };

    let current_monitor = get_current_monitor();

    let current_monitor = current_monitor.first().map(|(i, _)| *i).unwrap_or(0);

    let (monitor_pixels_size, monitor_size_in_inch) =
        glfw.with_connected_monitors(|_, monitors| {
            let monitor = &monitors[current_monitor];
            let vid_mode = monitor.get_video_mode().unwrap();
            let mm = monitor.get_physical_size();
            (vid_mode.width as f32, mm.0 as f32 / 25.4)
        });

    let pixels_per_inch = monitor_pixels_size / monitor_size_in_inch;
    pixels_per_inch / 72.0
}

impl EguiBackend {
    pub fn new(window: &mut glfw::Window, glfw: &mut glfw::Glfw) -> Self {
        // TODO(ish): need to figure out how to choose the correct
        // monitor based on the where the window is, for now choosing
        // the primary monitor to set the pixels per inch value
        // TODO(ish): need to figure out whether to use x axis or y
        // axis or the diagonal for calculating the pixels per inch
        // value

        // load opengl symbols
        gl::load_with(|symbol| window.get_proc_address(symbol));

        let pixels_per_point = get_pixels_per_point(window, glfw);
        let mut input = Input::new(pixels_per_point);
        input.set_screen_rect(window);

        let vertex_code = r#"
#version 330 core

uniform mat4 projection;
uniform vec2 screen_size; // (width, height)

in vec2 in_pos;
in vec2 in_uv;
in vec4 in_color;

out vec4 finalColor;
out vec2 v_uv;

// 0-1 linear  from  0-255 sRGB
// from egui_glium
vec3 linear_from_srgb(vec3 srgb) {
  bvec3 cutoff = lessThan(srgb, vec3(10.31475));
  vec3 lower = srgb / vec3(3294.6);
  vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
  return mix(higher, lower, cutoff);
}

vec4 linear_from_srgba(vec4 srgba) {
  return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}

void main()
{
  vec2 pos = vec2(in_pos.x / screen_size.x, 1.0 - in_pos.y / screen_size.y) * screen_size;
  gl_Position = projection * vec4(pos, -10.0, 1.0);
  finalColor = linear_from_srgba(in_color);
  v_uv = in_uv;
}
"#;
        let fragment_code = r#"
#version 330 core

uniform sampler2D egui_texture;

in vec4 finalColor;
in vec2 v_uv;
out vec4 fragColor;

void main()
{
  fragColor = finalColor * texture(egui_texture, v_uv);
}
"#;

        let shader =
            Shader::from_strings(vertex_code.to_string(), fragment_code.to_string()).unwrap();

        println!(
            "egui: uniforms: {:?} attributes: {:?}",
            shader.get_uniforms(),
            shader.get_attributes(),
        );

        Self {
            egui_ctx: Default::default(),
            imm: GPUImmediate::new(),
            input,
            texture: None,
            shader,
        }
    }

    pub fn begin_frame(&mut self, window: &glfw::Window, glfw: &mut glfw::Glfw) {
        let pixels_per_point = get_pixels_per_point(window, glfw);
        self.input.set_pixels_per_point(pixels_per_point);
        self.egui_ctx.begin_frame(self.input.take());
        if self.texture.is_none() {
            self.texture = Some(Texture::from_egui(&self.egui_ctx.texture()));
        }
    }

    pub fn end_frame(&mut self, screen_size: glm::Vec2) -> Output {
        let (output, shapes) = self.egui_ctx.end_frame();

        let meshes = self.egui_ctx.tessellate(shapes);

        self.shader.use_shader();
        self.shader.set_mat4(
            "projection\0",
            &glm::ortho(
                0.0,
                screen_size[0] as _,
                0.0,
                screen_size[1] as _,
                0.1,
                1000.0,
            ),
        );
        self.draw_gui(&meshes, screen_size);

        output
    }

    fn draw_gui(&mut self, meshes: &[ClippedMesh], screen_size: glm::Vec2) {
        self.shader.set_int("egui_texture\0", 31);
        let texture = self.texture.as_mut().unwrap();
        texture.update_from_egui(&self.egui_ctx.texture());
        texture.activate(31);

        let mut draw_data = ClippedMeshDrawData::new(&mut self.imm, &self.shader, screen_size);
        meshes
            .iter()
            .for_each(|mesh| mesh.draw(&mut draw_data).unwrap_or(()));
    }

    pub fn handle_event(&mut self, event: &glfw::WindowEvent, window: &glfw::Window) {
        self.input.handle_event(event, window);
    }

    pub fn get_egui_ctx(&self) -> &egui::CtxRef {
        &self.egui_ctx
    }
}

struct ClippedMeshDrawData<'a> {
    imm: &'a mut GPUImmediate,

    /// needs a 2d shader with position, uv, and color defined per vertex
    shader: &'a Shader,

    screen_size: glm::Vec2,
}

impl<'a> ClippedMeshDrawData<'a> {
    pub fn new(imm: &'a mut GPUImmediate, shader: &'a Shader, screen_size: glm::Vec2) -> Self {
        Self {
            imm,
            shader,
            screen_size,
        }
    }
}

impl Drawable<ClippedMeshDrawData<'_>, ()> for ClippedMesh {
    fn draw(&self, extra_data: &mut ClippedMeshDrawData) -> Result<(), ()> {
        let rect = &self.0;
        let mesh = &self.1;
        if mesh.indices.is_empty() {
            // TODO(ish): make this a proper error
            return Err(()); // mesh is not a mesh, no indices
        }
        let imm = &mut extra_data.imm;
        let shader = extra_data.shader;
        let screen_size = extra_data.screen_size;
        shader.use_shader();
        shader.set_vec2("screen_size\0", &screen_size);

        let format = imm.get_cleared_vertex_format();
        let pos_attr = format.add_attribute(
            "in_pos\0".to_string(),
            GPUVertCompType::F32,
            2,
            GPUVertFetchMode::Float,
        );
        let uv_attr = format.add_attribute(
            "in_uv\0".to_string(),
            GPUVertCompType::F32,
            2,
            GPUVertFetchMode::Float,
        );
        let color_attr = format.add_attribute(
            "in_color\0".to_string(),
            GPUVertCompType::F32,
            4,
            GPUVertFetchMode::Float,
        );

        // Need to turn off backface culling because egui doesn't use proper winding order
        let cull_on;
        let depth_on;
        let scissor_not_on;
        let blend_not_on;
        let srgb_not_on;
        unsafe {
            cull_on = gl::IsEnabled(gl::CULL_FACE) == gl::TRUE;
            depth_on = gl::IsEnabled(gl::DEPTH_TEST) == gl::TRUE;
            scissor_not_on = gl::IsEnabled(gl::SCISSOR_TEST) == gl::FALSE;
            blend_not_on = gl::IsEnabled(gl::BLEND) == gl::FALSE;
            srgb_not_on = gl::IsEnabled(gl::FRAMEBUFFER_SRGB) == gl::FALSE;
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::SCISSOR_TEST);
            gl::Enable(gl::BLEND);
            //Let OpenGL know we are dealing with SRGB colors so that it
            //can do the blending correctly. Not setting the framebuffer
            //leads to darkened, oversaturated colors.
            gl::Enable(gl::FRAMEBUFFER_SRGB);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA); // premultiplied alpha
        }

        //scissor viewport since these are clipped meshes
        let clip_min_x = rect.min.x;
        let clip_min_y = rect.min.y;
        let clip_max_x = rect.max.x;
        let clip_max_y = rect.max.y;
        let clip_min_x = clip_min_x.clamp(0.0, screen_size[0]);
        let clip_min_y = clip_min_y.clamp(0.0, screen_size[1]);
        let clip_max_x = clip_max_x.clamp(clip_min_x, screen_size[0]);
        let clip_max_y = clip_max_y.clamp(clip_min_y, screen_size[1]);
        let clip_min_x = clip_min_x.round() as i32;
        let clip_min_y = clip_min_y.round() as i32;
        let clip_max_x = clip_max_x.round() as i32;
        let clip_max_y = clip_max_y.round() as i32;
        unsafe {
            gl::Scissor(
                clip_min_x,
                screen_size[1] as i32 - clip_max_y,
                clip_max_x - clip_min_x,
                clip_max_y - clip_min_y,
            );
        }

        imm.begin(gpu_immediate::GPUPrimType::Tris, mesh.indices.len(), shader);

        mesh.indices.iter().for_each(|index| {
            let vert = &mesh.vertices[*index as usize];

            imm.attr_2f(uv_attr, vert.uv.x, vert.uv.y);
            imm.attr_4f(
                color_attr,
                vert.color.r().into(),
                vert.color.g().into(),
                vert.color.b().into(),
                vert.color.a().into(),
            );
            imm.vertex_2f(pos_attr, vert.pos.x, vert.pos.y);
        });

        imm.end();

        if cull_on {
            unsafe {
                gl::Enable(gl::CULL_FACE);
            }
        }
        if depth_on {
            unsafe {
                gl::Enable(gl::DEPTH_TEST);
            }
        }
        if scissor_not_on {
            unsafe {
                gl::Disable(gl::SCISSOR_TEST);
            }
        }
        if blend_not_on {
            unsafe {
                gl::Disable(gl::BLEND);
            }
        }
        if srgb_not_on {
            unsafe {
                gl::Disable(gl::FRAMEBUFFER_SRGB);
            }
        }

        Ok(())
    }
}
