use egui::{Event, Pos2, RawInput};
use glfw::{Key, MouseButton};

pub struct Input {
    raw_input: RawInput,
}

impl Input {
    /// Create a new [`Input`] with the given pixels per point.
    pub fn new(pixels_per_point: f32) -> Self {
        let mut raw_input = RawInput::default();
        raw_input
            .viewports
            .values_mut()
            .for_each(|viewport| viewport.native_pixels_per_point = Some(pixels_per_point));
        Self { raw_input }
    }

    /// Set the pixels per point.
    pub fn set_pixels_per_point(&mut self, pixels_per_point: f32) {
        self.raw_input.viewports.values_mut().for_each(|viewport| {
            viewport.native_pixels_per_point = Some(pixels_per_point);
        });
    }

    /// Refer to egui's RawInput for details on take
    pub fn take(&mut self) -> RawInput {
        self.raw_input.take()
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
    fn get_cur_pos(window: &glfw::Window, pixels_per_point: f32) -> Pos2 {
        let pos = window.get_cursor_pos();
        egui::pos2(
            pos.0 as f32 / pixels_per_point,
            pos.1 as f32 / pixels_per_point,
        )
    }

    /// Get the corresponding [`egui::Key`] for the [`glfw::Key`].
    fn get_key(key: &glfw::Key) -> Option<egui::Key> {
        Some(match key {
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

            // egui::Key::Copy,
            // egui::Key::Cut,
            // egui::Key::Paste,

            // egui::Key::Colon,
            glfw::Key::Comma => egui::Key::Comma,
            glfw::Key::Backslash => egui::Key::Backslash,
            glfw::Key::Slash => egui::Key::Slash,
            // egui::Key::Pipe
            // egui::Key::Questionmark,
            glfw::Key::LeftBracket => egui::Key::OpenBracket,
            glfw::Key::RightBracket => egui::Key::CloseBracket,
            // egui::Key::Backtick,
            glfw::Key::Minus => egui::Key::Minus,
            glfw::Key::Period => egui::Key::Period,
            // egui::Key::Plus,
            glfw::Key::Equal => egui::Key::Equals,
            glfw::Key::Semicolon => egui::Key::Semicolon,

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

            glfw::Key::F1 => egui::Key::F1,
            glfw::Key::F2 => egui::Key::F2,
            glfw::Key::F3 => egui::Key::F3,
            glfw::Key::F4 => egui::Key::F4,
            glfw::Key::F5 => egui::Key::F5,
            glfw::Key::F6 => egui::Key::F6,
            glfw::Key::F7 => egui::Key::F7,
            glfw::Key::F8 => egui::Key::F8,
            glfw::Key::F9 => egui::Key::F9,
            glfw::Key::F10 => egui::Key::F10,
            glfw::Key::F11 => egui::Key::F11,
            glfw::Key::F12 => egui::Key::F12,
            glfw::Key::F13 => egui::Key::F13,
            glfw::Key::F14 => egui::Key::F14,
            glfw::Key::F15 => egui::Key::F15,
            glfw::Key::F16 => egui::Key::F16,
            glfw::Key::F17 => egui::Key::F17,
            glfw::Key::F18 => egui::Key::F18,
            glfw::Key::F19 => egui::Key::F19,
            glfw::Key::F20 => egui::Key::F20,

            _ => return None,
        })
    }

    /// Get the corresponding physical [`egui::Key`] for the
    /// [`glfw::Scancode`].
    fn get_physical_key(_scan_code: &glfw::Scancode) -> Option<egui::Key> {
        // TODO: need to figure out what scancode correspondings to
        // what key
        None
    }

    pub fn handle_event(
        &mut self,
        event: &glfw::WindowEvent,
        window: &glfw::Window,
        pixels_per_point: f32,
    ) {
        let raw_event = match event {
            glfw::WindowEvent::CursorPos(x, y) => Some(Event::PointerMoved(egui::pos2(
                *x as f32 / pixels_per_point,
                *y as f32 / pixels_per_point,
            ))),
            glfw::WindowEvent::MouseButton(button, action, modifier) => {
                let button = Self::button_type(button);
                match button {
                    Some(button) => {
                        let pressed = Self::is_pressed(action);
                        Some(Event::PointerButton {
                            pos: Self::get_cur_pos(window, pixels_per_point),
                            button,
                            pressed,
                            modifiers: Self::get_modifier(modifier),
                        })
                    }
                    None => None,
                }
            }
            glfw::WindowEvent::Key(key, scancode, action, modifiers) => {
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
                Self::get_key(key).map(|key| Event::Key {
                    key,
                    physical_key: Self::get_physical_key(scancode),
                    pressed,
                    repeat: false,
                    modifiers: Self::get_modifier(modifiers),
                })
            }
            glfw::WindowEvent::Scroll(x, y) => {
                let multiplier = 50.0;
                let scroll = multiplier * egui::vec2(*x as _, *y as _);
                if Self::is_pressed(&window.get_key(glfw::Key::LeftControl))
                    || Self::is_pressed(&window.get_key(glfw::Key::RightControl))
                {
                    let factor = (scroll.y / 200.0).exp();
                    Some(Event::Zoom(factor as _))
                } else {
                    Some(Event::Scroll(scroll))
                }
            }
            glfw::WindowEvent::FramebufferSize(width, height) => {
                unsafe {
                    gl::Viewport(0, 0, *width, *height);
                }
                self.set_screen_rect_from_size(
                    egui::vec2(*width as _, *height as _),
                    pixels_per_point,
                );
                None
            }
            glfw::WindowEvent::CursorEnter(enter) => {
                if !enter {
                    Some(Event::PointerGone)
                } else {
                    None
                }
            }
            glfw::WindowEvent::Char(c) => Some(Event::Text(c.to_string())),
            glfw::WindowEvent::Pos(_, _) => None,
            glfw::WindowEvent::Refresh => None,
            glfw::WindowEvent::Focus(_) => None,
            glfw::WindowEvent::CharModifiers(_, _) => None,
            glfw::WindowEvent::Close => None,
            glfw::WindowEvent::Size(_, _) => None,
            glfw::WindowEvent::Iconify(_) => None,
            glfw::WindowEvent::FileDrop(paths) => {
                self.raw_input
                    .dropped_files
                    .extend(paths.iter().map(|path| egui::DroppedFile {
                        path: Some(path.to_path_buf()),
                        ..Default::default()
                    }));
                None
            }
            glfw::WindowEvent::Maximize(_) => None,
            glfw::WindowEvent::ContentScale(x, _y) => {
                // taking the x scale because egui supports only one
                // value
                self.set_pixels_per_point(*x);
                None
            }
        };
        if let Some(raw_event) = raw_event {
            self.raw_input.events.push(raw_event);
        }
    }

    /// Push a [`egui::Event`] to egui. This is useful when a certain
    /// event is not handled yet or it is not possible to handle an
    /// event due to discrepancies in what shortcut to use. An example
    /// of this is [`egui::Event::Copy`], the user may want a shortcut
    /// that is not `C-c`.
    pub fn push_event(&mut self, event: egui::Event) {
        self.raw_input.events.push(event);
    }

    /// Set the screen rect from the given screen size in pixels.
    fn set_screen_rect_from_size(
        &mut self,
        screen_size_in_pixels: egui::Vec2,
        pixels_per_point: f32,
    ) {
        self.raw_input.screen_rect = Some(egui::Rect::from_min_size(
            Default::default(),
            screen_size_in_pixels / pixels_per_point,
        ));
    }

    /// Set the screen rect from the given window and pixels per
    /// point.
    pub fn set_screen_rect(&mut self, window: &glfw::Window, pixels_per_point: f32) {
        let screen_size_in_pixels = window.get_framebuffer_size();
        let screen_size_in_pixels =
            egui::vec2(screen_size_in_pixels.0 as _, screen_size_in_pixels.1 as _);
        self.set_screen_rect_from_size(screen_size_in_pixels, pixels_per_point);
    }

    /// Get the internal raw input state mutably.
    ///
    /// # Safety
    ///
    /// This is basically never required unless something isn't
    /// supported. Need to be careful about how it is used. There
    /// cannot be any crashes but can cause the GUI to mess up if used
    /// wrong.
    pub unsafe fn get_raw_input(&mut self) -> &mut RawInput {
        &mut self.raw_input
    }
}
