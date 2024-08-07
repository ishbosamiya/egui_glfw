use std::{borrow::Cow, convert::TryInto};

/// GPU Texture RGBA8. Each pixel has 4 channels, [`u8`] each.
pub struct TextureRGBA8 {
    /// Width of the texture.
    width: usize,
    /// Height of the texture.
    height: usize,

    /// Pixels of the image stored from bottom left row wise.
    pixels: Vec<(u8, u8, u8, u8)>,

    /// [`egui::TextureOptions`] for the current texture.
    texture_options: egui::TextureOptions,

    /// OpenGL texture ID.
    ///
    /// If [`None`] if the texture is not yet uploaded to the GPU.
    gl_tex: Option<gl::types::GLuint>,
}

impl TextureRGBA8 {
    /// Create a [`TextureRGBA8`] from pixels. The pixels provided
    /// must follow the pixel memory layout mapping of bottom left
    /// left to right rows.
    pub fn from_pixels(
        width: usize,
        height: usize,
        pixels: Vec<(u8, u8, u8, u8)>,
        texture_options: egui::TextureOptions,
    ) -> Self {
        assert_eq!(pixels.len(), width * height);
        Self {
            width,
            height,
            pixels,
            gl_tex: None,
            texture_options,
        }
    }

    /// Create a [`TextureRGBA8`] from an egui ImageDelta.
    pub fn from_egui(delta: &egui::epaint::image::ImageDelta) -> Option<Self> {
        // the delta should be for the whole image, the total image
        // size cannot be determined to update only a portion of the
        // image
        if !delta.is_whole() {
            return None;
        }

        let image = &delta.image;

        // need to flip the image vertically since egui uses top left
        // left to right rows but opengl uses bottom left left to
        // right rows

        Some(Self {
            width: image.width(),
            height: image.height(),
            pixels: match image {
                egui::ImageData::Color(image) => image
                    .pixels
                    .chunks(image.width())
                    .rev()
                    .flat_map(|row| {
                        row.iter()
                            .map(|pixel| (pixel.r(), pixel.g(), pixel.b(), pixel.a()))
                    })
                    .collect(),
                egui::ImageData::Font(image) => image
                    .srgba_pixels(None)
                    .map(|pixel| (pixel.r(), pixel.g(), pixel.b(), pixel.a()))
                    .collect::<Vec<_>>()
                    .chunks(image.width())
                    .rev()
                    .flat_map(|row| row.iter().copied())
                    .collect(),
            },
            texture_options: delta.options,
            gl_tex: None,
        })
    }

    /// Update the texture from egui's ImageDelta.
    pub fn update_from_egui(&mut self, delta: &egui::epaint::image::ImageDelta) {
        // if the whole image has changed, then just replace self
        if delta.is_whole() {
            *self = Self::from_egui(delta).unwrap();
            return;
        }

        // TODO: need to optimize this, shouldn't delete the entire
        // texture from the GPU and resend everything. It does not
        // take advantage of the delta that is provided. Make sure
        // `self.texture_options` match the delta's
        // [`egui::TextureOptions`]. For the texture options, it might
        // make sense to use a texture sampler instead of assigning
        // the sampler options to the texture itself.

        // delete the entire texture from the GPU if it was previously
        // uploaded
        if self.gl_tex.is_some() {
            self.cleanup_opengl();
        }

        let (delta_image_pixels, delta_image_width) = match &delta.image {
            egui::ImageData::Color(image) => {
                (Cow::Borrowed(image.pixels.as_slice()), image.width())
            }
            egui::ImageData::Font(image) => (image.srgba_pixels(None).collect(), image.width()),
        };

        // the position provided will be from top left as (0, 0) but
        // Self requires (0, 0) as bottom left, so need to update by
        // "reversing" (flip vertically) the image during the update
        let start_pos = delta.pos.unwrap();
        self.pixels
            .chunks_mut(self.width)
            .rev()
            .enumerate()
            .skip(start_pos[1])
            .enumerate()
            .map_while(|(y, (row_index, row))| {
                (row_index < (start_pos[1] + delta.image.height())).then_some((row, y))
            })
            .for_each(|(row, y)| {
                row.iter_mut()
                    .enumerate()
                    .skip(start_pos[0])
                    .enumerate()
                    .map_while(|(x, (column_index, pixel))| {
                        (column_index < (start_pos[0] + delta.image.width())).then_some((pixel, x))
                    })
                    .for_each(|(pixel, x)| {
                        let new_pixel = delta_image_pixels[y * delta_image_width + x];
                        *pixel = (new_pixel.r(), new_pixel.g(), new_pixel.b(), new_pixel.a())
                    });
            });
    }

    /// # Safety
    ///
    /// There is no way to generate [`Texture`] without automatically
    /// sending the texture to the GPU except during deserialization
    /// so there is no need to call this function except immediately
    /// after deserialization once. Unless absolutely necessary (like
    /// sending the texture to the GPU early), [`Self::get_gl_tex()`]
    /// should handle it.
    pub unsafe fn send_to_gpu(&mut self) {
        assert!(self.gl_tex.is_none());

        self.gl_tex = Some(Self::gen_gl_texture(&self.texture_options));

        self.new_texture_to_gl();
    }

    pub fn activate(&mut self, texture_target: u8) {
        if self.gl_tex.is_none() {
            unsafe { self.send_to_gpu() };
        }

        let target = match texture_target {
            0 => gl::TEXTURE0,
            1 => gl::TEXTURE1,
            2 => gl::TEXTURE2,
            3 => gl::TEXTURE3,
            4 => gl::TEXTURE4,
            5 => gl::TEXTURE5,
            6 => gl::TEXTURE6,
            7 => gl::TEXTURE7,
            8 => gl::TEXTURE8,
            9 => gl::TEXTURE9,
            10 => gl::TEXTURE10,
            11 => gl::TEXTURE11,
            12 => gl::TEXTURE12,
            13 => gl::TEXTURE13,
            14 => gl::TEXTURE14,
            15 => gl::TEXTURE15,
            16 => gl::TEXTURE16,
            17 => gl::TEXTURE17,
            18 => gl::TEXTURE18,
            19 => gl::TEXTURE19,
            20 => gl::TEXTURE20,
            21 => gl::TEXTURE21,
            22 => gl::TEXTURE22,
            23 => gl::TEXTURE23,
            24 => gl::TEXTURE24,
            25 => gl::TEXTURE25,
            26 => gl::TEXTURE26,
            27 => gl::TEXTURE27,
            28 => gl::TEXTURE28,
            29 => gl::TEXTURE29,
            30 => gl::TEXTURE30,
            31 => gl::TEXTURE31,
            _ => panic!("Texture target not possible, gl support [0, 32)"),
        };
        unsafe {
            gl::ActiveTexture(target);
            gl::BindTexture(gl::TEXTURE_2D, self.gl_tex.unwrap());
        }
    }

    fn new_texture_to_gl(&self) {
        assert_eq!(self.pixels.len(), self.width * self.height);

        let pixel_size = 4;

        // set the row alignment based on the pixel size and the
        // number of rows
        let row_length_in_bytes = pixel_size * self.width;
        let unpack_alignment = if row_length_in_bytes % 8 == 0 {
            8
        } else if row_length_in_bytes % 4 == 0 {
            4
        } else if row_length_in_bytes % 2 == 0 {
            2
        } else {
            1
        };

        unsafe {
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, unpack_alignment);

            gl::BindTexture(gl::TEXTURE_2D, self.gl_tex.unwrap());

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8.try_into().unwrap(),
                self.width.try_into().unwrap(),
                self.height.try_into().unwrap(),
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                self.pixels.as_ptr() as *const gl::types::GLvoid,
            )
        }
    }

    fn gen_gl_texture(texture_options: &egui::TextureOptions) -> gl::types::GLuint {
        let mut gl_tex = 0;
        unsafe {
            gl::GenTextures(1, &mut gl_tex);
        }
        assert_ne!(gl_tex, 0);

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, gl_tex);

            // wrapping method
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                texture_options.wrap_mode.to_gl().try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                texture_options.wrap_mode.to_gl().try_into().unwrap(),
            );

            // filter method
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                texture_options.minification.to_gl().try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                texture_options.magnification.to_gl().try_into().unwrap(),
            );
        }

        gl_tex
    }

    /// Get OpenGL texture name (GLuint) of the current texture, send
    /// texture to GPU if not done so already.
    pub fn get_gl_tex(&mut self) -> gl::types::GLuint {
        if self.gl_tex.is_none() {
            unsafe { self.send_to_gpu() };
        }
        self.gl_tex.unwrap()
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn cleanup_opengl(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.gl_tex.unwrap());
        }
        self.gl_tex = None;
    }
}

impl Drop for TextureRGBA8 {
    fn drop(&mut self) {
        if self.gl_tex.is_some() {
            self.cleanup_opengl();
        }
    }
}

/// Extention to [`egui::TextureFilter`].
pub trait EguiTextureFilterExtend {
    /// Convert to equivalent texture filter parameter to OpenGL.
    fn to_gl(&self) -> gl::types::GLenum;
}

impl EguiTextureFilterExtend for egui::TextureFilter {
    fn to_gl(&self) -> gl::types::GLenum {
        match self {
            egui::TextureFilter::Nearest => gl::NEAREST,
            egui::TextureFilter::Linear => gl::LINEAR,
        }
    }
}

/// Extention to [`egui::TextureWrapMode`].
pub trait EguiTextureWrapModeExtend {
    /// Convert to equivalent texture wrap mode to OpenGL.
    fn to_gl(&self) -> gl::types::GLenum;
}

impl EguiTextureWrapModeExtend for egui::TextureWrapMode {
    fn to_gl(&self) -> gl::types::GLenum {
        match self {
            egui::TextureWrapMode::ClampToEdge => gl::CLAMP_TO_EDGE,
            egui::TextureWrapMode::Repeat => gl::REPEAT,
            egui::TextureWrapMode::MirroredRepeat => gl::MIRRORED_REPEAT,
        }
    }
}
