#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Twinkle,
    Rain,
}

#[derive(Debug, Clone)]
pub struct Star {
    pub x: u16,
    pub y: u16,
    stage: u8,
    wait: u8,
    key: u32,
}

impl Star {
    pub fn level(&self) -> Level {
        CYCLE[self.stage as usize]
    }

    // Stable for the life of the star, so its glyph and colour do not change when it moves.
    pub fn key(&self) -> u32 {
        self.key
    }
}

// Picks one item by a multiply hash of the star key. `salt` keeps two picks independent.
pub fn pick<T: Copy>(key: u32, salt: u64, items: &[T]) -> T {
    let mut h = u64::from(key).wrapping_mul(salt);
    h ^= h >> 32;
    items[(h % items.len() as u64) as usize]
}

pub struct Field {
    w: u16,
    h: u16,
    density: u8,
    mode: Mode,
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
            mode: Mode::Twinkle,
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

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn tick(&mut self) {
        for i in 0..self.stars.len() {
            // A rain star repeats its cycle while it falls.
            if self.advance(i) && self.mode == Mode::Twinkle {
                self.relocate(i);
            }
        }
        if self.mode == Mode::Rain {
            self.fall();
        }
    }

    // Moves the star one tick through its cycle. Gives true when the cycle wraps to the start.
    fn advance(&mut self, i: usize) -> bool {
        if self.stars[i].wait > 0 {
            self.stars[i].wait -= 1;
            return false;
        }
        let wait = self.next_wait();
        let star = &mut self.stars[i];
        star.wait = wait;
        if (star.stage as usize) < CYCLE.len() - 1 {
            star.stage += 1;
            return false;
        }
        star.stage = 0;
        true
    }

    fn relocate(&mut self, i: usize) {
        let old = self.index(self.stars[i].x, self.stars[i].y);
        self.taken[old] = false;
        let (x, y) = self.free_cell();
        self.respawn(i, x, y, 0);
    }

    // Puts the star at (x, y) as a new star, with a new key.
    fn respawn(&mut self, i: usize, x: u16, y: u16, stage: u8) {
        let key = self.next_key();
        let star = &mut self.stars[i];
        star.x = x;
        star.y = y;
        star.stage = stage;
        star.key = key;
    }

    // Every star moves down one row. A star that leaves the bottom row comes back as a new
    // star at row 0. Row 0 is empty after the move, so each new star finds a free column.
    fn fall(&mut self) {
        self.taken.fill(false);
        let (w, h) = (self.w as usize, self.h);
        let mut gone = Vec::new();
        for (i, s) in self.stars.iter_mut().enumerate() {
            s.y += 1;
            if s.y < h {
                self.taken[s.y as usize * w + s.x as usize] = true;
            } else {
                gone.push(i);
            }
        }
        for i in gone {
            let x = self.free_column();
            let stage = self.below(CYCLE.len() as u32) as u8;
            self.respawn(i, x, 0, stage);
        }
    }

    // Claims a random free cell in row 0. The caller ensures one exists.
    fn free_column(&mut self) -> u16 {
        self.claim_free(self.w as usize) as u16
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
            let key = self.next_key();
            self.stars.push(Star {
                x,
                y,
                stage,
                wait,
                key,
            });
        }
    }

    fn index(&self, x: u16, y: u16) -> usize {
        y as usize * self.w as usize + x as usize
    }

    // Claims a random free cell. The caller ensures one exists.
    fn free_cell(&mut self) -> (u16, u16) {
        let cell = self.claim_free(self.taken.len());
        (
            (cell % self.w as usize) as u16,
            (cell / self.w as usize) as u16,
        )
    }

    // Claims a random free cell among the first `n` cells, from a random start.
    fn claim_free(&mut self, n: usize) -> usize {
        let start = self.below(n as u32) as usize;
        let cell = (0..n)
            .map(|k| (start + k) % n)
            .find(|&c| !self.taken[c])
            .expect("no free cell");
        self.taken[cell] = true;
        cell
    }

    fn next_wait(&mut self) -> u8 {
        self.below(MAX_WAIT) as u8
    }

    fn next_key(&mut self) -> u32 {
        self.below(u32::MAX)
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
            assert!(
                s.x < f.size().0 && s.y < f.size().1,
                "star out of bounds: {s:?}"
            );
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

    // A respawn gives the star a new key. Record the levels of one full life, from one
    // respawn to the next.
    #[test]
    fn star_cycles_dark_dim_bright_dim_dark_then_respawns() {
        let mut f = Field::new(10, 10, 1, 7);
        let mut key = f.stars()[0].key();
        let mut lives = Vec::new();
        let mut levels = Vec::new();
        for _ in 0..500 {
            f.tick();
            let s = &f.stars()[0];
            if s.key() != key {
                key = s.key();
                lives.push(std::mem::take(&mut levels));
            }
            if levels.last() != Some(&s.level()) {
                levels.push(s.level());
            }
        }
        assert!(lives.len() >= 3, "star respawned {} times", lives.len());
        for life in &lives[1..] {
            assert_eq!(
                life,
                &[
                    Level::Dark,
                    Level::Dim,
                    Level::Bright,
                    Level::Dim,
                    Level::Dark
                ]
            );
        }
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

    fn rain(w: u16, h: u16, density: u8, seed: u64) -> Field {
        let mut f = Field::new(w, h, density, seed);
        f.set_mode(Mode::Rain);
        f
    }

    #[test]
    fn rain_tick_moves_each_star_down_one_row() {
        let mut f = rain(20, 10, 20, 11);
        let mut entered = 0;
        for _ in 0..50 {
            let before: Vec<_> = f.stars().iter().map(|s| (s.x, s.y, s.key())).collect();
            f.tick();
            assert_eq!(f.stars().len(), before.len());
            assert_valid(&f);
            for (s, &(x, y, key)) in f.stars().iter().zip(&before) {
                if y + 1 < f.size().1 {
                    assert_eq!((s.x, s.y, s.key()), (x, y + 1, key), "star did not fall");
                } else {
                    assert_eq!(s.y, 0, "new star entered below row 0");
                    entered += 1;
                }
            }
        }
        assert!(entered > 0, "no star left the bottom row");
    }

    #[test]
    fn rain_new_stars_enter_only_at_row_0() {
        let mut f = rain(30, 12, 10, 4);
        for _ in 0..50 {
            let keys: HashSet<_> = f.stars().iter().map(|s| s.key()).collect();
            f.tick();
            for s in f.stars().iter().filter(|s| !keys.contains(&s.key())) {
                assert_eq!(s.y, 0, "{s:?}");
            }
        }
    }

    #[test]
    fn rain_stars_keep_twinkling() {
        let mut f = rain(20, 200, 5, 6);
        let levels: HashSet<_> = (0..40)
            .map(|_| {
                f.tick();
                f.stars()[0].level()
            })
            .collect();
        assert_eq!(levels.len(), 3, "{levels:?}");
    }

    #[test]
    fn rain_tiny_sizes_do_not_panic() {
        for (w, h) in [(0, 0), (1, 1), (0, 5), (5, 0), (5, 1), (1, 5)] {
            let mut f = rain(w, h, 100, 1);
            for _ in 0..20 {
                f.tick();
                assert_valid(&f);
            }
            f.resize(3, 3);
            f.tick();
            assert_valid(&f);
        }
    }

    #[test]
    fn twinkle_after_rain_keeps_one_star_per_cell() {
        let mut f = rain(20, 10, 50, 3);
        for _ in 0..5 {
            f.tick();
        }
        f.set_mode(Mode::Twinkle);
        for _ in 0..200 {
            f.tick();
            assert_valid(&f);
            assert_eq!(f.stars().len(), 100);
        }
    }
}
