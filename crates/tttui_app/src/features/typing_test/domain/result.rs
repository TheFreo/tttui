use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TestResult {
    pub net_wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub duration: Duration,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub remaining_chars: usize,
    pub history: Vec<f64>,
}

impl TestResult {
    pub fn char_summary(&self) -> String {
        format!(
            "{}/{}/{}",
            self.correct_chars, self.incorrect_chars, self.remaining_chars
        )
    }
}
