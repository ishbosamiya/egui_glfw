use glfw::{self, Context};

use egui_glfw::{egui, EguiBackend};

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    // set to opengl 3.3 or higher
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    // if msaa is available, use it
    glfw.window_hint(glfw::WindowHint::Samples(Some(16)));
    glfw.window_hint(glfw::WindowHint::ScaleToMonitor(true));
    #[cfg(target_os = "macos")]
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));
    let (mut window, events) = glfw
        .create_window(1280, 720, "Demo App", glfw::WindowMode::Windowed)
        .expect("Failed to create glfw window");

    // setup bunch of polling data
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_scroll_polling(true);
    window.set_char_polling(true);
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
        gl::Enable(gl::FRAMEBUFFER_SRGB);
    }

    let mut egui = EguiBackend::new(&mut window, &mut glfw);

    egui_extras::install_image_loaders(egui.get_egui_ctx());

    let mut inspection_window = true;
    let mut demo_windows = egui_demo_lib::DemoWindows::default();
    let mut color_test = egui_demo_lib::ColorTest::default();
    let mut color_test_window_open = false;

    unsafe {
        gl::ClearColor(0.1, 0.3, 0.2, 1.0);
    }

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);

            match event {
                glfw::WindowEvent::Key(
                    glfw::Key::X,
                    _,
                    glfw::Action::Press,
                    glfw::Modifiers::Control,
                ) => {
                    egui.push_event(egui::Event::Cut);
                }
                glfw::WindowEvent::Key(
                    glfw::Key::C,
                    _,
                    glfw::Action::Press,
                    glfw::Modifiers::Control,
                ) => {
                    egui.push_event(egui::Event::Copy);
                }
                glfw::WindowEvent::Key(
                    glfw::Key::V,
                    _,
                    glfw::Action::Press,
                    glfw::Modifiers::Control,
                ) => {
                    let text = match copypasta_ext::try_context() {
                        Some(mut context) => Some(context.get_contents().unwrap()),
                        None => {
                            eprintln!("enable to gather context for clipboard");
                            None
                        }
                    };
                    if let Some(text) = text {
                        egui.push_event(egui::Event::Text(text));
                    }
                }
                _ => {}
            }
        });

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        egui.begin_frame(&window, &mut glfw);

        demo_windows.ui(egui.get_egui_ctx());

        egui::Window::new("window")
            .open(&mut inspection_window)
            .vscroll(true)
            .show(egui.get_egui_ctx(), |ui| {
                egui.get_egui_ctx().inspection_ui(ui);
            });

        egui::Window::new("Color Test")
            .open(&mut color_test_window_open)
            .show(egui.get_egui_ctx(), |ui| {
                color_test.ui(ui);
            });

        let (width, height) = window.get_framebuffer_size();
        let output = egui.end_frame((width as _, height as _));

        if !output.platform_output.copied_text.is_empty() {
            match copypasta_ext::try_context() {
                Some(mut context) => context
                    .set_contents(output.platform_output.copied_text)
                    .unwrap(),
                None => {
                    eprintln!("enable to gather context for clipboard");
                }
            }
        }

        window.swap_buffers();
    }
}
