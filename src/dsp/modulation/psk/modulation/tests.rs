#[cfg(test)]
pub mod psk_tests {
    use crate::dsp::modulation::psk::modulation::phasegen::{PSKPoint, PhaseVectorGenerator};
    use crate::dsp::modulation::psk::modulation::psk::{PSKModulator, BasisType};
    use crate::pipeline::api::*;
    use std::f32::consts::PI;

    #[test]
    fn phasegen_test() {
        let mut phase_vector_generator: PhaseVectorGenerator = PhaseVectorGenerator::new(PSKPoint::BPSK, PI);
        let result: Result<ODFormat<Vec<f32>>, String> = phase_vector_generator.run_SISO(vec![210]);

        let golden_reference = vec![0.0, PI, 0.0, 0.0, PI, 0.0, PI, PI];
        
        let result = result.unwrap();

        for (index, golden_val) in golden_reference.iter().enumerate() {
            assert!(*golden_val == *result.clone().unwrap_standard().get(index).unwrap());
        }

        let mut phase_vector_generator: PhaseVectorGenerator = PhaseVectorGenerator::new(PSKPoint::QPSK, 3.0 * PI / 2.0);
        let result: Result<ODFormat<Vec<f32>>, String> = phase_vector_generator.run_SISO(vec![210]);

        let golden_reference = vec![PI, 0.0, PI / 2.0, 3.0 * PI / 2.0];

        let result = result.unwrap();
        
        for (index, golden_val) in golden_reference.iter().enumerate() {
            assert!(*golden_val == *result.clone().unwrap_standard().get(index).unwrap());
        }
    }

    #[test]
    fn psk_test() {
        let mut phase_vector_generator: PhaseVectorGenerator = PhaseVectorGenerator::new(PSKPoint::BPSK, PI);
        let result: Result<ODFormat<Vec<f32>>, String> = phase_vector_generator.run_SISO(vec![210]);

        let result = result.unwrap();
        let result = match result {
            ODFormat::Standard(data) => data,
            _ => panic!("Expected ODFormat::Interleaved")
        };

        let length = (&result).len();

        let mut modulator = PSKModulator::new(16.0, BasisType::COSINE);

        let result_final = modulator.run_SISO(result);

        assert!(result_final.unwrap().unwrap_standard().len() == (15) * (length) as usize);
    }
}