use egui::{Event, Pos2, RawInput};
use glfw::{self, Action, Key, MouseButton};

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

    #[inline]
    fn is_pressed(action: &glfw::Action) -> bool {
        match action {
            glfw::Action::Press => true,
            glfw::Action::Release => false,
            glfw::Action::Repeat => true,
        }
    }

    #[inline]
    fn button_type(button: &glfw::MouseButton) -> Option<egui::PointerButton> {
        match button {
            MouseButton::Button1 => Some(egui::PointerButton::Primary),
            MouseButton::Button2 => Some(egui::PointerButton::Secondary),
            MouseButton::Button3 => Some(egui::PointerButton::Middle),
            _ => None,
        }
    }

    #[inline]
    fn get_modifier(modifiers: &glfw::Modifiers) -> egui::Modifiers {
        // TODO(ish): handle mac os properly
        egui::Modifiers {
            alt: modifiers.contains(glfw::Modifiers::Alt),
            ctrl: modifiers.contains(glfw::Modifiers::Control),
            shift: modifiers.contains(glfw::Modifiers::Shift),
            mac_cmd: false,
            command: modifiers.contains(glfw::Modifiers::Control),
        }
    }

    #[inline]
    fn get_cur_pos(window: &glfw::Window) -> Pos2 {
        let pos = window.get_cursor_pos();
        egui::pos2(pos.0 as _, pos.1 as _)
    }

    #[inline]
    fn get_key(key: &glfw::Key) -> Option<egui::Key> {
        Some(match key {
            glfw::Key::A => egui::Key::A,
            glfw::Key::B => egui::Key::B,
            glfw::Key::C => egui::Key::C,
            glfw::Key::D => egui::Key::D,
            glfw::Key::E => egui::Key::E,
            glfw::Key::F => egui::Key::F,
            glfw::Key::G => egui::Key::G,
            glfw::Key::H => egui::Key::H,
            glfw::Key::I => egui::Key::I,
            glfw::Key::J => egui::Key::J,
            glfw::Key::K => egui::Key::K,
            glfw::Key::L => egui::Key::L,
            glfw::Key::M => egui::Key::M,
            glfw::Key::N => egui::Key::N,
            glfw::Key::O => egui::Key::O,
            glfw::Key::P => egui::Key::P,
            glfw::Key::Q => egui::Key::Q,
            glfw::Key::R => egui::Key::R,
            glfw::Key::S => egui::Key::S,
            glfw::Key::T => egui::Key::T,
            glfw::Key::U => egui::Key::U,
            glfw::Key::V => egui::Key::V,
            glfw::Key::W => egui::Key::W,
            glfw::Key::X => egui::Key::X,
            glfw::Key::Y => egui::Key::Y,
            glfw::Key::Z => egui::Key::Z,

            glfw::Key::Down => egui::Key::ArrowDown,
            glfw::Key::Left => egui::Key::ArrowLeft,
            glfw::Key::Right => egui::Key::ArrowRight,
            glfw::Key::Up => egui::Key::ArrowUp,
            glfw::Key::Escape => egui::Key::Escape,
            glfw::Key::Tab => egui::Key::Tab,
            glfw::Key::Backspace => egui::Key::Backspace,
            glfw::Key::Enter => egui::Key::Enter,
            glfw::Key::Space => egui::Key::Space,
            glfw::Key::Insert => egui::Key::Insert,
            glfw::Key::Delete => egui::Key::Delete,
            glfw::Key::Home => egui::Key::Home,
            glfw::Key::End => egui::Key::End,
            glfw::Key::PageUp => egui::Key::PageUp,
            glfw::Key::PageDown => egui::Key::PageDown,
            glfw::Key::Num0 => egui::Key::Num0,
            glfw::Key::Num1 => egui::Key::Num1,
            glfw::Key::Num2 => egui::Key::Num2,
            glfw::Key::Num3 => egui::Key::Num3,
            glfw::Key::Num4 => egui::Key::Num4,
            glfw::Key::Num5 => egui::Key::Num5,
            glfw::Key::Num6 => egui::Key::Num6,
            glfw::Key::Num7 => egui::Key::Num7,
            glfw::Key::Num8 => egui::Key::Num8,
            glfw::Key::Num9 => egui::Key::Num9,

            _ => return None,
        })
    }

    pub fn handle_event(&mut self, event: &glfw::WindowEvent, window: &glfw::Window) {
        let raw_event = match event {
            glfw::WindowEvent::CursorPos(x, y) => {
                Some(Event::PointerMoved(egui::pos2(*x as _, *y as _)))
            }
            glfw::WindowEvent::MouseButton(button, action, modifier) => {
                let button = Self::button_type(button);
                match button {
                    Some(button) => {
                        let pressed = Self::is_pressed(action);
                        Some(Event::PointerButton {
                            pos: Self::get_cur_pos(window),
                            button,
                            pressed,
                            modifiers: Self::get_modifier(modifier),
                        })
                    }
                    None => None,
                }
            }
            glfw::WindowEvent::Key(key, _scancode, action, modifiers) => {
                let pressed = Self::is_pressed(action);
                match key {
                    Key::LeftShift | Key::RightShift => self.raw_input.modifiers.shift = pressed,
                    Key::LeftControl | Key::RightControl => {
                        self.raw_input.modifiers.ctrl = pressed;
                        self.raw_input.modifiers.command = pressed;
                    }
                    Key::LeftAlt | Key::RightAlt => self.raw_input.modifiers.alt = pressed,
                    _ => (),
                }
                match Self::get_key(key) {
                    Some(key) => Some(Event::Key {
                        key,
                        pressed,
                        modifiers: Self::get_modifier(modifiers),
                    }),
                    None => None,
                }
            }
            glfw::WindowEvent::Scroll(x, y) => {
                self.raw_input.scroll_delta = egui::vec2(*x as _, *y as _);
                None
            }
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe {
                    gl::Viewport(0, 0, *width, *height);
                }
                self.set_screen_rect_from_size(egui::vec2(*width as _, *height as _));
                None
            }
            _ => todo!("handle the event {:?}", event),
        };
        if let Some(raw_event) = raw_event {
            self.raw_input.events.push(raw_event);
        }
    }

    fn set_screen_rect_from_size(&mut self, screen_size: egui::Vec2) {
        // TODO(ish): will need to divide with pixels per point most probably
        self.raw_input.screen_rect =
            Some(egui::Rect::from_min_size(Default::default(), screen_size));
    }

    pub fn set_screen_rect(&mut self, window: &glfw::Window) {
        let screen_size = window.get_framebuffer_size();
        let screen_size = egui::vec2(screen_size.0 as _, screen_size.1 as _);
        self.set_screen_rect_from_size(screen_size);
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
