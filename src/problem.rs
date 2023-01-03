use crate::{
    colorizer::Colorizer,
    util::{neighbours, Point},
};
use std::{collections::HashSet, fmt::Display};

/// A number denoting a color (by index)
pub type Color = u8;

/// A 'flood it' problem instance
#[derive(Clone)]
pub struct Problem {
    pub grid: Vec<Vec<Color>>,
}

impl Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let colorizer = Colorizer::new();

        for line in self.grid.iter() {
            for field in line {
                colorizer.write(f, "  ", *field as usize)?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl Problem {
    /// Construct a problem instance from stdin
    ///
    /// Problems should be encoded as
    /// ```
    /// 010
    /// 102
    /// 201
    /// ```
    // where every digit denotes a color between 0 and 9 (inclusive).
    pub fn from_stdin() -> Self {
        let grid: Vec<Vec<Color>> = std::io::stdin()
            .lines()
            .map(|line| {
                line.unwrap()
                    .chars()
                    .map(|ch| ch.to_digit(10).unwrap() as u8)
                    .collect()
            })
            .collect();

        assert!(!grid.is_empty(), "Grid must not be empty");
        assert!(!grid[0].is_empty(), "Grid rows must not be empty");
        assert_eq!(
            *grid.iter().flat_map(|row| row.iter()).min().unwrap(),
            0,
            "Min color value must be 0"
        );

        Self { grid }
    }

    /// The problem's height
    pub fn height(&self) -> usize {
        self.grid.len()
    }

    /// The problem's width
    pub fn width(&self) -> usize {
        self.grid[0].len()
    }

    /// Max color number used in this problem
    pub fn num_colors(&self) -> usize {
        *self.grid.iter().flat_map(|row| row.iter()).max().unwrap() as usize + 1
    }

    /// Colors a problem instance with the given color
    pub fn apply_color(&mut self, color: Color) {
        let curr_color = self.grid[0][0];

        let mut current_cluster: HashSet<Point> = Default::default();

        let mut queue: Vec<Point> = vec![(0, 0)];
        while let Some(pos @ (y, x)) = queue.pop() {
            if !current_cluster.insert(pos) {
                continue;
            }

            for (y, x) in neighbours(y, x, self.height(), self.width()) {
                if self.grid[y as usize][x as usize] == curr_color {
                    queue.push((y, x))
                }
            }
        }

        for (y, x) in current_cluster {
            self.grid[y as usize][x as usize] = color;
        }
    }
}
