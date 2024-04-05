use std::time::Duration;

use serde::{Deserialize, Serialize};
use serialimage::{OptimumExposure, OptimumExposureBuilder};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct OptExposureConfig {
    pub percentile_pix: f32,
    pub pixel_tgt: f32,
    pub pixel_tol: f32,
    pub pixel_exclusion: u32,
    pub min_exopsure: Duration,
    pub max_exposure: Duration,
    pub max_bin: u16,
}

impl OptExposureConfig {
    pub fn get_optimum_exp_config(&self) -> Option<OptimumExposure> {
        OptimumExposureBuilder::default()
        .percentile_pix(self.percentile_pix)
        .pixel_tgt(self.pixel_tgt)
        .pixel_uncertainty(self.pixel_tol)
        .pixel_exclusion(self.pixel_exclusion)
        .min_allowed_exp(self.min_exopsure)
        .max_allowed_exp(self.max_exposure)
        .max_allowed_bin(self.max_bin)
        .build()
        .ok()
    }
}
