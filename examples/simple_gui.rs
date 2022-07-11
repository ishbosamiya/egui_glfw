use glfw::{self, Context};
use nalgebra_glm as glm;

use egui_glfw::{egui, EguiBackend, TextureRGBA8};

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

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
        .create_window(1280, 720, "Simple GUI", glfw::WindowMode::Windowed)
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

    unsafe {
        gl::ClearColor(0.1, 0.3, 0.2, 1.0);
    }

    let mut texture_param_t = 0.0;
    let mut texture_param_r = 1.25;
    let mut texture_width = 300;
    let mut texture_height = 200;
    let mut texture = generate_texture(
        texture_width,
        texture_height,
        texture_param_t,
        texture_param_r,
    );

    let mut inspection_window = true;
    let mut text_input_test_window = true;
    let mut text_input_test = String::from("hello");

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

        egui::SidePanel::left("my_side_panel")
            .resizable(true)
            .show(egui.get_egui_ctx(), |ui| {
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

                ui.label(format!(
                    "window content scale: {:?}",
                    window.get_content_scale()
                ));
                ui.label(format!(
                    "monitor content scale: {:?}",
                    glfw.with_connected_monitors(|_, monitors| {
                        monitors
                            .iter()
                            .map(|monitor| monitor.get_content_scale())
                            .collect::<Vec<_>>()
                    })
                ));
                ui.label(format!(
                    "monitor physical size in mm: {:?}",
                    glfw.with_connected_monitors(|_, monitors| {
                        monitors
                            .iter()
                            .map(|monitor| monitor.get_physical_size())
                            .collect::<Vec<_>>()
                    })
                ));
                ui.label(format!(
                    "monitor physical size in inch: {:?}",
                    glfw.with_connected_monitors(|_, monitors| {
                        monitors
                            .iter()
                            .map(|monitor| {
                                let mm = monitor.get_physical_size();
                                (mm.0 as f32 / 25.4, mm.1 as f32 / 25.4)
                            })
                            .collect::<Vec<_>>()
                    })
                ));
                ui.label(format!(
                    "monitor positions: {:?}",
                    glfw.with_connected_monitors(|_, monitors| {
                        monitors
                            .iter()
                            .map(|monitor| monitor.get_pos())
                            .collect::<Vec<_>>()
                    })
                ));
                ui.label(format!("window position: {:?}", window.get_pos()));
            });

        let egui_window = egui::Window::new("window").open(&mut inspection_window);
        let egui_window = egui_window.vscroll(true);
        egui_window.show(egui.get_egui_ctx(), |ui| {
            egui.get_egui_ctx().inspection_ui(ui);
        });

        egui::Window::new("Text Input Test Window")
            .open(&mut text_input_test_window)
            .show(egui.get_egui_ctx(), |ui| {
                ui.text_edit_singleline(&mut text_input_test);
                ui.label(format!("wrote: {}", text_input_test));
            });

        egui::Window::new("User Texture Test Window").show(egui.get_egui_ctx(), |ui| {
            ui.add(egui::Slider::new(&mut texture_width, 0..=512).text("Image Width"));
            ui.add(egui::Slider::new(&mut texture_height, 0..=512).text("Image Height"));
            ui.add(egui::Slider::new(&mut texture_param_t, 0.0..=1.0).text("Parameter 1"));
            ui.add(egui::Slider::new(&mut texture_param_r, 0.0..=1.0).text("Parameter 2"));

            if ui.button("Generate Texture").clicked() {
                texture = generate_texture(
                    texture_width,
                    texture_height,
                    texture_param_t,
                    texture_param_r,
                );
            }

            ui.image(
                egui::TextureId::User(texture.get_gl_tex().into()),
                &[texture.get_width() as _, texture.get_height() as _],
            );
        });

        let (width, height) = window.get_framebuffer_size();
        let output = egui.end_frame(glm::vec2(width as _, height as _));

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

fn generate_texture(
    texture_width: usize,
    texture_height: usize,
    texture_param_t: f64,
    texture_param_r: f64,
) -> TextureRGBA8 {
    TextureRGBA8::from_pixels(
        texture_width,
        texture_height,
        (0..(texture_width * texture_height))
            .map(|pixel| {
                let pixel_x = pixel % texture_width;
                let pixel_y = pixel / texture_height;
                let (u, v) = (
                    pixel_x as f64 / texture_width as f64,
                    pixel_y as f64 / texture_height as f64,
                );

                // generating texture credits: notargs @notargs
                // https://twitter.com/notargs/status/1250468645030858753?s=20

                let func = |mut p: glm::DVec3| {
                    p[2] -= texture_param_t * 10.0;
                    let a = p[2] * 0.1;

                    let b = glm::mat2(a.cos(), a.sin(), -a.sin(), a.cos()) * p.xy();
                    p[0] = b[0];
                    p[1] = b[1];

                    0.1 - (glm::vec2(p[0].cos(), p[1].cos()) + glm::vec2(p[1].sin(), p[2].sin()))
                        .norm()
                };

                let param_r = glm::vec3(texture_param_r, texture_param_r, texture_param_r);
                let d = glm::vec3(0.5, 0.5, 0.5) - glm::vec3(u, v, 1.0) / param_r[1];

                let mut p = glm::zero();
                for _ in 0..32 {
                    let val = func(p) * d;
                    p += val;
                }

                let color = (glm::vec3(p[0].sin(), p[1].sin(), p[2].sin())
                    + glm::vec3(2.0, 5.0, 9.0))
                    / p.norm();

                (
                    (color[0] * 255.0) as _,
                    (color[1] * 255.0) as _,
                    (color[2] * 255.0) as _,
                    255,
                )
            })
            .collect(),
    )
}
