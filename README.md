# software-modem

A software implementation of a QAM/OFDM modem written in pure rust.

The goal is have a fully working OFDM modulator and demodulator for use in RF Systems.

## Modules

1. **QAM**:
   The QAM modulator and demodulator, which maps bits to Complex Numbers representing amplitude/phase combinations and vice verca.

2. **OFDM**
   1. **Modulator**
      Here lives the main code to modulate QAM Symbols (or just any Coordinates on the Complex Plane) to a number of samples in the time domain.
   2. **Demodulator**
      Here lives the code to demodulate samples from the time domain to QAM Symbols.

## Example

```rust
use software_modem::qam::QAMOrder;
use software_modem::ofdm::demodulator::{OFDMDemodulator, OFDMDemodulatorConfig};
use software_modem::ofdm::modulator::{OFDMModulator, OFDMModulatorConfig};

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
```
