
use std::path::{PathBuf, Path};

use ccdi_common::append_to_file;

// ============================================ PUBLIC =============================================

pub struct ProgrammableOutput {
    path: PathBuf,
    pattern: Vec<bool>,
    position: usize,
}

pub fn write_output(path: &Path, value: bool) -> Result<(), String> {
    append_to_file(
        match value {
            false => "0\n",
            true => "1\n",
        },
        path
    )
}

pub fn pattern_pwm(value: f32) -> Vec<bool> {
    let pivot = (value*100.0) as usize;

    (0..100usize)
        .map(|index| if index < pivot { true } else { false } )
        .collect()
}

pub fn status_healthy() -> Vec<bool> {
    vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        .into_iter().map(|value| value > 0).collect()
}

impl ProgrammableOutput {
    pub fn new(path: &str) -> Self {
        Self {
            path: PathBuf::from(path.to_owned()),
            pattern: vec![false],
            position: 0,
        }
    }

    pub fn set_pattern(&mut self, pattern: Vec<bool>) {
        self.pattern = pattern;
    }

    #[allow(unused)]
    pub fn iterate(&mut self) -> Result<(), String> {
        self.position += 1;

        if self.position >= self.pattern.len() {
            self.position = 0;
        }

        match self.pattern.get(self.position) {
            None => Ok(()),
            Some(value) => write_output(&self.path, *value),
        }
    }
}

