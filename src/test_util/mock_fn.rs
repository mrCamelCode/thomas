pub struct MockFn {
    num_calls: u32,
}
impl MockFn {
    pub fn new() -> Self {
        Self { num_calls: 0 }
    }

    pub fn call(&mut self) {
        self.num_calls += 1;
    }

    pub fn reset(&mut self) {
        self.num_calls = 0;
    }

    pub fn num_calls(&self) -> u32 {
        self.num_calls
    }
}
