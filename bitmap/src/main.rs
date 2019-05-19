use std::fs::File;
use std::io::{BufWriter, Result, Write};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
}

pub struct Bitmap {
    width: usize,
    height: usize,
    image: Vec<Color>,
}

impl Bitmap {
    pub fn new(width: usize, height: usize) -> Self {
        Bitmap {
            width,
            height,
            image: vec![Color::new(0, 0, 0); width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn fill(&mut self, c: Color) {
        for pixel in &mut self.image {
            *pixel = c;
        }
    }

    pub fn color(&self, x: usize, y: usize) -> &Color {
        let idx = self.width * y + x;
        &self.image[idx]
    }

    pub fn color_mut(&mut self, x: usize, y: usize) -> &mut Color {
        let idx = self.width * y + x;
        &mut self.image[idx]
    }

    pub fn write_ppm(&self, out: &mut Write) -> Result<()> {
        let header = format!("P6\n{} {}\n255\n", self.width(), self.height());
        out.write(header.as_bytes())?;

        for c in &self.image {
            out.write(&[c.r, c.g, c.b])?;
        }

        Ok(())
    }
}

fn print_usage(name: &str) {
    println!("Usage: {} filename", name);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        debug_assert!(args.len() == 1);
        print_usage(&args[0]);
        return;
    }

    let f = File::create(&args[1]).expect(&format!("{} cannot be created.", args[1]));
    let mut f = BufWriter::new(f);

    let mut bmp = Bitmap::new(300, 400);

    bmp.fill(Color::new(128, 64, 255));
    for x in 10..100 {
        *bmp.color_mut(x, 20) = Color::new(255, 255, 255);
    }

    bmp.write_ppm(&mut f).expect("write PPM failed.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill() {
        let mut bmp = Bitmap::new(10, 20);
        bmp.fill(Color::new(0, 128, 255));

        assert_eq!(Color::new(0, 128, 255), *bmp.color(0, 0));
        assert_eq!(Color::new(0, 128, 255), *bmp.color(5, 10));
        assert_eq!(Color::new(0, 128, 255), *bmp.color(9, 19));
    }

    #[test]
    fn test_set() {
        let mut bmp = Bitmap::new(10, 20);
        *bmp.color_mut(2, 3) = Color::new(255, 128, 0);

        assert_eq!(Color::new(0, 0, 0), *bmp.color(0, 0));
        assert_eq!(Color::new(255, 128, 0), *bmp.color(2, 3));
    }
}
