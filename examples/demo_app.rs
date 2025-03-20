use egui_demo_lib::{ColorTest, DemoWindows};
use glfw::{self, Context};

use egui_glfw::{egui, EguiBackend};

/// Application.
pub struct Application {
    /// [`DemoWindows`].
    pub demo_windows: DemoWindows,

    /// [`ColorTest`].
    pub color_test: ColorTest,

    /// [`SubApplicationSelection`].
    pub sub_application_selection: SubApplicationSelection,

    /// Is inspection window open?
    pub is_inspection_window_open: bool,
}

impl Application {
    /// Create a new [`Application`].
    pub fn new() -> Self {
        Self {
            demo_windows: DemoWindows::default(),
            color_test: ColorTest::default(),
            sub_application_selection: SubApplicationSelection::Demo,
            is_inspection_window_open: true,
        }
    }

    /// Create the UI for the [`Application`].
    pub fn ui(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        match self.sub_application_selection {
            SubApplicationSelection::Demo => {
                self.demo_windows.ui(ui.ctx());
            }
            SubApplicationSelection::ColorTest => {
                egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                    self.color_test.ui(ui);
                });
            }
        }

        egui::Window::new("Inspection")
            .open(&mut self.is_inspection_window_open)
            .scroll([true, true])
            .show(ui.ctx(), |ui| {
                ui.ctx().clone().inspection_ui(ui);
            });
    }

    /// Create the UI for the [`Application`] selection.
    pub fn ui_app_selection(&mut self, ui: &mut egui::Ui, _id: egui::Id) {
        SubApplicationSelection::all()
            .iter()
            .for_each(|sub_app_selection| {
                if ui
                    .selectable_label(
                        *sub_app_selection == self.sub_application_selection,
                        sub_app_selection.to_string(),
                    )
                    .clicked()
                {
                    self.sub_application_selection = *sub_app_selection;
                }
            });
    }
}

impl Default for Application {
    fn default() -> Self {
        Self::new()
    }
}

/// Sub application selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubApplicationSelection {
    Demo,
    ColorTest,
}

impl SubApplicationSelection {
    /// Get all the [`SubApplicationSelection`]s.
    pub const fn all() -> &'static [Self] {
        &[Self::Demo, Self::ColorTest]
    }
}

impl std::fmt::Display for SubApplicationSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubApplicationSelection::Demo => write!(f, "Demo"),
            SubApplicationSelection::ColorTest => write!(f, "Color test"),
        }
    }
}

fn main() {
    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();

    // set to opengl 3.3 or higher
    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));
    // if msaa is available, use it
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));
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

    let mut app = Application::default();

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

        egui.begin_pass(&window, &mut glfw);

        egui::TopBottomPanel::top(egui::Id::new("top_panel")).show(egui.get_egui_ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("Edit", |ui| {
                    ui.menu_button("Windows", |ui| {
                        ui.checkbox(&mut app.is_inspection_window_open, "Inspection");
                    });
                });

                app.ui_app_selection(ui, egui::Id::new("top_panel").with("app_selection"));
            });
        });

        egui::CentralPanel::default().show(egui.get_egui_ctx(), |ui| {
            app.ui(ui, egui::Id::new("central_panel").with("app"));
        });

        let (width, height) = window.get_framebuffer_size();
        let output = egui.end_pass((width as _, height as _));

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
