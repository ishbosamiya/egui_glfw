pub mod drawable;
pub mod gpu_immediate;
pub mod input;
pub mod shader;
pub mod util;

use drawable::Drawable;
use gpu_immediate::{GPUImmediate, GPUVertCompType, GPUVertFetchMode};
use input::Input;
use shader::Shader;

use egui::{ClippedMesh, Output};
use gl;
use nalgebra_glm as glm;

pub struct EguiBackend {
    egui_ctx: egui::CtxRef,
    input: Input,
}

impl EguiBackend {
    pub fn new(window: &glfw::Window) -> Self {
        let mut input = Input::default();
        input.set_screen_rect(window);
        return Self {
            egui_ctx: Default::default(),
            input,
        };
    }

    pub fn begin_frame(&mut self) {
        self.egui_ctx.begin_frame(self.input.take());
    }

    pub fn end_frame(&self) -> (Output, Vec<ClippedMesh>) {
        let (output, shapes) = self.egui_ctx.end_frame();

        let meshes = self.egui_ctx.tessellate(shapes);

        return (output, meshes);
    }

    pub fn draw_gui(meshes: &[ClippedMesh], draw_data: &mut ClippedMeshDrawData) {
        meshes.iter().for_each(|mesh| {
            mesh.draw(draw_data).unwrap();
        });
    }

    pub fn handle_event(&mut self, event: &glfw::WindowEvent, window: &glfw::Window) {
        self.input.handle_event(event, window);
    }

    pub fn get_egui_ctx(&self) -> &egui::CtxRef {
        return &self.egui_ctx;
    }
}

pub struct ClippedMeshDrawData<'a> {
    imm: &'a mut GPUImmediate,

    /// needs a 2d shader with position, uv, and color defined per vertex
    shader: &'a Shader,

    screen_size: glm::Vec2,
}

impl<'a> ClippedMeshDrawData<'a> {
    pub fn new(imm: &'a mut GPUImmediate, shader: &'a Shader, screen_size: glm::Vec2) -> Self {
        return Self {
            imm,
            shader,
            screen_size,
        };
    }
}

impl Drawable<ClippedMeshDrawData<'_>, ()> for ClippedMesh {
    fn draw(&self, extra_data: &mut ClippedMeshDrawData) -> Result<(), ()> {
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
        // let uv_attr = format.add_attribute(
        //     "in_uv\0".to_string(),
        //     GPUVertCompType::F32,
        //     2,
        //     GPUVertFetchMode::Float,
        // );
        let color_attr = format.add_attribute(
            "in_color\0".to_string(),
            GPUVertCompType::F32,
            4,
            GPUVertFetchMode::Float,
        );

        let rect = &self.0;
        let mesh = &self.1;

        // TODO(ish): handle textures

        // Need to turn off backface culling because egui doesn't use proper winding order
        let cull_on;
        let depth_on;
        let scissor_not_on;
        unsafe {
            cull_on = gl::IsEnabled(gl::CULL_FACE) == gl::TRUE;
            depth_on = gl::IsEnabled(gl::DEPTH_TEST) == gl::TRUE;
            scissor_not_on = gl::IsEnabled(gl::SCISSOR_TEST) == gl::TRUE;
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Enable(gl::SCISSOR_TEST);
        }

        // scissor viewport since these are clipped meshes
        let scissor = (
            rect.left_bottom().x as _,
            (rect.left_bottom().y - rect.height()) as _,
            rect.width() as _,
            rect.height() as _,
        );
        unsafe { gl::Scissor(scissor.0, scissor.1, scissor.2, scissor.3) }

        imm.begin(gpu_immediate::GPUPrimType::Tris, mesh.indices.len(), shader);

        mesh.indices.iter().for_each(|index| {
            let vert = &mesh.vertices[*index as usize];

            // imm.attr_2f(uv_attr, vert.uv.x, vert.uv.y);
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

        return Ok(());
    }
}
