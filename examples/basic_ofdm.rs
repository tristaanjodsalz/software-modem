use software_modem::ofdm::demodulator::{OFDMDemodulator, OFDMDemodulatorConfig};
use software_modem::ofdm::modulator::{OFDMModulator, OFDMModulatorConfig};
use software_modem::qam::QAMOrder;

fn main() {
    let ofdm_modulator = OFDMModulator::new(OFDMModulatorConfig {
        num_subcarriers: 64,
        cyclic_prefix_length: 4,
        pilot_subcarrier_every: 4,
        qam_order: QAMOrder::QAM16,
        fft: None,
    });

    let test_data = "Hello, OFDM!";

    // move test_data into buffer of correct size
    let mut data_buffer = vec![0; 32 - 6 - 2]; // 32 bytes for QAM16 (4bits) * 64 Subcarriers minus 6 pilot subcarriers and first and last subcarrier
    data_buffer[..test_data.len()].copy_from_slice(test_data.as_bytes());

    // modulate the buffer
    let mut modulated_symbol = vec![0.0; ofdm_modulator.get_symbol_length()];

    ofdm_modulator.modulate_buffer_as_symbol(&data_buffer, &mut modulated_symbol);
    println!("Modulated Symbol: {:?}", &modulated_symbol[..8]); // print first 8 samples

    let ofdm_demodulator = OFDMDemodulator::new(OFDMDemodulatorConfig {
        num_subcarriers: 64,
        cyclic_prefix_length: 4,
        pilot_subcarrier_every: 4,
        qam_order: QAMOrder::QAM16,
        fft: None,
    });

    // demodulate the symbol
    let mut demodulated_buffer = ofdm_demodulator.demodulate_symbol_from_buffer(&modulated_symbol);

    // strip trailing zeros
    demodulated_buffer.retain(|&x| x != 0);

    let demodulated_data = String::from_utf8(demodulated_buffer).unwrap();
    assert_eq!(demodulated_data, test_data);

    println!("Demodulated Data: {}", demodulated_data);
}
