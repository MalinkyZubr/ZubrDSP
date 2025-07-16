#[cfg(test)]
pub mod psk_tests {
    use crate::dsp::modulation::psk::modulation::phasegen::{PSKPoint, PhaseVectorGenerator};
    use crate::dsp::modulation::psk::modulation::psk::{PSKModulator, BasisType};
    use crate::pipeline::pipeline_step::PipelineStep;
    use std::f32::consts::PI;

    #[test]
    fn phasegen_test() {
        let mut phase_vector_generator: PhaseVectorGenerator = PhaseVectorGenerator::new(PSKPoint::BPSK, PI);
        let result: Vec<f32> = phase_vector_generator.run(vec![210]);

        let golden_reference = vec![0.0, PI, 0.0, 0.0, PI, 0.0, PI, PI];

        for (index, golden_val) in golden_reference.iter().enumerate() {
            assert!(*golden_val == *result.get(index).unwrap());
        }

        let mut phase_vector_generator: PhaseVectorGenerator = PhaseVectorGenerator::new(PSKPoint::QPSK, 3.0 * PI / 2.0);
        let result: Vec<f32> = phase_vector_generator.run(vec![210]);

        let golden_reference = vec![PI, 0.0, PI / 2.0, 3.0 * PI / 2.0];

        for (index, golden_val) in golden_reference.iter().enumerate() {
            assert!(*golden_val == *result.get(index).unwrap());
        }
    }

    #[test]
    fn psk_test() {
        let mut phase_vector_generator: PhaseVectorGenerator = PhaseVectorGenerator::new(PSKPoint::BPSK, PI);
        let result: Vec<f32> = phase_vector_generator.run(vec![210]);

        let length = (&result).len();

        let mut modulator = PSKModulator::new(16.0, BasisType::COSINE);

        let result_final = modulator.run(result);
        dbg!("{}", &result_final.len());

        assert!(result_final.len() == (15) * (length) as usize);
    }
}