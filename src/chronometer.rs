use crate::SystemTime;

#[derive(Clone, Copy)]
pub struct Chronometer {
    start: SystemTime,
}

impl Chronometer {
    pub fn new() -> Self {
        Self { start: SystemTime::now() }
    }

    pub fn elapsed(&self) -> String {
        let elapsed = SystemTime::now().duration_since(self.start).unwrap().as_millis();
        format!("{:02}:{:02}.{:03}", elapsed / 60000, (elapsed % 60000) / 1000, elapsed % 1000)
    }
}
