pub struct ChatInstance {
    pub promote: String,
    pub model: String,
    pub seed: u64,
    pub temp: Option<f64>,
    pub top_p: Option<f64>,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
}

impl Default for ChatInstance {
    fn default() -> Self {
        Self {
            promote: Default::default(),
            model: Default::default(),
            seed: Default::default(),
            temp: None,
            top_p: None,
            repeat_penalty: Default::default(),
            repeat_last_n: Default::default(),
        }
    }
}
