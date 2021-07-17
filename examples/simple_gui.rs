use glfw::{self, Context};
use nalgebra_glm as glm;

use egui_glfw::EguiBackend;

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

    let mut egui = EguiBackend::new(&window, &mut glfw);

    unsafe {
        gl::ClearColor(0.1, 0.3, 0.2, 1.0);
    }

    let mut inspection_window = true;

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

    while !window.should_close() {
        glfw.poll_events();

        glfw::flush_messages(&events).for_each(|(_, event)| {
            egui.handle_event(&event, &window);
        });

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        egui.begin_frame();

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
                ui.label(format!("{:?}", monitor_data));
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
                ui.label(format!(
                    "current monitor: {:?}",
                    current_monitor.first().map(|(i, _)| i)
                ));
            });

        egui::Window::new("window")
            .open(&mut inspection_window)
            .scroll(true)
            .show(egui.get_egui_ctx(), |ui| {
                egui.get_egui_ctx().inspection_ui(ui);
            });

        let (width, height) = window.get_framebuffer_size();
        let _output = egui.end_frame(glm::vec2(width as _, height as _));

        window.swap_buffers();
    }
}
