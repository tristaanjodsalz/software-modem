use std::sync::Arc;

use realfft::{RealFftPlanner, RealToComplex, num_complex::Complex32};
use smart_default::SmartDefault;

use crate::{
    ofdm::OFDMConstants,
    qam::{QAMModem, QAMOrder},
};

#[allow(dead_code)]
const PILOT_VALUE_TO_BE_CHANGED: Complex32 = Complex32 { re: 1.0, im: 0.0 };

pub struct OFDMDemodulator {
    fft: Arc<dyn RealToComplex<f32>>,
    qam_modem: QAMModem,
    constants: OFDMConstants,
}

impl OFDMDemodulator {
    /// Creates a new OFDM modulator with the given [configuration](OFDMDemodulatorConfig).
    pub fn new(config: OFDMDemodulatorConfig) -> Self {
        let qam_modem = QAMModem::new(config.qam_order);

        let constants = OFDMConstants::new(
            config.num_subcarriers,
            config.pilot_subcarrier_every,
            config.cyclic_prefix_length,
            config.qam_order,
            qam_modem.bits_per_symbol(),
        );

        let fft = config.fft.unwrap_or_else(|| {
            RealFftPlanner::<f32>::new().plan_fft_forward(2 * config.num_subcarriers as usize)
        });

        OFDMDemodulator {
            fft,
            qam_modem,
            constants,
        }
    }

    /// Demodulates a single OFDM symbol from the given input buffer.
    ///
    /// The input buffer must have a length equal to the expected symbol length,
    /// which is `2 * num_subcarriers + cyclic_prefix_length`,
    /// or: `self.get_symbol_length()`.
    ///
    /// # Panics
    /// If the input buffer length does not match the expected length.
    ///
    /// # Example
    /// ```
    /// use software_modem::ofdm::demodulator::{OFDMDemodulator, OFDMDemodulatorConfig};
    /// use software_modem::qam::QAMOrder;
    ///
    /// let demodulator = OFDMDemodulator::new(OFDMDemodulatorConfig {
    ///     num_subcarriers: 64,
    ///     cyclic_prefix_length: 4,
    ///     pilot_subcarrier_every: 4,
    ///     qam_order: QAMOrder::QAM16,
    ///     fft: None,
    /// });
    ///
    /// let input_buffer = vec![1.5578203, 10.757554, -60.41084, -22.017548, 170.0, -42.44605, 54.674767, 22.390936, 6.2399883, -4.9697013, 22.430595, 17.925348, -2.8670907, -23.034523, -11.360638, 0.024665833, -3.071948, -7.734082, 3.0158787, 21.293457, 0.82842445, -35.719788, -33.072395, -19.85823, -0.14415121, -1.0148859, 1.0802565, 1.3617897, 1.0318756, -7.007739, 2.1753244, 15.374781, 21.054213, 0.07890889, -1.2171764, -3.3891459, -2.0, 41.081707, -4.085703, 0.47892523, -0.24726725, 6.605378, -11.310527, -4.8029222, -3.2976942, 6.129626, -5.986044, 17.46577, 33.94296, 56.904747, 10.276956, 26.332466, -21.798985, -45.932056, 16.227457, -11.979431, -5.4379044, -10.107577, 12.925878, 5.066286, 7.585412, -2.9996142, 5.774047, -8.335448, -6.82592, -9.922427, 26.371922, 19.215015, -6.0, -0.36616898, -44.328407, -32.542404, -11.508089, -6.3610272, -14.268342, -14.096208, 4.5239453, 3.1953726, -9.655043, -32.157936, -18.771591, -23.806992, -12.9909935, -65.67099, -4.8284245, 67.96052, 26.218727, 38.012096, 13.98769, 15.913272, -13.206813, -18.395777, -10.68873, 22.887703, 19.290443, -5.741539, -23.786112, -0.9140358, 27.256096, 6.191677, -42.0, 1.7305107, -14.260653, 9.6725445, -2.4846325, 4.7253504, -4.8517256, 0.97378147, -6.3591604, 13.709526, 19.001724, 14.6675, -20.099422, -25.363672, -8.301841, 18.045067, 17.798985, 13.69133, -17.373789, -6.1744323, -16.405634, -4.7908087, -8.799321, 11.967701, -5.9285583, -12.88035, -35.239815, -1.2977934, 1.5578203, 10.757554, -60.41084, -22.017548];
    ///
    /// let demodulated_data = demodulator.demodulate_symbol_from_buffer(&input_buffer);
    ///
    /// assert_eq!(demodulated_data, "Hello, OFDM!            ".as_bytes());
    /// ```
    pub fn demodulate_symbol_from_buffer(&self, input_buffer: &[f32]) -> Vec<u8> {
        if input_buffer.len() != self.get_symbol_length() {
            panic!(
                "Symbol buffer length must be {}, but got {}",
                self.get_symbol_length(),
                input_buffer.len()
            );
        }

        let demodulated_symbol = self.demodulate_ofdm_symbol(input_buffer).unwrap();

        self.qam_modem.demodulate(&demodulated_symbol)
    }

    fn demodulate_ofdm_symbol(&self, input: &[f32]) -> Result<Vec<Complex32>, String> {
        // remove cyclic prefix
        let mut input_no_cp = vec![0.0; 2 * self.constants.num_subcarriers as usize];
        input_no_cp.clone_from_slice(&input[self.constants.cyclic_prefix_length as usize..]);

        // time domain to frequency domain
        let mut output_buffer = self.fft.make_output_vec();
        self.fft
            .process(&mut input_no_cp, &mut output_buffer)
            .unwrap();

        // equalize
        // for now, just scale everything to fit the range of QAM symbols
        let max_value = output_buffer.iter().map(|c| c.norm()).fold(0.0, f32::max);
        if max_value > 0.0 {
            for value in output_buffer.iter_mut() {
                *value /= max_value / 3.0;
            }
        }

        // extract data subcarriers
        let mut output_symbols =
            vec![Complex32::default(); self.constants.data_subcarrier_indices.len()];
        for (i, &idx) in self.constants.data_subcarrier_indices.iter().enumerate() {
            output_symbols[i] = output_buffer[idx as usize];
        }

        Ok(output_symbols)
    }

    /// Returns the length of the OFDM symbol, including the cyclic prefix.
    ///
    /// The length is calculated as:
    /// `2 * num_subcarriers + cyclic_prefix_length`.
    pub fn get_symbol_length(&self) -> usize {
        (2 * self.constants.num_subcarriers + self.constants.cyclic_prefix_length) as usize
    }
}

/// Configuration for the [OFDM Demodulator](OFDMDemodulator).
///
/// Just contruct this struct with the desired parameters and pass it to the `OFDMDemodulator::new()` method.
#[derive(SmartDefault)]
pub struct OFDMDemodulatorConfig {
    pub num_subcarriers: u32,
    /// Length of the cyclic prefix in samples.
    ///
    /// One OFDM symbol double num_subcarriers samples. If you want to have a CP of 1/4 you need to set this to `(2 * num_subcarriers) / 4`
    pub cyclic_prefix_length: u32,
    /// Interval for pilot subcarriers.
    ///
    /// Inserts pilot subcarriers every `pilot_subcarrier_every` subcarrier.
    #[default(4)]
    pub pilot_subcarrier_every: u32,
    pub qam_order: QAMOrder,
    /// Optional FFT implementation/planner to use.
    ///
    /// If `None`, a default FFT planner will be used.
    pub fft: Option<Arc<dyn RealToComplex<f32>>>,
}
