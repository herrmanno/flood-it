use std::fmt::Display;

use crate::{colorizer::Colorizer, problem::Color};

/// A solution to a problem, encoded as sequence of colors
#[derive(Clone)]
pub struct Solution {
    pub colors: Vec<Color>,
}

impl Display for Solution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let colorizer = Colorizer::new();
        for color in self.colors.iter() {
            colorizer.write(f, "  ", *color as usize)?;
            f.write_str(" ")?;
        }
        Ok(())
    }
}

impl From<Vec<Color>> for Solution {
    fn from(colors: Vec<Color>) -> Self {
        Self { colors }
    }
}
