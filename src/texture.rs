use std::convert::TryInto;

/// GPU Texture RGBA8. Each pixel has 4 channels, [`u8`] each.
pub struct TextureRGBA8 {
    width: usize,
    height: usize,

    /// pixels of the image stored from bottom left row wise
    pixels: Vec<(u8, u8, u8, u8)>,

    gl_tex: Option<gl::types::GLuint>,
}

impl TextureRGBA8 {
    /// Create a [`TextureRGBA8`] from pixels. The pixels provided
    /// must follow the pixel memory layout mapping of bottom left
    /// left to right rows.
    pub fn from_pixels(width: usize, height: usize, pixels: Vec<(u8, u8, u8, u8)>) -> Self {
        assert_eq!(pixels.len(), width * height);
        Self {
            width,
            height,
            pixels,
            gl_tex: None,
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
                    .srgba_pixels(1.0)
                    .map(|pixel| (pixel.r(), pixel.g(), pixel.b(), pixel.a()))
                    .collect::<Vec<_>>()
                    .chunks(image.width())
                    .rev()
                    .flat_map(|row| row.iter().copied())
                    .collect(),
            },
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

        let gamma = 1.0;

        // TODO: need to optimize this, shouldn't delete the entire
        // texture from the GPU and resend everything. It does not
        // take advantage of the delta that is provided.

        // delete the entire texture from the GPU if it was previously
        // uploaded
        if self.gl_tex.is_some() {
            self.cleanup_opengl();
        }

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
                (row_index < (start_pos[1] + delta.image.height())).then(|| (row, y))
            })
            .for_each(|(row, y)| {
                row.iter_mut()
                    .enumerate()
                    .skip(start_pos[0])
                    .enumerate()
                    .map_while(|(x, (column_index, pixel))| {
                        (column_index < (start_pos[0] + delta.image.width())).then(|| (pixel, x))
                    })
                    .for_each(|(pixel, x)| {
                        match &delta.image {
                            egui::ImageData::Color(image) => {
                                let new_pixel = image.pixels[y * image.width() + x];
                                *pixel =
                                    (new_pixel.r(), new_pixel.g(), new_pixel.b(), new_pixel.a())
                            }
                            egui::ImageData::Font(image) => {
                                let new_pixel = image.pixels[y * image.width() + x];

                                // directly from srgba_pixel()
                                //
                                // This is arbitrarily chosen to make text look as good as possible.
                                // In particular, it looks good with gamma=1 and the default eframe backend,
                                // which uses linear blending.
                                // See https://github.com/emilk/egui/issues/1410
                                let a = fast_round(new_pixel.powf(gamma / 2.2) * 255.0);

                                *pixel = (a, a, a, a)
                            }
                        }
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

        self.gl_tex = Some(Self::gen_gl_texture());

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

        let pixel_size = 4 * 1;

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

    fn gen_gl_texture() -> gl::types::GLuint {
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
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE.try_into().unwrap(),
            );

            // filter method
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR.try_into().unwrap(),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR.try_into().unwrap(),
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

/// Fast round from egui.
fn fast_round(r: f32) -> u8 {
    (r + 0.5).floor() as _ // rust does a saturating cast since 1.45
}
