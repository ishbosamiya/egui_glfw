use egui::RawInput;
use glfw::{self, Action, Key};

pub struct Input {
    raw_input: RawInput,
}

impl Default for Input {
    fn default() -> Self {
        return Self {
            raw_input: Default::default(),
        };
    }
}

impl Input {
    /// Refer to egui's RawInput for details on take
    pub fn take(&mut self) -> RawInput {
        return self.raw_input.take();
    }

    pub fn handle_event(&mut self, _event: &glfw::WindowEvent) {
        todo!("handle the event");
    }

    pub fn set_screen_rect(&mut self, window: &glfw::Window) {
        let screen_size = window.get_size();
        // TODO(ish): will need to divide with pixels per point most probably
        self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
            Default::default(),
            egui::vec2(screen_size.0 as f32, screen_size.1 as f32),
        ));
    }

    pub fn set_modifiers(&mut self, window: &glfw::Window) {
        let shift;
        let control;
        let alt;
        let command;
        // TODO(ish): support mac properly

        if window.get_key(Key::LeftShift) == Action::Press
            || window.get_key(Key::RightShift) == Action::Press
        {
            shift = true;
        } else {
            shift = false;
        }

        if window.get_key(Key::LeftControl) == Action::Press
            || window.get_key(Key::RightControl) == Action::Press
        {
            control = true;
        } else {
            control = false;
        }

        if window.get_key(Key::LeftAlt) == Action::Press
            || window.get_key(Key::RightAlt) == Action::Press
        {
            alt = true;
        } else {
            alt = false;
        }

        command = control;

        let modifiers = egui::Modifiers {
            alt,
            ctrl: control,
            shift,
            mac_cmd: false,
            command,
        };

        self.raw_input.modifiers = modifiers;
    }
}
