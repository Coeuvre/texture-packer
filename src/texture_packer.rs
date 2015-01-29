use std::collections::HashMap;

use {
    TexturePackerAlrogithm,
    TexturePackerConfig,
};

use texture::{
    Pixel,
    Texture,
};

use frame::Frame;
use packer::{
    Packer,
    SkylinePacker,
};

pub struct TexturePacker<'a, P: Pixel> {
    textures: HashMap<String, Box<Texture<Pixel = P> + 'a>>,
    frames: Vec<Frame>,
    packer: Box<Packer<Pixel = P> + 'a>,
}

impl<'a, P: Pixel> TexturePacker<'a, P> {
    pub fn new(config: TexturePackerConfig) -> TexturePacker<'a, P> {
        let packer = match config.algorithm {
            TexturePackerAlrogithm::Skyline => {
                Box::new(SkylinePacker::new(config))
            }
        };

        TexturePacker {
            textures: HashMap::new(),
            frames: Vec::new(),
            packer: packer,
        }
    }

    pub fn pack(&mut self, key: String, texture: Box<Texture<Pixel = P> + 'a>) {
        if let Some(frame) = self.packer.pack(key.clone(), &*texture) {
            self.frames.push(frame);
        }

        self.textures.insert(key, texture);
    }

    fn get_frame_at(&self, x: u32, y: u32) -> Option<&Frame> {
        for frame in self.frames.iter() {
            if frame.frame.contains_point(x, y) {
                return Some(frame);
            }
        }
        None
    }
}

impl<'a, P: Pixel> Texture for TexturePacker<'a, P> {
    type Pixel = P;

    fn width(&self) -> u32 {
        let mut right = 0;

        for frame in self.frames.iter() {
            if frame.frame.right() > right {
                right = frame.frame.right();
            }
        }

        right + 1
    }

    fn height(&self) -> u32 {
        let mut bottom = 0;

        for frame in self.frames.iter() {
            if frame.frame.bottom() > bottom {
                bottom = frame.frame.bottom();
            }
        }

        bottom + 1
    }

    fn get(&self, x: u32, y: u32) -> Option<P> {
        if let Some(frame) = self.get_frame_at(x, y) {
           if let Some(texture) = self.textures.get(&frame.key) {
                let x = x - frame.frame.x;
                let y = y - frame.frame.y;
                return if frame.rotated {
                    texture.get_rotated(x, y)
                } else {
                    texture.get(x, y)
                };

           }
        }

        None
    }

    fn set(&mut self, _x: u32, _y: u32, _val: P) {
        panic!("Can't set pixel directly");
    }
}