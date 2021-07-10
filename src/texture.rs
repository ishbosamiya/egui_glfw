use std::convert::TryInto;

use gl::types::{GLuint, GLvoid};

pub struct Texture {
    version: u64,
    width: usize,
    height: usize,
    pixels: Vec<(u8, u8, u8, u8)>,

    gl_tex: GLuint,
}

impl Texture {
    pub fn from_egui(tex: &egui::Texture) -> Self {
        let gl_tex = Self::gen_gl_texture();
        let res = Self {
            version: tex.version,
            width: tex.size()[0],
            height: tex.size()[1],
            pixels: tex
                .pixels
                .iter()
                .map(|&a| egui::Color32::from_white_alpha(a).to_tuple())
                .collect(),
            gl_tex,
        };

        assert_eq!(res.pixels.len(), res.width * res.height);

        res.new_texture_to_gl();

        res
    }

    pub fn update_from_egui(&mut self, tex: &egui::Texture) {
        if self.version == tex.version {
            return;
        }

        self.version = tex.version;
        self.width = tex.width;
        self.height = tex.height;
        self.pixels = tex
            .pixels
            .iter()
            .map(|&a| egui::Color32::from_white_alpha(a).to_tuple())
            .collect();

        self.new_texture_to_gl();
    }

    pub fn activate(&mut self, texture_target: u8) {
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
            gl::BindTexture(gl::TEXTURE_2D, self.gl_tex);
        }
    }

    fn new_texture_to_gl(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.gl_tex);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA.try_into().unwrap(),
                self.width.try_into().unwrap(),
                self.height.try_into().unwrap(),
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                self.pixels.as_ptr() as *const GLvoid,
            )
        }
    }

    fn gen_gl_texture() -> GLuint {
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
}
