use std::time::{SystemTime, UNIX_EPOCH};

pub fn suffix<T>(vec: &mut [T], n: usize) {
    let len = vec.len();
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as usize;
    let mut rng = SimpleRng::new(seed);
    for i in 0..n {
        let j = rng.gen_range(i, len);
        vec.swap(i, j);
    }
}

pub fn random_choose_n<T>(mut vec: Vec<T>, n: usize) -> Vec<T> {
    if n == 0 {
        return Vec::new();
    }
    let len = vec.len();
    if n > len {
        panic!("n cannot be greater than the length of the vector");
    }
    suffix(&mut vec, n);
    vec.truncate(n);
    vec
}

struct SimpleRng {
    state: usize,
}

impl SimpleRng {
    fn new(seed: usize) -> Self {
        Self { state: seed }
    }
    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        min + (self.state % (max - min))
    }
}
