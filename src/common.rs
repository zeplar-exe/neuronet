pub type Position = (f64, f64);
pub type Voltage = f64;

#[derive(Clone, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Default for Color {
    fn default() -> Self {
        Color {
            red: 255u8,
            green: 255u8,
            blue: 255u8,
            alpha: 255,
        }
    }
}

impl Color {
    pub fn hex(&self) -> String {
        format!("#{:02X?}", [self.red, self.green, self.blue, self.alpha])
    }
}

#[derive(Clone, Default, PartialEq, Debug)]
pub struct Rect {
    pub top: f64,
    pub left: f64,
    pub bottom: f64,
    pub right: f64,
}

impl Rect {
    pub fn from_corners(top_left: Position, bottom_right: Position) -> Rect {
        Rect {
            top: top_left.1,
            left: top_left.0,
            bottom: bottom_right.1,
            right: bottom_right.0,
        }
    }

    pub fn from_dimensions(position: Position, size: Position) -> Rect {
        Rect {
            top: position.1,
            left: position.0,
            bottom: position.1 + size.1,
            right: position.0 + size.0,
        }
    }

    pub fn from_dimensions_f(left: f64, top: f64, width: f64, height: f64) -> Rect {
        Rect {
            left,
            top,
            bottom: top + height,
            right: left + width,
        }
    }

    pub fn area(&self) -> f64 {
        (self.right - self.left) * (self.bottom - self.top)
    }

    pub fn width(&self) -> f64 {
        self.right - self.left
    }

    pub fn height(&self) -> f64 {
        self.bottom - self.top
    }

    pub fn as_tuple(&self) -> (f64, f64, f64, f64) {
        (self.top, self.left, self.bottom, self.right)
    }

    pub fn as_tuple_wh(&self) -> (f64, f64, f64, f64) {
        (self.left, self.top, self.width(), self.height())
    }
}
