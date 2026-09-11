//! Original maze and deterministic 120 Hz simulation. No platform I/O.
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

pub const W: i16 = 23;
pub const H: i16 = 25;
pub const STEP: f32 = 1.0 / 120.0;
pub const TUNNEL: i16 = 11;
pub const START: Pos = Pos { x: 11, y: 19 };
pub const HOME: Pos = Pos { x: 11, y: 11 };

// A circuit-like maze authored for this game. All collectables are reachable.
pub const MAP: [&str; H as usize] = [
    "#######################",
    "#o....#.........#....o#",
    "#.##.#.#.#####.#.#.##.#",
    "#....#.#...#...#.#....#",
    "#.####.###.#.###.####.#",
    "#......#.......#......#",
    "###.##.#.#####.#.##.###",
    "#...#....#...#....#...#",
    "#.###.##.#.#.#.##.###.#",
    "#.....#....#....#.....#",
    "#####.#.##   ##.#.#####",
    "........       ........",
    "#.###.#.#######.#.###.#",
    "#...#.#....#....#.#...#",
    "###.#.####.#.####.#.###",
    "#.....#.........#.....#",
    "#.###.#.#######.#.###.#",
    "#.#...#...#.#...#...#.#",
    "#.#.#####.#.#.#####.#.#",
    "#.........   .........#",
    "#.####.##.#.#.##.####.#",
    "#...#.....#.#.....#...#",
    "###.#.###.#.#.###.#.###",
    "#o....#.........#....o#",
    "#######################",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pos {
    pub x: i16,
    pub y: i16,
}
impl Pos {
    pub fn index(self) -> usize {
        (self.y * W + self.x) as usize
    }
    pub fn open(self) -> bool {
        self.x >= 0
            && self.x < W
            && self.y >= 0
            && self.y < H
            && MAP[self.y as usize].as_bytes()[self.x as usize] != b'#'
    }
    pub fn neighbor(self, dir: Dir) -> Option<Self> {
        let (dx, dy) = dir.delta();
        let mut next = Self {
            x: self.x + dx,
            y: self.y + dy,
        };
        if next.y == TUNNEL {
            next.x = next.x.rem_euclid(W);
        }
        next.open().then_some(next)
    }
    fn nearest(self) -> Self {
        (0..H)
            .flat_map(|y| (0..W).map(move |x| Self { x, y }))
            .filter(|p| p.open())
            .min_by_key(|p| {
                (p.x as i32 - self.x as i32).pow(2) + (p.y as i32 - self.y as i32).pow(2)
            })
            .unwrap_or(START)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dir {
    Up,
    Left,
    Down,
    Right,
}
impl Dir {
    pub const ALL: [Self; 4] = [Self::Up, Self::Left, Self::Down, Self::Right];
    pub fn delta(self) -> (i16, i16) {
        match self {
            Self::Up => (0, -1),
            Self::Left => (-1, 0),
            Self::Down => (0, 1),
            Self::Right => (1, 0),
        }
    }
    pub fn opposite(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Left => Self::Right,
            Self::Down => Self::Up,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mover {
    pub from: Pos,
    pub to: Pos,
    pub progress: f32,
    pub dir: Dir,
}
impl Mover {
    pub fn at(pos: Pos, dir: Dir) -> Self {
        Self {
            from: pos,
            to: pos,
            progress: 0.,
            dir,
        }
    }
    pub fn xy(&self) -> (f32, f32) {
        let mut dx = (self.to.x - self.from.x) as f32;
        if dx.abs() > 1. {
            dx = if dx > 0. { -1. } else { 1. };
        }
        (
            self.from.x as f32 + dx * self.progress,
            self.from.y as f32 + (self.to.y - self.from.y) as f32 * self.progress,
        )
    }
    pub fn reverse(&mut self) {
        self.dir = self.dir.opposite();
        if self.from != self.to {
            std::mem::swap(&mut self.from, &mut self.to);
            self.progress = 1. - self.progress;
        }
    }
    fn begin(&mut self, dir: Dir) -> bool {
        if let Some(next) = self.from.neighbor(dir) {
            self.dir = dir;
            self.to = next;
            true
        } else {
            false
        }
    }
    // Speeds are bounded to < 1 tile per fixed tick; arrival is exact.
    fn advance(&mut self, distance: f32) -> bool {
        if self.from == self.to {
            return false;
        }
        self.progress += distance;
        if self.progress >= 1. {
            self.from = self.to;
            self.progress = 0.;
            true
        } else {
            false
        }
    }
    fn valid(&self) -> bool {
        self.from.open()
            && self.to.open()
            && self.progress.is_finite()
            && (0.0..=1.0).contains(&self.progress)
            && if self.from == self.to {
                self.progress == 0.
            } else {
                self.from.neighbor(self.dir) == Some(self.to)
            }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Ready,
    Playing,
    Dying,
    Cleared,
    GameOver,
}
impl Phase {
    pub fn active(self) -> bool {
        matches!(self, Self::Playing | Self::Dying | Self::Cleared)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pursuer {
    pub mover: Mover,
    pub returning: bool,
    pub release: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Dot,
    Power,
    Capture,
    Death,
    Clear,
    ExtraLife,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Game {
    pub player: Mover,
    pub pursuers: [Pursuer; 4],
    pub pellets: Vec<u8>,
    pub score: u32,
    pub level: u32,
    pub lives: u8,
    pub phase: Phase,
    pub phase_time: f32,
    pub power: f32,
    pub chain: u8,
    pub elapsed: f32,
    pub next_life: u32,
    pub wanted: Dir,
    pub relaxed: bool,
}
impl Default for Game {
    fn default() -> Self {
        Self::new(false)
    }
}
impl Game {
    pub fn new(relaxed: bool) -> Self {
        let mut game = Self {
            player: Mover::at(START, Dir::Left),
            pursuers: std::array::from_fn(|i| Pursuer {
                mover: Mover::at(
                    Pos {
                        x: 9 + i as i16,
                        y: 11,
                    },
                    Dir::Up,
                ),
                returning: false,
                release: i as f32 * 1.8,
            }),
            pellets: vec![],
            score: 0,
            level: 1,
            lives: 3,
            phase: Phase::Ready,
            phase_time: 0.,
            power: 0.,
            chain: 0,
            elapsed: 0.,
            next_life: 10000,
            wanted: Dir::Left,
            relaxed,
        };
        game.fill_pellets();
        game
    }
    fn fill_pellets(&mut self) {
        self.pellets = MAP
            .iter()
            .flat_map(|row| {
                row.bytes().map(|b| match b {
                    b'.' => 1,
                    b'o' => 2,
                    _ => 0,
                })
            })
            .collect();
    }
    fn reset_actors(&mut self) {
        self.player = Mover::at(START, Dir::Up);
        self.wanted = Dir::Up;
        for (i, ghost) in self.pursuers.iter_mut().enumerate() {
            *ghost = Pursuer {
                mover: Mover::at(
                    Pos {
                        x: 9 + i as i16,
                        y: 11,
                    },
                    Dir::Up,
                ),
                returning: false,
                release: i as f32 * 1.8 + 1.5,
            };
        }
        self.power = 0.;
        self.elapsed = 0.;
        self.chain = 0;
    }
    pub fn input(&mut self, dir: Dir) {
        self.wanted = dir;
        if self.phase == Phase::Ready {
            self.phase = Phase::Playing;
        }
        if self.phase == Phase::Playing && dir == self.player.dir.opposite() {
            self.player.reverse();
        }
    }
    pub fn remaining(&self) -> usize {
        self.pellets.iter().filter(|&&p| p != 0).count()
    }
    pub fn power_duration(&self) -> f32 {
        (7.5 - (self.level - 1).min(10) as f32 * 0.4).max(3.5)
    }
    pub fn speed(&self) -> f32 {
        (5.5 + (self.level - 1).min(12) as f32 * 0.13) * if self.relaxed { 0.75 } else { 1. }
    }
    pub fn scatter(&self) -> bool {
        self.elapsed % 27. < 7.
    }
    fn award(&mut self, points: u32, events: &mut Vec<Event>) {
        self.score = self.score.saturating_add(points);
        if self.score >= self.next_life {
            self.next_life = self.next_life.saturating_add(10000);
            self.lives = (self.lives + 1).min(9);
            events.push(Event::ExtraLife);
        }
    }
    fn collect(&mut self, events: &mut Vec<Event>) {
        let p = &mut self.pellets[self.player.from.index()];
        let kind = *p;
        *p = 0;
        match kind {
            1 => {
                self.award(10, events);
                events.push(Event::Dot);
            }
            2 => {
                self.power = self.power_duration();
                self.chain = 0;
                self.award(50, events);
                events.push(Event::Power);
                for g in &mut self.pursuers {
                    if !g.returning && g.release <= 0. {
                        g.mover.reverse();
                    }
                }
            }
            _ => (),
        }
        if kind != 0 && self.remaining() == 0 {
            self.phase = Phase::Cleared;
            self.phase_time = 1.6;
            events.push(Event::Clear);
        }
    }
    pub fn tick(&mut self) -> Vec<Event> {
        let mut events = Vec::new();
        if matches!(self.phase, Phase::Dying | Phase::Cleared) {
            self.phase_time = (self.phase_time - STEP).max(0.);
            if self.phase_time == 0. {
                if self.phase == Phase::Cleared {
                    self.level = self.level.saturating_add(1);
                    self.fill_pellets();
                }
                if self.lives == 0 {
                    self.phase = Phase::GameOver;
                } else {
                    self.reset_actors();
                    self.phase = Phase::Ready;
                }
            }
            return events;
        }
        if self.phase != Phase::Playing {
            return events;
        }
        self.elapsed = (self.elapsed + STEP) % 2700.;
        self.power = (self.power - STEP).max(0.);
        if self.player.from == self.player.to {
            self.collect(&mut events);
            if self.phase != Phase::Playing {
                return events;
            }
            if !self.player.begin(self.wanted) {
                self.player.begin(self.player.dir);
            }
        }
        if self.player.advance(self.speed() * STEP) {
            self.collect(&mut events);
            if self.phase != Phase::Playing {
                return events;
            }
        }
        for i in 0..4 {
            if self.pursuers[i].release > 0. {
                self.pursuers[i].release = (self.pursuers[i].release - STEP).max(0.);
                continue;
            }
            if self.pursuers[i].returning && self.pursuers[i].mover.from == HOME {
                self.pursuers[i].returning = false;
                self.pursuers[i].release = 1.5;
                continue;
            }
            if self.pursuers[i].mover.from == self.pursuers[i].mover.to {
                if let Some(dir) = self.choose(i) {
                    self.pursuers[i].mover.begin(dir);
                }
            }
            let multiplier = if self.pursuers[i].returning {
                1.9
            } else if self.power > 0. {
                0.58
            } else {
                0.88 + (i as f32 * 0.015)
            };
            self.pursuers[i]
                .mover
                .advance(self.speed() * multiplier * STEP);
        }
        self.collide(&mut events);
        events
    }
    fn collide(&mut self, events: &mut Vec<Event>) {
        let (px, py) = self.player.xy();
        for i in 0..4 {
            let g = &self.pursuers[i];
            if g.returning || g.release > 0. {
                continue;
            }
            let (gx, gy) = g.mover.xy();
            let mut dx = (px - gx).abs();
            if py == TUNNEL as f32 && gy == TUNNEL as f32 {
                dx = dx.min((W as f32 - dx).abs());
            }
            if dx * dx + (py - gy).powi(2) >= 0.62_f32.powi(2) {
                continue;
            }
            if self.power > 0. {
                self.pursuers[i].returning = true;
                self.award(200 << self.chain.min(3), events);
                self.chain = (self.chain + 1).min(4);
                events.push(Event::Capture);
            } else {
                self.lives = self.lives.saturating_sub(1);
                self.phase = Phase::Dying;
                self.phase_time = 1.3;
                events.push(Event::Death);
                break;
            }
        }
    }
    fn target(&self, i: usize) -> Pos {
        let corners = [
            Pos { x: 21, y: 1 },
            Pos { x: 1, y: 1 },
            Pos { x: 21, y: 23 },
            Pos { x: 1, y: 23 },
        ];
        if self.pursuers[i].returning {
            return HOME;
        }
        if self.scatter() && self.power == 0. {
            return corners[i];
        }
        let p = self.player.from;
        let (dx, dy) = self.player.dir.delta();
        match i {
            0 => p,
            1 => Pos {
                x: p.x + dx * 4,
                y: p.y + dy * 4,
            }
            .nearest(),
            2 => Pos {
                x: p.x + dx * 2 - dy * 3,
                y: p.y + dy * 2 + dx * 3,
            }
            .nearest(),
            _ => {
                let g = self.pursuers[i].mover.from;
                if (g.x - p.x).abs() + (g.y - p.y).abs() < 6 {
                    corners[i]
                } else {
                    p
                }
            }
        }
    }
    fn choose(&self, i: usize) -> Option<Dir> {
        let ghost = &self.pursuers[i];
        let pos = ghost.mover.from;
        let flee = self.power > 0. && !ghost.returning;
        let distances = distances(if flee {
            self.player.from
        } else {
            self.target(i)
        });
        let mut choices: Vec<_> = Dir::ALL
            .into_iter()
            .filter_map(|dir| pos.neighbor(dir).map(|p| (dir, distances[p.index()])))
            .collect();
        if choices.len() > 1 && !ghost.returning {
            choices.retain(|(d, _)| *d != ghost.mover.dir.opposite());
        }
        choices
            .into_iter()
            .min_by_key(|(_, d)| if flee { -(*d as i32) } else { *d as i32 })
            .map(|(dir, _)| dir)
    }
    pub fn validate(&self) -> bool {
        self.pellets.len() == (W * H) as usize
            && self.pellets.iter().enumerate().all(|(i, &p)| {
                let original = MAP[i / W as usize].as_bytes()[i % W as usize];
                p == 0 || p == 1 && original == b'.' || p == 2 && original == b'o'
            })
            && self.player.valid()
            && self.pursuers.iter().all(|g| {
                g.mover.valid() && g.release.is_finite() && (0.0..=20.).contains(&g.release)
            })
            && (1..=1_000_000).contains(&self.level)
            && self.lives <= 9
            && (self.lives > 0 || matches!(self.phase, Phase::Dying | Phase::GameOver))
            && (self.phase != Phase::GameOver || self.lives == 0)
            && self.phase_time.is_finite()
            && (0.0..=2.).contains(&self.phase_time)
            && self.power.is_finite()
            && (0.0..=8.).contains(&self.power)
            && self.elapsed.is_finite()
            && (0.0..=2700.).contains(&self.elapsed)
            && self.chain <= 4
            && self.next_life > self.score
            && self.next_life.is_multiple_of(10000)
    }
}

pub fn distances(start: Pos) -> Vec<u16> {
    let mut ds = vec![u16::MAX; (W * H) as usize];
    if !start.open() {
        return ds;
    }
    ds[start.index()] = 0;
    let mut queue = VecDeque::from([start]);
    while let Some(p) = queue.pop_front() {
        for dir in Dir::ALL {
            if let Some(n) = p.neighbor(dir) {
                if ds[n.index()] == u16::MAX {
                    ds[n.index()] = ds[p.index()] + 1;
                    queue.push_back(n);
                }
            }
        }
    }
    ds
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    #[test]
    fn maze_is_connected_and_bounded() {
        for row in MAP {
            assert_eq!(row.len(), W as usize, "{row}");
        }
        assert!(START.open());
        assert!(HOME.open());
        let ds = distances(START);
        for y in 0..H {
            for x in 0..W {
                let p = Pos { x, y };
                if p.open() {
                    assert_ne!(ds[p.index()], u16::MAX, "unreachable {p:?}");
                }
            }
        }
        assert_eq!(
            Game::default().pellets.iter().filter(|&&p| p == 2).count(),
            4
        );
    }
    #[test]
    fn tunnel_is_bidirectional() {
        assert_eq!(
            Pos { x: 0, y: 11 }.neighbor(Dir::Left),
            Some(Pos { x: 22, y: 11 })
        );
        assert_eq!(
            Pos { x: 22, y: 11 }.neighbor(Dir::Right),
            Some(Pos { x: 0, y: 11 })
        );
        assert_eq!(Pos { x: 1, y: 1 }.neighbor(Dir::Up), None);
    }
    #[test]
    fn reversal_preserves_position() {
        let mut m = Mover {
            from: Pos { x: 1, y: 1 },
            to: Pos { x: 2, y: 1 },
            progress: 0.3,
            dir: Dir::Right,
        };
        let before = m.xy();
        m.reverse();
        assert!((m.xy().0 - before.0).abs() < 0.0001);
        assert!(m.valid());
    }
    #[test]
    fn buffered_turn_and_wall_stop() {
        let mut g = Game::default();
        g.player = Mover::at(Pos { x: 1, y: 1 }, Dir::Up);
        g.input(Dir::Up);
        for ghost in &mut g.pursuers {
            ghost.release = 20.;
        }
        for _ in 0..120 {
            g.tick();
        }
        assert_eq!(g.player.from, Pos { x: 1, y: 1 });
        g.input(Dir::Right);
        for _ in 0..60 {
            g.tick();
        }
        assert!(g.player.from.x > 1);
        assert!(g.player.valid());
    }
    #[test]
    fn pellet_is_collected_once() {
        let mut g = Game::default();
        g.player = Mover::at(Pos { x: 2, y: 1 }, Dir::Right);
        let mut e = vec![];
        g.collect(&mut e);
        g.collect(&mut e);
        assert_eq!(g.score, 10);
    }
    #[test]
    fn early_turn_waits_until_the_next_junction() {
        let mut g = Game::default();
        g.player = Mover {
            from: Pos { x: 1, y: 1 },
            to: Pos { x: 2, y: 1 },
            progress: 0.25,
            dir: Dir::Right,
        };
        g.phase = Phase::Playing;
        for ghost in &mut g.pursuers {
            ghost.release = 20.;
        }
        g.input(Dir::Down);
        for _ in 0..120 {
            g.tick();
            if g.player.from.y > 1 {
                break;
            }
        }
        assert_eq!(g.player.from, Pos { x: 4, y: 2 });
    }
    #[test]
    fn power_expiry_restores_collision_danger() {
        let mut g = Game::default();
        g.phase = Phase::Playing;
        g.power = STEP / 2.;
        g.pursuers[0].mover = Mover::at(START, Dir::Up);
        g.tick();
        assert_eq!(g.power, 0.);
        assert_eq!(g.lives, 2);
    }
    #[test]
    fn captured_pursuer_returns_home_before_rejoining() {
        let mut g = Game::default();
        g.phase = Phase::Playing;
        for ghost in &mut g.pursuers {
            ghost.release = 20.;
        }
        g.pursuers[0] = Pursuer {
            mover: Mover::at(Pos { x: 1, y: 1 }, Dir::Right),
            returning: true,
            release: 0.,
        };
        for _ in 0..1500 {
            g.tick();
            if !g.pursuers[0].returning {
                break;
            }
        }
        assert!(!g.pursuers[0].returning);
        assert_eq!(g.pursuers[0].mover.from, HOME);
        assert!(g.pursuers[0].release > 0.);
    }
    #[test]
    fn power_chain_and_returning_are_safe() {
        let mut g = Game::default();
        g.power = 5.;
        g.phase = Phase::Playing;
        for ghost in &mut g.pursuers {
            ghost.mover = Mover::at(START, Dir::Left);
            ghost.release = 0.;
        }
        g.collide(&mut vec![]);
        assert_eq!(g.score, 3000);
        assert_eq!(g.lives, 3);
        g.collide(&mut vec![]);
        assert_eq!(g.score, 3000);
        assert!(g.pursuers.iter().all(|p| p.returning));
    }
    #[test]
    fn contact_removes_only_one_life() {
        let mut g = Game::default();
        g.phase = Phase::Playing;
        for ghost in &mut g.pursuers {
            ghost.mover = Mover::at(START, Dir::Left);
            ghost.release = 0.;
        }
        g.collide(&mut vec![]);
        assert_eq!(g.lives, 2);
        assert_eq!(g.phase, Phase::Dying);
        for _ in 0..180 {
            g.tick();
        }
        assert_eq!(g.lives, 2);
        assert_eq!(g.phase, Phase::Ready);
    }
    #[test]
    fn final_death_finishes_game() {
        let mut g = Game::default();
        g.lives = 1;
        g.pursuers[0].mover = Mover::at(START, Dir::Up);
        g.collide(&mut vec![]);
        for _ in 0..180 {
            g.tick();
        }
        assert_eq!(g.phase, Phase::GameOver);
        assert!(g.validate());
    }
    #[test]
    fn level_clear_repopulates_without_resetting_score() {
        let mut g = Game::default();
        g.pellets.fill(0);
        g.player = Mover::at(Pos { x: 2, y: 1 }, Dir::Right);
        g.pellets[g.player.from.index()] = 1;
        g.collect(&mut vec![]);
        assert_eq!(g.phase, Phase::Cleared);
        for _ in 0..210 {
            g.tick();
        }
        assert_eq!(g.level, 2);
        assert_eq!(g.score, 10);
        assert!(g.remaining() > 100);
        assert_eq!(g.phase, Phase::Ready);
    }
    #[test]
    fn extra_life_is_awarded_once_per_threshold() {
        let mut g = Game::default();
        g.score = 9990;
        g.award(10, &mut vec![]);
        g.award(10, &mut vec![]);
        assert_eq!(g.lives, 4);
        assert_eq!(g.next_life, 20000);
    }
    #[test]
    fn corrupt_geometry_and_nan_are_rejected() {
        let mut g = Game::default();
        assert!(g.validate());
        g.player.progress = f32::NAN;
        assert!(!g.validate());
        let mut g = Game::default();
        g.pellets[0] = 1;
        assert!(!g.validate());
    }
    #[test]
    fn snapshot_continues_deterministically() {
        let mut a = Game::default();
        a.input(Dir::Down);
        for _ in 0..210 {
            a.tick();
        }
        let mut b: Game = serde_json::from_str(&serde_json::to_string(&a).unwrap()).unwrap();
        for t in 0..4000 {
            if t % 83 == 0 {
                let d = Dir::ALL[(t / 83) % 4];
                a.input(d);
                b.input(d);
            }
            assert_eq!(a.tick(), b.tick());
        }
        assert_eq!(a, b);
        assert!(a.validate());
    }
    #[test]
    fn long_random_input_stays_valid() {
        let mut g = Game::default();
        let mut rng = 71231u32;
        for t in 0..60000 {
            if t % 31 == 0 {
                rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
                g.input(Dir::ALL[(rng >> 16) as usize % 4]);
            }
            g.tick();
            assert!(g.validate(), "tick {t}");
            if g.phase == Phase::GameOver {
                g = Game::default();
            }
        }
    }
}
