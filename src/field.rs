#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Dark,
    Dim,
    Bright,
}

const CYCLE: [Level; 5] = [
    Level::Dark,
    Level::Dim,
    Level::Bright,
    Level::Dim,
    Level::Dark,
];

#[derive(Debug, Clone)]
pub struct Star {
    pub x: u16,
    pub y: u16,
    stage: u8,
    wait: u8,
}

impl Star {
    pub fn level(&self) -> Level {
        CYCLE[self.stage as usize]
    }
}

pub struct Field {
    w: u16,
    h: u16,
    density: u8,
    stars: Vec<Star>,
    taken: Vec<bool>,
    rng: u64,
}

const MAX_WAIT: u32 = 8;

impl Field {
    pub fn new(w: u16, h: u16, density: u8, seed: u64) -> Self {
        let mut f = Field {
            w,
            h,
            density,
            stars: Vec::new(),
            taken: vec![false; w as usize * h as usize],
            rng: seed,
        };
        f.fill();
        f
    }

    pub fn size(&self) -> (u16, u16) {
        (self.w, self.h)
    }

    pub fn stars(&self) -> &[Star] {
        &self.stars
    }

    pub fn tick(&mut self) {
        for i in 0..self.stars.len() {
            if self.stars[i].wait > 0 {
                self.stars[i].wait -= 1;
                continue;
            }
            let wait = self.next_wait();
            let star = &mut self.stars[i];
            star.wait = wait;
            if (star.stage as usize) < CYCLE.len() - 1 {
                star.stage += 1;
                continue;
            }
            let old = self.index(self.stars[i].x, self.stars[i].y);
            self.taken[old] = false;
            let (x, y) = self.free_cell();
            let star = &mut self.stars[i];
            star.stage = 0;
            star.x = x;
            star.y = y;
        }
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.w = w;
        self.h = h;
        self.stars.retain(|s| s.x < w && s.y < h);
        self.stars.truncate(self.target());
        self.taken = vec![false; w as usize * h as usize];
        for i in 0..self.stars.len() {
            let c = self.index(self.stars[i].x, self.stars[i].y);
            self.taken[c] = true;
        }
        self.fill();
    }

    fn target(&self) -> usize {
        let cells = self.w as usize * self.h as usize;
        (cells * self.density as usize + 50) / 100
    }

    // New stars start at a random stage so the first frame is already lit.
    fn fill(&mut self) {
        while self.stars.len() < self.target() {
            let (x, y) = self.free_cell();
            let stage = self.below(CYCLE.len() as u32) as u8;
            let wait = self.next_wait();
            self.stars.push(Star { x, y, stage, wait });
        }
    }

    fn index(&self, x: u16, y: u16) -> usize {
        y as usize * self.w as usize + x as usize
    }

    // Claims a random free cell. The caller ensures one exists.
    fn free_cell(&mut self) -> (u16, u16) {
        let n = self.taken.len();
        let start = self.below(n as u32) as usize;
        let cell = (0..n)
            .map(|k| (start + k) % n)
            .find(|&c| !self.taken[c])
            .expect("no free cell");
        self.taken[cell] = true;
        (
            (cell % self.w as usize) as u16,
            (cell / self.w as usize) as u16,
        )
    }

    fn next_wait(&mut self) -> u8 {
        self.below(MAX_WAIT) as u8
    }

    // splitmix64
    fn below(&mut self, n: u32) -> u32 {
        self.rng = self.rng.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (((z >> 32) * n as u64) >> 32) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn assert_valid(f: &Field) {
        let mut seen = HashSet::new();
        for s in f.stars() {
            assert!(s.x < f.w && s.y < f.h, "star out of bounds: {s:?}");
            assert!(seen.insert((s.x, s.y)), "two stars share a cell: {s:?}");
        }
    }

    #[test]
    fn seeds_density_percent_of_cells() {
        let f = Field::new(80, 24, 5, 1);
        assert_eq!(f.stars().len(), 96);
        assert_valid(&f);
        assert_eq!(Field::new(10, 10, 1, 1).stars().len(), 1);
        assert_eq!(Field::new(10, 10, 100, 1).stars().len(), 100);
    }

    #[test]
    fn star_cycles_dark_dim_bright_dim_dark_then_respawns() {
        let mut f = Field::new(10, 10, 1, 7);
        f.stars[0].stage = 0;
        f.stars[0].wait = 0;
        let mut levels = vec![f.stars[0].level()];
        let mut moved = false;
        for _ in 0..200 {
            let before = (f.stars[0].stage, f.stars[0].x, f.stars[0].y);
            f.tick();
            let s = &f.stars[0];
            if before.0 == 4 && s.stage == 0 {
                moved = true;
                break;
            }
            if levels.last() != Some(&s.level()) {
                levels.push(s.level());
            }
        }
        assert!(moved, "star never respawned");
        assert_eq!(
            levels,
            [
                Level::Dark,
                Level::Dim,
                Level::Bright,
                Level::Dim,
                Level::Dark
            ]
        );
    }

    #[test]
    fn respawn_lands_in_free_cell() {
        let mut f = Field::new(10, 10, 90, 3);
        for _ in 0..500 {
            f.tick();
            assert_eq!(f.stars().len(), 90);
            assert_valid(&f);
        }
    }

    #[test]
    fn respawn_moves_stars_over_time() {
        let mut f = Field::new(40, 20, 5, 9);
        let start: HashSet<_> = f.stars().iter().map(|s| (s.x, s.y)).collect();
        for _ in 0..500 {
            f.tick();
        }
        let end: HashSet<_> = f.stars().iter().map(|s| (s.x, s.y)).collect();
        assert_ne!(start, end);
    }

    #[test]
    fn resize_smaller_drops_out_of_bounds_stars() {
        let mut f = Field::new(80, 24, 50, 5);
        f.resize(20, 10);
        assert_eq!(f.stars().len(), 100);
        assert_valid(&f);
    }

    #[test]
    fn resize_bigger_fills_to_density() {
        let mut f = Field::new(20, 10, 5, 5);
        f.resize(80, 24);
        assert_eq!(f.stars().len(), 96);
        assert_valid(&f);
    }

    #[test]
    fn tiny_sizes_do_not_panic() {
        for (w, h) in [(0, 0), (1, 1), (0, 5), (5, 0)] {
            let mut f = Field::new(w, h, 100, 1);
            f.tick();
            f.resize(1, 1);
            f.tick();
            f.resize(0, 0);
            f.tick();
            assert_valid(&f);
        }
        let mut f = Field::new(1, 1, 100, 1);
        for _ in 0..50 {
            f.tick();
        }
        assert_eq!(f.stars().len(), 1);
    }
}
