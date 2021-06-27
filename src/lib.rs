pub mod drawable;
pub mod gpu_immediate;
pub mod input;
pub mod shader;
pub mod util;

use gl;

use drawable::Drawable;
use gpu_immediate::{GPUImmediate, GPUVertCompType, GPUVertFetchMode};
use input::Input;
use shader::Shader;

use egui::{ClippedMesh, Output};

pub struct EguiBackend {
    egui_ctx: egui::CtxRef,
    input: Input,
}

impl EguiBackend {
    pub fn new() -> Self {
        return Self {
            egui_ctx: Default::default(),
            input: Default::default(),
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
}

impl<'a> ClippedMeshDrawData<'a> {
    pub fn new(imm: &'a mut GPUImmediate, shader: &'a Shader) -> Self {
        return Self { imm, shader };
    }
}

impl Drawable<ClippedMeshDrawData<'_>, ()> for ClippedMesh {
    fn draw(&self, extra_data: &mut ClippedMeshDrawData) -> Result<(), ()> {
        let imm = &mut extra_data.imm;
        let shader = extra_data.shader;
        shader.use_shader();

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

        let _rect = &self.0;
        let mesh = &self.1;

        // TODO(ish): add the scissor stuff
        // TODO(ish): handle textures

        // Need to turn off backface culling because egui doesn't use proper winding order
        let cull_on;
        let depth_on;
        unsafe {
            cull_on = gl::IsEnabled(gl::CULL_FACE) == gl::TRUE;
            depth_on = gl::IsEnabled(gl::DEPTH_TEST) == gl::TRUE;
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
        }

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

        return Ok(());
    }
}
