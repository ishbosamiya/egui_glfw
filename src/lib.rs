use egui::Output;

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
