//! Solution model

use std::fmt::Display;

use crate::{colorizer::Colorizer, problem::Color};

/// A solution to a problem, encoded as sequence of colors
#[derive(Clone)]
pub struct Solution {
    /// The sequence of colors that solves a specific problem
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

impl<T> From<T> for Solution
where
    T: AsRef<[Color]>,
{
    fn from(colors: T) -> Self {
        let colors = colors.as_ref().to_vec();
        Self { colors }
    }
}
