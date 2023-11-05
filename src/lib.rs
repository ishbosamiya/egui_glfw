mod drawable;
mod gpu_immediate;
mod input;
mod shader;
mod texture;
mod util;

use std::{convert::TryInto, time::Duration, usize};

use drawable::Drawable;
use gpu_immediate::{GPUImmediate, GPUVertCompType, GPUVertFetchMode};
use input::Input;
use shader::Shader;
pub use texture::TextureRGBA8;

pub use egui;
use egui::{epaint::ahash::AHashMap, ClippedPrimitive, PlatformOutput};
use nalgebra_glm as glm;

/// Monitor data.
#[derive(Debug)]
pub struct MonitorData {
    pos: (u32, u32),
    size: (u32, u32), // width, height in pixels
}

impl MonitorData {
    /// Create new [`MonitorData`].
    pub fn new(pos: (u32, u32), size: (u32, u32)) -> Self {
        Self { pos, size }
    }
}

/// Egui backend by which the GUI can be drawn, inputs are handled,
/// etc.
pub struct EguiBackend {
    egui_ctx: egui::Context,
    input: Input,
    imm: GPUImmediate,
    textures: AHashMap<egui::TextureId, TextureRGBA8>,
    shader: Shader,
    start_time: std::time::Instant,
}

/// Get the true pixels per point of the monitor that the window in
/// is.
///
/// This can differ from the content scale.
#[allow(dead_code)]
fn get_monitor_true_pixels_per_point(window: &glfw::Window, glfw: &mut glfw::Glfw) -> f32 {
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
    pub fn new(window: &mut glfw::Window, _glfw: &mut glfw::Glfw) -> Self {
        // load opengl symbols
        gl::load_with(|symbol| window.get_proc_address(symbol));

        // TODO: need to set the pixels per point based on the content
        // scale of the monitor.
        //
        // let pixels_per_point = get_monitor_true_pixels_per_point(window, glfw);
        let pixels_per_point = 1.0;
        let mut input = Input::new(pixels_per_point);
        input.set_screen_rect(window, pixels_per_point);

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
            textures: AHashMap::new(),
            shader,
            start_time: std::time::Instant::now(),
        }
    }

    /// Start the egui frame. This sets up the necessary data for egui
    /// to work.
    pub fn begin_frame(&mut self, _window: &glfw::Window, _glfw: &mut glfw::Glfw) {
        let time = self.start_time.elapsed().as_secs_f64();
        unsafe {
            let raw_input = self.get_raw_input();
            raw_input.max_texture_side = Some(gl::MAX_TEXTURE_SIZE.try_into().unwrap());
            raw_input.time = Some(time);
        }
        self.egui_ctx.begin_frame(self.input.take());
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
    /// It is up to the caller to handle the [`egui::PlatformOutput`]
    /// within [`Output`], it is not processed by this function. This
    /// allows for more flexibility in the implementation. But for
    /// most use cases, it is useful to have clipboard support thus
    /// see the provided example to handle the same.
    ///
    /// # Example
    ///
    /// The following example handles copying the required text to the
    /// clipboard using the crate `copypasta_ext`.
    ///
    /// ```no_run
    /// let output = egui.end_frame((width as _, height as _));
    ///
    /// if !output.platform_output.copied_text.is_empty() {
    ///     match copypasta_ext::try_context() {
    ///         Some(mut context) => context
    ///             .set_contents(output.platform_output.copied_text)
    ///             .unwrap(),
    ///         None => {
    ///             eprintln!("enable to gather context for clipboard");
    ///         }
    ///     }
    /// }
    /// ```
    pub fn end_frame(&mut self, screen_size_in_pixels: (f32, f32)) -> Output {
        let full_output = self.egui_ctx.end_frame();

        // TODO: need to handle full_output.textures_delta
        //
        // This will involve storing all the textures internally
        // within Self.

        // delete any textures that must be freed
        full_output
            .textures_delta
            .free
            .iter()
            .for_each(|texture_id| {
                self.textures.remove(texture_id);
            });

        // create or update textures
        full_output
            .textures_delta
            .set
            .iter()
            .for_each(|(texture_id, delta)| {
                if let Some(texture) = self.textures.get_mut(texture_id) {
                    // update the texture
                    texture.update_from_egui(delta);
                } else {
                    // create the texture
                    self.textures
                        .insert(*texture_id, TextureRGBA8::from_egui(delta).expect("new texture created but probably got a partial delta which doesn't make sense"));
                }
            });

        let output = Output {
            platform_output: full_output.platform_output,
            repaint_after: full_output.repaint_after,
        };
        let shapes = full_output.shapes;

        let meshes = self.egui_ctx.tessellate(shapes);

        let pixels_per_point = self.egui_ctx.pixels_per_point();

        self.shader.use_shader();
        self.shader
            .set_mat4("projection\0", &glm::ortho(0.0, 1.0, 0.0, 1.0, 0.1, 1000.0));
        let screen_size_in_points = glm::vec2(
            screen_size_in_pixels.0 / pixels_per_point,
            screen_size_in_pixels.1 / pixels_per_point,
        );
        self.shader
            .set_vec2("screen_size\0", &screen_size_in_points);
        self.draw_gui(
            &meshes,
            pixels_per_point,
            glm::vec2(screen_size_in_pixels.0, screen_size_in_pixels.1),
        );

        output
    }

    /// Draw the gui by processing the provided `meshes`.
    fn draw_gui(
        &mut self,
        meshes: &[ClippedPrimitive],
        pixels_per_point: f32,
        screen_size_in_pixels: glm::Vec2,
    ) {
        // activate the texture. 31 is arbritrary, just needs to be
        // consistent between the shader and the texture that is
        // activated.
        self.shader.set_int("egui_texture\0", 31);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE31);
        }

        let mut draw_data = ClippedPrimitiveDrawData::new(
            &mut self.imm,
            &self.shader,
            &mut self.textures,
            pixels_per_point,
            screen_size_in_pixels,
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
        self.input
            .handle_event(event, window, self.egui_ctx.pixels_per_point());
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

    /// Get the internal raw input state mutably.
    ///
    /// # Safety
    ///
    /// This is basically never required unless something isn't
    /// supported. Need to be careful about how it is used. There
    /// cannot be any crashes but can cause the GUI to mess up if used
    /// wrong.
    pub unsafe fn get_raw_input(&mut self) -> &mut egui::RawInput {
        self.input.get_raw_input()
    }

    /// Get the egui context.
    pub fn get_egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }
}

struct ClippedPrimitiveDrawData<'a> {
    imm: &'a mut GPUImmediate,

    /// Needs a 2d shader with position, uv, and color defined per
    /// vertex
    shader: &'a Shader,
    /// Textures used by egui.
    textures: &'a mut AHashMap<egui::TextureId, TextureRGBA8>,

    /// Pixels per point.
    pixels_per_point: f32,

    /// Screen size in pixels.
    screen_size_in_pixels: glm::Vec2,
}

impl<'a> ClippedPrimitiveDrawData<'a> {
    pub fn new(
        imm: &'a mut GPUImmediate,
        shader: &'a Shader,
        textures: &'a mut AHashMap<egui::TextureId, TextureRGBA8>,
        pixels_per_point: f32,
        screen_size_in_pixels: glm::Vec2,
    ) -> Self {
        Self {
            imm,
            shader,
            textures,
            pixels_per_point,
            screen_size_in_pixels,
        }
    }
}

impl Drawable<ClippedPrimitiveDrawData<'_>, ()> for ClippedPrimitive {
    fn draw(&self, extra_data: &mut ClippedPrimitiveDrawData) -> Result<(), ()> {
        let rect = &self.clip_rect;
        let mesh = match &self.primitive {
            egui::epaint::Primitive::Mesh(mesh) => mesh,
            egui::epaint::Primitive::Callback(_) => {
                todo!("Need to add support for callback primitive")
            }
        };

        match mesh.texture_id {
            egui::TextureId::Managed(_) => {
                let texture = &mut extra_data.textures.get_mut(&mesh.texture_id).unwrap();
                unsafe {
                    gl::BindTexture(gl::TEXTURE_2D, texture.get_gl_tex());
                }
            }
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
        let clip_min_x = extra_data.pixels_per_point * rect.min.x;
        let clip_min_y = extra_data.pixels_per_point * rect.min.y;
        let clip_max_x = extra_data.pixels_per_point * rect.max.x;
        let clip_max_y = extra_data.pixels_per_point * rect.max.y;
        let clip_min_x = clip_min_x.clamp(0.0, extra_data.screen_size_in_pixels.x);
        let clip_min_y = clip_min_y.clamp(0.0, extra_data.screen_size_in_pixels.y);
        let clip_max_x = clip_max_x.clamp(clip_min_x, extra_data.screen_size_in_pixels.x);
        let clip_max_y = clip_max_y.clamp(clip_min_y, extra_data.screen_size_in_pixels.y);
        let clip_min_x = clip_min_x.round() as i32;
        let clip_min_y = clip_min_y.round() as i32;
        let clip_max_x = clip_max_x.round() as i32;
        let clip_max_y = clip_max_y.round() as i32;
        unsafe {
            gl::Scissor(
                clip_min_x,
                extra_data.screen_size_in_pixels.y as i32 - clip_max_y,
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

/// Output of [`EguiBackend::end_frame()`].
pub struct Output {
    /// egui's [`PlatformOutput`].
    pub platform_output: PlatformOutput,
    /// If `Duration::is_zero()`, egui is requesting immediate repaint
    /// (i.e. on the next frame).See [`FullOutput::needs_repaint`] for
    /// more details.
    pub repaint_after: Duration,
}
