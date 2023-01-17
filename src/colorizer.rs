//! Utility for printing colors
//! 
use owo_colors::colors::xterm::*;
use owo_colors::OwoColorize;

const DEFAULT_COLOR_MAP: [&str; 10] = [
    "red",
    "blue",
    "green",
    "yellow",
    "orange",
    "purple",
    "light_green",
    "pink",
    "indigo",
    "olive",
];

/// Utility helper to print string in different colors based on a color index
pub(crate) struct Colorizer {
    color_mapping: Vec<String>,
}

impl Colorizer {
    pub(crate) fn new() -> Self {
        let color_mapping = DEFAULT_COLOR_MAP.iter().map(|s| (*s).to_owned()).collect();

        Self { color_mapping }
    }

    pub(crate) fn write<W>(
        &self,
        w: &mut W,
        s: &str,
        color_index: usize,
    ) -> Result<(), std::fmt::Error>
    where
        W: std::fmt::Write,
    {
        match self.color_mapping[color_index].as_str() {
            "red" => write_colored::<Red>(w, s),
            "blue" => write_colored::<Blue>(w, s),
            "green" => write_colored::<DarkGreen>(w, s),
            "yellow" => write_colored::<Yellow>(w, s),
            "orange" => write_colored::<FlushOrange>(w, s),
            "purple" => write_colored::<Purple>(w, s),
            "light_green" => write_colored::<LightSpringGreen>(w, s),
            "pink" => write_colored::<PinkFlamingo>(w, s),
            "indigo" => write_colored::<Indigo>(w, s),
            "olive" => write_colored::<Olive>(w, s),
            _ => panic!("Invalid color index: {color_index}"),
        }
    }
}

fn write_colored<T: owo_colors::Color>(
    w: &mut impl std::fmt::Write,
    s: &str,
) -> Result<(), std::fmt::Error> {
    w.write_fmt(format_args!("{}", s.bg::<T>()))
}
