use std;
use std::cmp::max;

use {
    TexturePackerConfig,
    Rect,
    Frame,
};

use packer::Packer;
use texture::{
    Pixel,
    Texture,
};

struct Skyline {
    pub x: u32,
    pub y: u32,
    pub w: u32,
}

pub struct SkylinePacker<P: Pixel> {
    config: TexturePackerConfig,
    skylines: Vec<Skyline>,
}

impl<P: Pixel> SkylinePacker<P> {
    pub fn new(config: TexturePackerConfig) -> SkylinePacker<P> {
        let mut skylines = Vec::new();
        skylines.push(Skyline {
            x: 0,
            y: 0,
            w: config.max_width,
        });

        SkylinePacker {
            config: config,
            skylines: skylines,
        }
    }

    fn can_put(&self, i: usize, w: u32, h: u32) -> Option<u32> {
        let x = self.skylines[i].x;
        if x + w > self.config.max_width {
            return None;
        }
        let mut width_left = w;
        let mut i = i;
        let mut y = self.skylines[i].y;
        loop {
            y = max(y, self.skylines[i].y);
            if y + h > self.config.max_height {
                return None;
            }
            if self.skylines[i].w >= width_left {
                return Some(y);
            }
            width_left -= self.skylines[i].w;
            i += 1;
            assert!(i < self.skylines.len());
        }
    }

    fn find_skyline(&self, w: u32, h: u32) -> Option<(usize, Rect)> {
        let mut min_height = std::u32::MAX;
        let mut min_width = std::u32::MAX;
        let mut index = None;
        let mut rect = Rect::new(0, 0, 0, 0);

        // keep the min_height as small as possible
        for i in 0..self.skylines.len() {
            if let Some(y) = self.can_put(i, w, h) {
                if y + h < min_height ||
                   (y + h == min_height && self.skylines[i].w < min_width) {
                    min_height = y + h;
                    min_width = self.skylines[i].w;
                    index = Some(i);
                    rect.x = self.skylines[i].x;
                    rect.y = y;
                    rect.w = w;
                    rect.h = h;
                }
            }

            if self.config.allow_rotation {
                if let Some(y) = self.can_put(i, h, w) {
                    if y + w < min_height ||
                       (y + w == min_height && self.skylines[i].w < min_width) {
                        min_height = y + w;
                        min_width = self.skylines[i].w;
                        index = Some(i);
                        rect.x = self.skylines[i].x;
                        rect.y = y;
                        rect.w = h;
                        rect.h = w;
                    }
                }
            }
        }

        if let Some(index) = index {
            Some((index, rect))
        } else {
            None
        }
    }

    fn split(&mut self, index: usize, rect: &Rect) {
        let skyline = Skyline {
            x: rect.x,
            y: rect.y + rect.h,
            w: rect.w,
        };

        assert!(skyline.x + skyline.w <= self.config.max_width);
        assert!(skyline.y <= self.config.max_height);

        self.skylines.insert(index, skyline);

        let i = index + 1;
        while i < self.skylines.len() {
            assert!(self.skylines[i-1].x <= self.skylines[i].x);

            if self.skylines[i].x < self.skylines[i-1].x + self.skylines[i-1].w {
                let shrink = self.skylines[i-1].x + self.skylines[i-1].w - self.skylines[i].x;
                if self.skylines[i].w <= shrink {
                    self.skylines.remove(i);
                } else {
                    self.skylines[i].x += shrink;
                    self.skylines[i].w -= shrink;
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn merge(&mut self) {
        let mut i = 1;
        while i < self.skylines.len() {
            if self.skylines[i-1].y == self.skylines[i].y {
                self.skylines[i-1].w += self.skylines[i].w;
                self.skylines.remove(i);
                i -= 1;
            }
            i += 1;
        }
    }
}

impl<P: Pixel> Packer for SkylinePacker<P> {
    type Pixel = P;

    fn pack(&mut self, key: String, texture: &Texture<Pixel=P>) -> Option<Frame> {
        let width = texture.width();
        let height = texture.height();

        match self.find_skyline(width, height) {
            Some((i, rect)) => {
                self.split(i, &rect);
                self.merge();

                Some(Frame {
                    key: key,
                    frame: rect,
                    rotated: width != rect.w,
                    trimmed: false,
                    source: Rect {
                        x: 0,
                        y: 0,
                        w: texture.width(),
                        h: texture.height(),
                    },
                })
            },
            _ => {
                None
            },
        }
    }
}