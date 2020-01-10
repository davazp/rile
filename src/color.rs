//! Module to work with colors
//!

/// A color with `r` (red), `g` (green) and `b` (blue) components.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Build a color from the red, green and blue components.
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }

    /// Return a color by name by it exists.
    pub fn from_name(name: &str) -> Option<Color> {
        COLORS_256.iter().find(|e| e.name == name).map(|e| e.color)
    }

    pub fn name_from_code(n: u8) -> &'static str {
        COLORS_256[n as usize].name
    }

    fn distance_square(color1: Color, color2: Color) -> i32 {
        let dr = color1.r as i32 - color2.r as i32;
        let dg = color1.g as i32 - color2.g as i32;
        let db = color1.b as i32 - color2.b as i32;
        dr * dr + dg * dg + db * db
    }

    /// Return a system color that better approximates this color.
    pub fn to_256_code(&self) -> u8 {
        let (code, _color) = COLORS_256
            .iter()
            .enumerate()
            .min_by_key(|(_, c)| Color::distance_square(c.color, *self))
            .unwrap();
        code as u8
    }
}

struct ColorEntry {
    name: &'static str,
    color: Color,
}

const fn entry(name: &'static str, (r, g, b): (u8, u8, u8)) -> ColorEntry {
    ColorEntry {
        name,
        color: Color { r, g, b },
    }
}

#[rustfmt::skip]
const COLORS_256: &[ColorEntry; 256] = &[
    entry("Black",                              (0, 0, 0)),
    entry("Maroon",                             (128, 0, 0)),
    entry("Green",                              (0, 128, 0)),
    entry("Olive",                              (128, 128, 0)),
    entry("Navy",                               (0, 0, 128)),
    entry("Purple",                             (128, 0, 128)),
    entry("Teal",                               (0, 128, 128)),
    entry("Silver",                             (192, 192, 192)),
    entry("Grey",                               (128, 128, 128)),
    entry("Red",                                (255, 0, 0)),
    entry("Lime",                               (0, 255, 0)),
    entry("Yellow",                             (255, 255, 0)),
    entry("Blue",                               (0, 0, 255)),
    entry("Fuchsia",                            (255, 0, 255)),
    entry("Aqua",                               (0, 255, 255)),
    entry("White",                              (255, 255, 255)),
    entry("Grey0",                              (0, 0, 0)),
    entry("NavyBlue",                           (0, 0, 95)),
    entry("DarkBlue",                           (0, 0, 135)),
    entry("Blue3",                              (0, 0, 175)),
    entry("Blue3",                              (0, 0, 215)),
    entry("Blue1",                              (0, 0, 255)),
    entry("DarkGreen",                          (0, 95, 0)),
    entry("DeepSkyBlue4",                       (0, 95, 95)),
    entry("DeepSkyBlue4",                       (0, 95, 135)),
    entry("DeepSkyBlue4",                       (0, 95, 175)),
    entry("DodgerBlue3",                        (0, 95, 215)),
    entry("DodgerBlue2",                        (0, 95, 255)),
    entry("Green4",                             (0, 135, 0)),
    entry("SpringGreen4",                       (0, 135, 95)),
    entry("Turquoise4",                         (0, 135, 135)),
    entry("DeepSkyBlue3",                       (0, 135, 175)),
    entry("DeepSkyBlue3",                       (0, 135, 215)),
    entry("DodgerBlue1",                        (0, 135, 255)),
    entry("Green3",                             (0, 175, 0)),
    entry("SpringGreen3",                       (0, 175, 95)),
    entry("DarkCyan",                           (0, 175, 135)),
    entry("LightSeaGreen",                      (0, 175, 175)),
    entry("DeepSkyBlue2",                       (0, 175, 215)),
    entry("DeepSkyBlue1",                       (0, 175, 255)),
    entry("Green3",                             (0, 215, 0)),
    entry("SpringGreen3",                       (0, 215, 95)),
    entry("SpringGreen2",                       (0, 215, 135)),
    entry("Cyan3",                              (0, 215, 175)),
    entry("DarkTurquoise",                      (0, 215, 215)),
    entry("Turquoise2",                         (0, 215, 255)),
    entry("Green1",                             (0, 255, 0)),
    entry("SpringGreen2",                       (0, 255, 95)),
    entry("SpringGreen1",                       (0, 255, 135)),
    entry("MediumSpringGreen",                  (0, 255, 175)),
    entry("Cyan2",                              (0, 255, 215)),
    entry("Cyan1",                              (0, 255, 255)),
    entry("DarkRed",                            (95, 0, 0)),
    entry("DeepPink4",                          (95, 0, 95)),
    entry("Purple4",                            (95, 0, 135)),
    entry("Purple4",                            (95, 0, 175)),
    entry("Purple3",                            (95, 0, 215)),
    entry("BlueViolet",                         (95, 0, 255)),
    entry("Orange4",                            (95, 95, 0)),
    entry("Grey37",                             (95, 95, 95)),
    entry("MediumPurple4",                      (95, 95, 135)),
    entry("SlateBlue3",                         (95, 95, 175)),
    entry("SlateBlue3",                         (95, 95, 215)),
    entry("RoyalBlue1",                         (95, 95, 255)),
    entry("Chartreuse4",                        (95, 135, 0)),
    entry("DarkSeaGreen4",                      (95, 135, 95)),
    entry("PaleTurquoise4",                     (95, 135, 135)),
    entry("SteelBlue",                          (95, 135, 175)),
    entry("SteelBlue3",                         (95, 135, 215)),
    entry("CornflowerBlue",                     (95, 135, 255)),
    entry("Chartreuse3",                        (95, 175, 0)),
    entry("DarkSeaGreen4",                      (95, 175, 95)),
    entry("CadetBlue",                          (95, 175, 135)),
    entry("CadetBlue",                          (95, 175, 175)),
    entry("SkyBlue3",                           (95, 175, 215)),
    entry("SteelBlue1",                         (95, 175, 255)),
    entry("Chartreuse3",                        (95, 215, 0)),
    entry("PaleGreen3",                         (95, 215, 95)),
    entry("SeaGreen3",                          (95, 215, 135)),
    entry("Aquamarine3",                        (95, 215, 175)),
    entry("MediumTurquoise",                    (95, 215, 215)),
    entry("SteelBlue1",                         (95, 215, 255)),
    entry("Chartreuse2",                        (95, 255, 0)),
    entry("SeaGreen2",                          (95, 255, 95)),
    entry("SeaGreen1",                          (95, 255, 135)),
    entry("SeaGreen1",                          (95, 255, 175)),
    entry("Aquamarine1",                        (95, 255, 215)),
    entry("DarkSlateGray2",                     (95, 255, 255)),
    entry("DarkRed",                            (135, 0, 0)),
    entry("DeepPink4",                          (135, 0, 95)),
    entry("DarkMagenta",                        (135, 0, 135)),
    entry("DarkMagenta",                        (135, 0, 175)),
    entry("DarkViolet",                         (135, 0, 215)),
    entry("Purple",                             (135, 0, 255)),
    entry("Orange4",                            (135, 95, 0)),
    entry("LightPink4",                         (135, 95, 95)),
    entry("Plum4",                              (135, 95, 135)),
    entry("MediumPurple3",                      (135, 95, 175)),
    entry("MediumPurple3",                      (135, 95, 215)),
    entry("SlateBlue1",                         (135, 95, 255)),
    entry("Yellow4",                            (135, 135, 0)),
    entry("Wheat4",                             (135, 135, 95)),
    entry("Grey53",                             (135, 135, 135)),
    entry("LightSlateGrey",                     (135, 135, 175)),
    entry("MediumPurple",                       (135, 135, 215)),
    entry("LightSlateBlue",                     (135, 135, 255)),
    entry("Yellow4",                            (135, 175, 0)),
    entry("DarkOliveGreen3",                    (135, 175, 95)),
    entry("DarkSeaGreen",                       (135, 175, 135)),
    entry("LightSkyBlue3",                      (135, 175, 175)),
    entry("LightSkyBlue3",                      (135, 175, 215)),
    entry("SkyBlue2",                           (135, 175, 255)),
    entry("Chartreuse2",                        (135, 215, 0)),
    entry("DarkOliveGreen3",                    (135, 215, 95)),
    entry("PaleGreen3",                         (135, 215, 135)),
    entry("DarkSeaGreen3",                      (135, 215, 175)),
    entry("DarkSlateGray3",                     (135, 215, 215)),
    entry("SkyBlue1",                           (135, 215, 255)),
    entry("Chartreuse1",                        (135, 255, 0)),
    entry("LightGreen",                         (135, 255, 95)),
    entry("LightGreen",                         (135, 255, 135)),
    entry("PaleGreen1",                         (135, 255, 175)),
    entry("Aquamarine1",                        (135, 255, 215)),
    entry("DarkSlateGray1",                     (135, 255, 255)),
    entry("Red3",                               (175, 0, 0)),
    entry("DeepPink4",                          (175, 0, 95)),
    entry("MediumVioletRed",                    (175, 0, 135)),
    entry("Magenta3",                           (175, 0, 175)),
    entry("DarkViolet",                         (175, 0, 215)),
    entry("Purple",                             (175, 0, 255)),
    entry("DarkOrange3",                        (175, 95, 0)),
    entry("IndianRed",                          (175, 95, 95)),
    entry("HotPink3",                           (175, 95, 135)),
    entry("MediumOrchid3",                      (175, 95, 175)),
    entry("MediumOrchid",                       (175, 95, 215)),
    entry("MediumPurple2",                      (175, 95, 255)),
    entry("DarkGoldenrod",                      (175, 135, 0)),
    entry("LightSalmon3",                       (175, 135, 95)),
    entry("RosyBrown",                          (175, 135, 135)),
    entry("Grey63",                             (175, 135, 175)),
    entry("MediumPurple2",                      (175, 135, 215)),
    entry("MediumPurple1",                      (175, 135, 255)),
    entry("Gold3",                              (175, 175, 0)),
    entry("DarkKhaki",                          (175, 175, 95)),
    entry("NavajoWhite3",                       (175, 175, 135)),
    entry("Grey69",                             (175, 175, 175)),
    entry("LightSteelBlue3",                    (175, 175, 215)),
    entry("LightSteelBlue",                     (175, 175, 255)),
    entry("Yellow3",                            (175, 215, 0)),
    entry("DarkOliveGreen3",                    (175, 215, 95)),
    entry("DarkSeaGreen3",                      (175, 215, 135)),
    entry("DarkSeaGreen2",                      (175, 215, 175)),
    entry("LightCyan3",                         (175, 215, 215)),
    entry("LightSkyBlue1",                      (175, 215, 255)),
    entry("GreenYellow",                        (175, 255, 0)),
    entry("DarkOliveGreen2",                    (175, 255, 95)),
    entry("PaleGreen1",                         (175, 255, 135)),
    entry("DarkSeaGreen2",                      (175, 255, 175)),
    entry("DarkSeaGreen1",                      (175, 255, 215)),
    entry("PaleTurquoise1",                     (175, 255, 255)),
    entry("Red3",                               (215, 0, 0)),
    entry("DeepPink3",                          (215, 0, 95)),
    entry("DeepPink3",                          (215, 0, 135)),
    entry("Magenta3",                           (215, 0, 175)),
    entry("Magenta3",                           (215, 0, 215)),
    entry("Magenta2",                           (215, 0, 255)),
    entry("DarkOrange3",                        (215, 95, 0)),
    entry("IndianRed",                          (215, 95, 95)),
    entry("HotPink3",                           (215, 95, 135)),
    entry("HotPink2",                           (215, 95, 175)),
    entry("Orchid",                             (215, 95, 215)),
    entry("MediumOrchid1",                      (215, 95, 255)),
    entry("Orange3",                            (215, 135, 0)),
    entry("LightSalmon3",                       (215, 135, 95)),
    entry("LightPink3",                         (215, 135, 135)),
    entry("Pink3",                              (215, 135, 175)),
    entry("Plum3",                              (215, 135, 215)),
    entry("Violet",                             (215, 135, 255)),
    entry("Gold3",                              (215, 175, 0)),
    entry("LightGoldenrod3",                    (215, 175, 95)),
    entry("Tan",                                (215, 175, 135)),
    entry("MistyRose3",                         (215, 175, 175)),
    entry("Thistle3",                           (215, 175, 215)),
    entry("Plum2",                              (215, 175, 255)),
    entry("Yellow3",                            (215, 215, 0)),
    entry("Khaki3",                             (215, 215, 95)),
    entry("LightGoldenrod2",                    (215, 215, 135)),
    entry("LightYellow3",                       (215, 215, 175)),
    entry("Grey84",                             (215, 215, 215)),
    entry("LightSteelBlue1",                    (215, 215, 255)),
    entry("Yellow2",                            (215, 255, 0)),
    entry("DarkOliveGreen1",                    (215, 255, 95)),
    entry("DarkOliveGreen1",                    (215, 255, 135)),
    entry("DarkSeaGreen1",                      (215, 255, 175)),
    entry("Honeydew2",                          (215, 255, 215)),
    entry("LightCyan1",                         (215, 255, 255)),
    entry("Red1",                               (255, 0, 0)),
    entry("DeepPink2",                          (255, 0, 95)),
    entry("DeepPink1",                          (255, 0, 135)),
    entry("DeepPink1",                          (255, 0, 175)),
    entry("Magenta2",                           (255, 0, 215)),
    entry("Magenta1",                           (255, 0, 255)),
    entry("OrangeRed1",                         (255, 95, 0)),
    entry("IndianRed1",                         (255, 95, 95)),
    entry("IndianRed1",                         (255, 95, 135)),
    entry("HotPink",                            (255, 95, 175)),
    entry("HotPink",                            (255, 95, 215)),
    entry("MediumOrchid1",                      (255, 95, 255)),
    entry("DarkOrange",                         (255, 135, 0)),
    entry("Salmon1",                            (255, 135, 95)),
    entry("LightCoral",                         (255, 135, 135)),
    entry("PaleVioletRed1",                     (255, 135, 175)),
    entry("Orchid2",                            (255, 135, 215)),
    entry("Orchid1",                            (255, 135, 255)),
    entry("Orange1",                            (255, 175, 0)),
    entry("SandyBrown",                         (255, 175, 95)),
    entry("LightSalmon1",                       (255, 175, 135)),
    entry("LightPink1",                         (255, 175, 175)),
    entry("Pink1",                              (255, 175, 215)),
    entry("Plum1",                              (255, 175, 255)),
    entry("Gold1",                              (255, 215, 0)),
    entry("LightGoldenrod2",                    (255, 215, 95)),
    entry("LightGoldenrod2",                    (255, 215, 135)),
    entry("NavajoWhite1",                       (255, 215, 175)),
    entry("MistyRose1",                         (255, 215, 215)),
    entry("Thistle1",                           (255, 215, 255)),
    entry("Yellow1",                            (255, 255, 0)),
    entry("LightGoldenrod1",                    (255, 255, 95)),
    entry("Khaki1",                             (255, 255, 135)),
    entry("Wheat1",                             (255, 255, 175)),
    entry("Cornsilk1",                          (255, 255, 215)),
    entry("Grey100",                            (255, 255, 255)),
    entry("Grey3",                              (8, 8, 8)),
    entry("Grey7",                              (18, 18, 18)),
    entry("Grey11",                             (28, 28, 28)),
    entry("Grey15",                             (38, 38, 38)),
    entry("Grey19",                             (48, 48, 48)),
    entry("Grey23",                             (58, 58, 58)),
    entry("Grey27",                             (68, 68, 68)),
    entry("Grey30",                             (78, 78, 78)),
    entry("Grey35",                             (88, 88, 88)),
    entry("Grey39",                             (98, 98, 98)),
    entry("Grey42",                             (108, 108, 108)),
    entry("Grey46",                             (118, 118, 118)),
    entry("Grey50",                             (128, 128, 128)),
    entry("Grey54",                             (138, 138, 138)),
    entry("Grey58",                             (148, 148, 148)),
    entry("Grey62",                             (158, 158, 158)),
    entry("Grey66",                             (168, 168, 168)),
    entry("Grey70",                             (178, 178, 178)),
    entry("Grey74",                             (188, 188, 188)),
    entry("Grey78",                             (198, 198, 198)),
    entry("Grey82",                             (208, 208, 208)),
    entry("Grey85",                             (218, 218, 218)),
    entry("Grey89",                             (228, 228, 228)),
    entry("Grey93",                             (238, 238, 238)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_two_colors() {
        let c1 = Color::from_rgb(0, 0, 0);
        let c2 = Color::from_rgb(0, 0, 0);
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_color_approximation() {
        let c = Color::from_rgb(1, 2, 3);
        assert_eq!(c.to_256_code(), 0)
    }
}
