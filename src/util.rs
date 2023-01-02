#[inline(always)]
pub(crate) fn neighbours(y: u8, x: u8, height: usize, width: usize) -> Vec<(u8, u8)> {
    let (y, x) = (y as i8, x as i8);
    [(0, 1), (0, -1), (1, 0), (-1, 0)]
        .into_iter()
        .map(|(dy, dx)| (y + dy, x + dx))
        .filter(|&(y, x)| y >= 0 && x >= 0 && y < height as i8 && x < width as i8)
        .map(|(y, x)| (y as u8, x as u8))
        .collect()
}
