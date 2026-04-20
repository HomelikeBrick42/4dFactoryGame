use crate::{
    texture::Texture,
    ui::{Frame, Quad},
};
use math::{Vector2, Vector4};
use std::collections::HashMap;

pub struct Font {
    images: Vec<Texture>,
    characters: HashMap<char, Character>,
    line_height: f32,
    base: f32,
}

pub struct Character {
    pub offset: Vector2<f32>,
    pub size: Vector2<f32>,
    pub uv_offset: Vector2<f32>,
    pub uv_size: Vector2<f32>,
    pub advance: f32,
    pub image: usize,
}

impl Font {
    pub fn new(file: &str, images: Vec<Texture>) -> Self {
        fn find_line<'a>(file: &'a str, name: &str) -> Option<&'a str> {
            file.lines().find(|line| line.starts_with(name))
        }

        fn find_usize(mut line: &str, name: &str) -> Option<usize> {
            line = &line[line.find(name)? + name.len()..];
            line = &line[..line
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(line.len())];
            line.parse().ok()
        }

        fn find_isize(mut line: &str, name: &str) -> Option<isize> {
            line = &line[line.find(name)? + name.len()..];
            line = &line[..line
                .find(|c: char| c != '-' && !c.is_ascii_digit())
                .unwrap_or(line.len())];
            line.parse().ok()
        }

        let info_line = find_line(file, "info ").unwrap();
        let size = find_usize(info_line, "size=").unwrap();
        let unicode = find_usize(info_line, "unicode=").unwrap();
        assert_eq!(unicode, 1);
        let stretch = find_usize(info_line, "stretchH=").unwrap();
        assert_eq!(stretch, 100);

        let common_line = find_line(file, "common ").unwrap();
        let line_height = find_usize(common_line, "lineHeight=").unwrap();
        let base = find_usize(common_line, "base=").unwrap();
        let scale_width = find_usize(common_line, "scaleW=").unwrap();
        let scale_height = find_usize(common_line, "scaleH=").unwrap();
        let page_count = find_usize(common_line, "pages=").unwrap();
        assert_eq!(page_count, images.len());

        let chars_line = find_line(file, "chars ").unwrap();
        let char_count = find_usize(chars_line, "count=").unwrap();

        let mut characters = HashMap::with_capacity(char_count);
        for char_line in file.lines().filter(|line| line.starts_with("char ")) {
            let id = find_usize(char_line, "id=").unwrap();
            let x = find_usize(char_line, "x=").unwrap();
            let y = find_usize(char_line, "y=").unwrap();
            let width = find_usize(char_line, "width=").unwrap();
            let height = find_usize(char_line, "height=").unwrap();
            let xoffset = find_isize(char_line, "xoffset=").unwrap();
            let yoffset = find_isize(char_line, "yoffset=").unwrap();
            let advance = find_usize(char_line, "xadvance=").unwrap();
            let page = find_usize(char_line, "page=").unwrap();

            characters.insert(
                id.try_into()
                    .ok()
                    .and_then(char::from_u32)
                    .expect("the character id should be a valid char"),
                Character {
                    offset: Vector2 {
                        x: xoffset as f32 / size as f32,
                        y: -(yoffset as f32 / size as f32),
                    },
                    size: Vector2 {
                        x: width as f32 / size as f32,
                        y: -(height as f32 / size as f32),
                    },
                    uv_offset: Vector2 {
                        x: x as f32 / scale_width as f32,
                        y: y as f32 / scale_height as f32,
                    },
                    uv_size: Vector2 {
                        x: width as f32 / scale_width as f32,
                        y: height as f32 / scale_height as f32,
                    },
                    advance: advance as f32 / size as f32,
                    image: page,
                },
            );
        }

        Self {
            images,
            characters,
            line_height: line_height as f32 / size as f32,
            base: base as f32 / size as f32,
        }
    }

    pub fn line_height(&self) -> f32 {
        self.line_height
    }

    pub fn base(&self) -> f32 {
        self.base
    }

    pub fn draw_char(
        &self,
        frame: &mut Frame<'_>,
        cursor: &mut Vector2<f32>,
        char_height: f32,
        color: Vector4<f32>,
        c: char,
    ) {
        let Some(c) = self.characters.get(&c) else {
            return;
        };

        frame.push_quad(
            Quad {
                position: *cursor
                    + Vector2 {
                        x: 0.0,
                        y: self.base * char_height,
                    }
                    + c.offset * char_height
                    + c.size * char_height * 0.5,
                size: c.size * char_height,
                uv_offset: c.uv_offset,
                uv_size: c.uv_size,
                color,
            },
            self.images.get(c.image),
        );
        cursor.x += c.advance * char_height;
    }

    pub fn draw_str(
        &self,
        frame: &mut Frame<'_>,
        cursor: &mut Vector2<f32>,
        char_height: f32,
        color: Vector4<f32>,
        s: &str,
    ) {
        for c in s.chars() {
            self.draw_char(frame, cursor, char_height, color, c);
        }
    }
}
