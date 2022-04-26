mod drawable;
mod gpu_immediate;
mod input;
mod shader;
mod texture;
mod util;

#[cfg(all(
    feature = "egui_0_16",
    any(feature = "egui_0_15", feature = "egui_0_14")
))]
compile_error!("multiple egui versions through features cannot be enabled at the same time");
#[cfg(all(feature = "egui_0_15", any(feature = "egui_0_14")))]
compile_error!("multiple egui versions through features cannot be enabled at the same time");

/// public re-export of `egui` so version related errors do not
/// occur. `use egui_glfw::egui;` instead of adding `egui` as a
/// separate dependency along side egui_glfw.
#[cfg(feature = "egui_0_14")]
pub extern crate dep_egui_0_14 as egui;
#[cfg(feature = "egui_0_15")]
pub extern crate dep_egui_0_15 as egui;
#[cfg(feature = "egui_0_16")]
pub extern crate dep_egui_0_16 as egui;

use std::{convert::TryInto, usize};

use drawable::Drawable;
use gpu_immediate::{GPUImmediate, GPUVertCompType, GPUVertFetchMode};
use input::Input;
use shader::Shader;
pub use texture::Texture;

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

/// Egui backend by which the GUI can be drawn, inputs are handled,
/// etc.
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
    /// Create a new egui backend utilizing glfw as the backend.
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

        let egui_shader_vert_code = include_str!("../shaders/egui_shader.vert");
        let egui_shader_frag_code = include_str!("../shaders/egui_shader.frag");
        let shader = Shader::from_strings(egui_shader_vert_code, egui_shader_frag_code).unwrap();

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

    /// Start the egui frame. This sets up the necessary data for egui
    /// to work.
    pub fn begin_frame(&mut self, window: &glfw::Window, glfw: &mut glfw::Glfw) {
        let pixels_per_point = get_pixels_per_point(window, glfw);
        self.input.set_pixels_per_point(pixels_per_point);
        self.egui_ctx.begin_frame(self.input.take());
        if self.texture.is_none() {
            // egui 0.16 onward switches to font_image() and FontImage
            // instead of using texture() and Texture
            #[cfg(any(feature = "egui_0_14", feature = "egui_0_15"))]
            {
                self.texture = Some(Texture::from_egui(&self.egui_ctx.texture()));
            }
            #[cfg(not(any(feature = "egui_0_14", feature = "egui_0_15")))]
            {
                self.texture = Some(Texture::from_egui(&self.egui_ctx.font_image()));
            }
        }
    }

    /// End the egui frame. This processes the GUI to render it to the
    /// screen.
    ///
    /// It is best to have this as the final call in the entire frame
    /// especially when other elements also drawn using OpenGL to the
    /// default render target so that the GUI is overlayed on top the
    /// other content without having to mess with the depth buffer.
    ///
    /// # Note
    ///
    /// It is up to the caller to handle the [`egui::Output`], it is
    /// not processed by this function. This allows for more
    /// flexibility in the implementation. But for most use cases, it
    /// is useful to have clipboard support thus see the provided
    /// example to handle the same.
    ///
    /// # Example
    ///
    /// The following example handles copying the required text to the
    /// clipboard using the crate `copypasta_ext`.
    ///
    /// ```no_run
    /// let output = egui.end_frame(glm::vec2(width as _, height as _));
    ///
    /// if !output.copied_text.is_empty() {
    ///     match copypasta_ext::try_context() {
    ///         Some(mut context) => context.set_contents(output.copied_text).unwrap(),
    ///         None => {
    ///             eprintln!("enable to gather context for clipboard");
    ///         }
    ///     }
    /// }
    /// ```
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

    /// Draw the gui by processing the provided `meshes`.
    fn draw_gui(&mut self, meshes: &[ClippedMesh], screen_size: glm::Vec2) {
        self.shader.set_int("egui_texture\0", 31);
        let texture = self.texture.as_mut().unwrap();
        // egui 0.16 onward switches to font_image() and FontImage
        // instead of using texture() and Texture
        #[cfg(any(feature = "egui_0_14", feature = "egui_0_15"))]
        {
            texture.update_from_egui(&self.egui_ctx.texture());
        }
        #[cfg(not(any(feature = "egui_0_14", feature = "egui_0_15")))]
        {
            texture.update_from_egui(&self.egui_ctx.font_image());
        }
        texture.activate(31);

        let mut draw_data = ClippedMeshDrawData::new(
            &mut self.imm,
            &self.shader,
            screen_size,
            texture.get_gl_tex(),
        );
        meshes
            .iter()
            .for_each(|mesh| mesh.draw(&mut draw_data).unwrap_or(()));
    }

    /// Process the [`glfw::WindowEvent`] to convert it to an event
    /// that egui supports.
    ///
    /// Also look at [`Self::push_event()`] for handling other egui
    /// events that are currently unsupported.
    pub fn handle_event(&mut self, event: &glfw::WindowEvent, window: &glfw::Window) {
        self.input.handle_event(event, window);
    }

    /// Push a [`egui::Event`] to egui. This is useful when a certain
    /// event is not handled yet or it is not possible to handle an
    /// event due to discrepancies in what shortcut to use. An example
    /// of this is [`egui::Event::Copy`], the user may want a shortcut
    /// that is not `C-c`.
    ///
    /// # Example
    ///
    /// A common use case would be to setup the code for cut, copy,
    /// paste since it is not handled by `egui_glfw` due to shortcut
    /// discrepancies.
    ///
    /// ```no_run
    /// egui.handle_event(event, window);
    /// match event {
    ///     glfw::WindowEvent::Key(
    ///         glfw::Key::X,
    ///         _,
    ///         glfw::Action::Press,
    ///         glfw::Modifiers::Control,
    ///     ) => {
    ///         egui.push_event(egui::Event::Cut);
    ///     }
    ///     glfw::WindowEvent::Key(
    ///         glfw::Key::C,
    ///         _,
    ///         glfw::Action::Press,
    ///         glfw::Modifiers::Control,
    ///     ) => {
    ///         egui.push_event(egui::Event::Copy);
    ///     }
    ///     glfw::WindowEvent::Key(
    ///         glfw::Key::V,
    ///         _,
    ///         glfw::Action::Press,
    ///         glfw::Modifiers::Control,
    ///     ) => {
    ///         let text = match copypasta_ext::try_context() {
    ///             Some(mut context) => Some(context.get_contents().unwrap()),
    ///             None => {
    ///                 eprintln!("enable to gather context for clipboard");
    ///                 None
    ///             }
    ///         };
    ///         if let Some(text) = text {
    ///             egui.push_event(egui::Event::Text(text));
    ///         }
    ///     }
    ///     _ => {}
    /// }
    /// ```
    ///
    /// Note that for [`egui::Event::Cut`] and [`egui::Event::Copy`],
    /// it is important support handle the output so that the copied
    /// text is copied to the clipboard. See [`Self::end_frame()`] for
    /// more details.
    pub fn push_event(&mut self, event: egui::Event) {
        self.input.push_event(event);
    }

    /// Get the egui context.
    pub fn get_egui_ctx(&self) -> &egui::CtxRef {
        &self.egui_ctx
    }
}

struct ClippedMeshDrawData<'a> {
    imm: &'a mut GPUImmediate,

    /// needs a 2d shader with position, uv, and color defined per vertex
    shader: &'a Shader,

    screen_size: glm::Vec2,
    egui_texture_gl_tex: gl::types::GLuint,
}

impl<'a> ClippedMeshDrawData<'a> {
    pub fn new(
        imm: &'a mut GPUImmediate,
        shader: &'a Shader,
        screen_size: glm::Vec2,
        egui_texture_gl_tex: gl::types::GLuint,
    ) -> Self {
        Self {
            imm,
            shader,
            screen_size,
            egui_texture_gl_tex,
        }
    }
}

impl Drawable<ClippedMeshDrawData<'_>, ()> for ClippedMesh {
    fn draw(&self, extra_data: &mut ClippedMeshDrawData) -> Result<(), ()> {
        let rect = &self.0;
        let mesh = &self.1;

        match mesh.texture_id {
            egui::TextureId::Egui => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, extra_data.egui_texture_gl_tex);
            },
            egui::TextureId::User(gl_tex) => unsafe {
                gl::BindTexture(gl::TEXTURE_2D, gl_tex.try_into().unwrap());
            },
        }

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

            // need to flip the y coordinate of the UV since egui has
            // (0.0, 0.0) as top left and (1.0, 1.0) as bottom right
            // but OpenGL has (0.0, 0.0) as bottom left and (1.0, 1.0)
            // as top right
            imm.attr_2f(uv_attr, vert.uv.x, 1.0 - vert.uv.y);
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
