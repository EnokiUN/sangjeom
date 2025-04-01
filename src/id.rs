use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct IdGen {
    counter: u8,
}

impl IdGen {
    pub fn new() -> Self {
        Self { counter: 0 }
    }
    pub fn generate(&mut self) -> i64 {
        if self.counter == u8::MAX {
            self.counter = 0
        } else {
            self.counter += 1;
        }
        debug!("IdGen counter incremented to {}", self.counter);
        // Yucky
        (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs() as i64
            & 0xFFFFFF)
            << 8
            | (self.counter as i64)
    }
}

#[cfg(test)]
mod test {
    use super::IdGen;

    fn extract_components(id: i64) -> (i64, u8) {
        (id >> 8, (id & 0xFF) as u8)
    }

    #[test]
    fn id_test() {
        let mut gen = IdGen::new();
        let (_, counter) = extract_components(gen.generate());
        assert_eq!(counter, 1);
        let (_, counter) = extract_components(gen.generate());
        assert_eq!(counter, 2);
        gen.counter = u8::MAX - 1;
        let (_, counter) = extract_components(gen.generate());
        assert_eq!(counter, u8::MAX);
        let (_, counter) = extract_components(gen.generate());
        assert_eq!(counter, 0);
    }
}
