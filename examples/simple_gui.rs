use egui;
use gl;
use glfw::{self, Context};
use nalgebra_glm as glm;

use egui_glfw::{gpu_immediate::GPUImmediate, shader::Shader, ClippedMeshDrawData, EguiBackend};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    // set to opengl 3.3 or higher
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    // if msaa is available, use it
    glfw.window_hint(glfw::WindowHint::Samples(Some(16)));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    let (mut window, events) = glfw
        .create_window(1280, 720, "Simple GUI", glfw::WindowMode::Windowed)
        .expect("Failed to create glfw window");

    // setup bunch of polling data
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_scroll_polling(true);
    window.make_current();

    // load opengl symbols
    gl::load_with(|symbol| window.get_proc_address(symbol));

    // enable vsync
    glfw.set_swap_interval(glfw::SwapInterval::Sync(1));

    // enable and disable certain opengl features
    unsafe {
        gl::Disable(gl::CULL_FACE);
        gl::Enable(gl::DEPTH_TEST);
        gl::Enable(gl::MULTISAMPLE);
    }

    let mut imm = GPUImmediate::new();

    let mut egui_shader = Shader::new(
        std::path::Path::new("shaders/egui_shader.vert"),
        std::path::Path::new("shaders/egui_shader.frag"),
    )
    .unwrap();

    println!(
        "egui: uniforms: {:?} attributes: {:?}",
        egui_shader.get_uniforms(),
        egui_shader.get_attributes(),
    );

    let mut egui = EguiBackend::new(&window);

    unsafe {
        gl::ClearColor(0.2, 0.2, 0.2, 1.0);
    }

    println!(
        "pixels_per_point: {}",
        egui.get_egui_ctx().pixels_per_point()
    );

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);
        });

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        egui.begin_frame();

        egui::SidePanel::left("my_side_panel").show(egui.get_egui_ctx(), |ui| {
            ui.heading("Hello World!");
            if ui.button("Quit").clicked() {
                window.set_should_close(true);
            }

            egui::ComboBox::from_label("Version")
                .width(150.0)
                .selected_text("foo")
                .show_ui(ui, |ui| {
                    egui::CollapsingHeader::new("Dev")
                        .default_open(true)
                        .show(ui, |ui| {
                            ui.label("contents");
                        });
                });
        });

        let (_output, meshes) = egui.end_frame();

        let (width, height) = window.get_size();
        egui_shader.use_shader();
        egui_shader.set_mat4(
            "projection\0",
            &glm::ortho(0.0, width as _, 0.0, height as _, 0.1, 1000.0),
        );
        let mut clipped_mesh_draw_data = ClippedMeshDrawData::new(
            &mut imm,
            &mut egui_shader,
            glm::vec2(width as _, height as _),
        );

        EguiBackend::draw_gui(&meshes, &mut clipped_mesh_draw_data);

        window.swap_buffers();
    }
}
