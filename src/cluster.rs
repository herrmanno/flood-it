use std::collections::HashSet;

use crate::problem::{Color, Problem};
use crate::util::{neighbours, Point};

/// A cluster is a set of connected points with a common color
#[derive(Debug)]
pub struct Cluster {
    pub color: Color,
    pub fields: HashSet<Point>,
}

impl Cluster {
    /// Extract a list of clusters from the problem's grid
    pub fn from_problem(instance: &Problem) -> Vec<Cluster> {
        let Problem { grid } = instance;
        let height = instance.height();
        let width = instance.width();

        let mut clusters: Vec<Cluster> = vec![];
        let mut visited: HashSet<Point> = Default::default();

        for y in 0..height {
            for x in 0..width {
                if !visited.insert((y as u8, x as u8)) {
                    continue;
                }

                let color = grid[y][x];

                let mut fields: HashSet<Point> = Default::default();
                let mut queue: Vec<Point> = vec![(y as u8, x as u8)];
                while let Some((y, x)) = queue.pop() {
                    if fields.contains(&(y, x)) {
                        continue;
                    }

                    if color == grid[y as usize][x as usize] {
                        fields.insert((y, x));
                        visited.insert((y, x));

                        for pos in neighbours(y, x, height, width) {
                            queue.push(pos);
                        }
                    }
                }

                if !fields.is_empty() {
                    clusters.push(Cluster { color, fields });
                }
            }
        }

        clusters
    }
    /// Returns all points adjacent to this cluster
    pub fn neighbours(&self, height: usize, width: usize) -> impl Iterator<Item = Point> + '_ {
        self.fields
            .iter()
            .flat_map(move |(y, x)| neighbours(*y, *x, height, width))
            .filter(|pos| !self.fields.contains(pos))
    }

    /// Returns ids (index in `clusters`) of all clusters adjacent to this cluster
    pub fn neighbour_clusters(
        &self,
        clusters: &[Cluster],
        height: usize,
        width: usize,
    ) -> HashSet<usize> {
        let mut neighbour_indices: HashSet<usize> = Default::default();
        for (ny, nx) in self.neighbours(height, width) {
            let neighbour_cluster_idx = clusters
                .iter()
                .position(|other| other.fields.contains(&(ny, nx)))
                .unwrap();

            neighbour_indices.insert(neighbour_cluster_idx);
        }

        neighbour_indices
    }
}
