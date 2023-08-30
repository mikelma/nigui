use rustfft::{Fft, FftPlanner};
use std::sync::RwLock;

use std::sync::Arc;

mod plot;
pub mod read;

pub use plot::plot_waves;

/// The number of wave points to store. Buffers are circular,
/// hence, once the limit of the buffer is reached, data points get
/// overwriten with the new data (starting from the oldest data point).
// pub const WAVE_BUFF_LEN: usize = 1024;
pub const WAVE_BUFF_LEN: usize = 512;
/// Number of waves to track
pub const WAVE_BUFFS_NUM: usize = 4;

lazy_static! {
    /// This list contains the (circular) buffers that store the wave data.
    /// Each element of the list corresponds to one EEG wave. Then, each wave
    /// is stored in a circular buffer: a list of `f32` that stores the data,
    /// and an `usize` that refers to the index of the element to replace in
    /// the circular buffer.
    pub static ref WAVE_BUFFS : RwLock<[(usize, [f32; WAVE_BUFF_LEN]); WAVE_BUFFS_NUM]> = {
        let values = [(0, [0f32; WAVE_BUFF_LEN]); WAVE_BUFFS_NUM];
        RwLock::new(values)
    };

    /// Global storage for the values of the FFTs. For each wave buffer (`WAVE_BUFFS_NUM`),
    /// this list contains a list containing the values of the FFT.
    pub static ref FFT_BUFFS : RwLock<[[f32; WAVE_BUFF_LEN / 2]; WAVE_BUFFS_NUM]> = {
        let values = [[0f32; WAVE_BUFF_LEN / 2]; WAVE_BUFFS_NUM];
        RwLock::new(values)
    };

    /// The `Fft` object is stored globally to avoid creating one in each frame.
    pub static ref FFT: Arc<dyn Fft<f32>> = {
        let mut planner = FftPlanner::new();
        planner.plan_fft_forward(WAVE_BUFF_LEN)
    };

    pub static ref FFT_SCALE: f32 = 1.0 / (WAVE_BUFF_LEN as f32).sqrt();

    pub static ref RECORDING_BUFFS : RwLock<Vec<Vec<f32>>> = {
        let values = vec![vec![]];
        RwLock::new(values)
    };

    pub static ref RECORDING_FLAG : RwLock<bool> = {
        RwLock::new(false)
    };

}
