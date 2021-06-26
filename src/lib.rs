pub mod drawable;
pub mod gpu_immediate;
pub mod shader;
pub mod util;

use gl;

use drawable::Drawable;
use gpu_immediate::{GPUImmediate, GPUVertCompType, GPUVertFetchMode};
use shader::Shader;

use egui::{ClippedMesh, Output};

pub struct EguiBackend {
    egui_ctx: egui::CtxRef,
}

impl EguiBackend {
    pub fn new() -> Self {
        return Self {
            egui_ctx: Default::default(),
        };
    }

    pub fn begin_frame(&mut self, raw_input: egui::RawInput) {
        self.egui_ctx.begin_frame(raw_input);
    }

    pub fn end_frame(&self) -> Output {
        let (output, shapes) = self.egui_ctx.end_frame();

        let _meshes = self.egui_ctx.tessellate(shapes);

        return output;
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

        let _rect = &self.0;
        let mesh = &self.1;

        // TODO(ish): add the scissor stuff
        // TODO(ish): handle textures

        // Need to turn off backface culling because egui doesn't use proper winding order
        let cull_on;
        unsafe {
            cull_on = gl::IsEnabled(gl::CULL_FACE) == gl::TRUE;
            gl::Disable(gl::CULL_FACE);
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

        return Ok(());
    }
}
