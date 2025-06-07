#[cfg(test)]
pub mod bit_reversal_fft {
    use num::Complex;
    use crate::dsp::fft::bit_reversal::FFTBitReversal;

    fn verify_not_too_much_error(accepted: &Vec<Complex<f32>>, reality: &Vec<Complex<f32>>) {
        for (accepted, true_value) in accepted.iter().zip(reality) {
            let error = (accepted - true_value);//.norm();
            //let percent_error = (error / accepted).norm() * 100.0;

            assert!(error.norm() < 5.0);
        }
    }

    fn test_fft_bit_reversal(input_buffer: Vec<Complex<f32>>, true_f_domain: &Vec<Complex<f32>>, threads: usize) {
        let mut fft_computer = FFTBitReversal::new(input_buffer.len(), threads);

        let original_buffer = input_buffer.clone();

        let result = fft_computer.fft(input_buffer);

        dbg!("{}", &result);

        verify_not_too_much_error(&true_f_domain, &result);

        let original = fft_computer.ifft(result);

        dbg!("{}", &original);

        verify_not_too_much_error(&original_buffer, &original);
    } 

    fn convert_to_complex(input: Vec<f32>) -> Vec<Complex<f32>> {
        let new_vector = input.iter()
            .map(|value| Complex::new(*value, 0.0))
            .collect();

        return new_vector;
    }

    fn bit_reversal_tester(inputs_outputs: Vec<(usize, usize, usize)>) {
        for (input, string_size, expected_output) in inputs_outputs.iter() {
            assert!(FFTBitReversal::get_bit_reversal(*input, *string_size) == * expected_output);
        }
    }

    #[test]
    pub fn test_bit_reversal() {
        bit_reversal_tester(
            vec![
                (3, 3, 6),
                (6, 3, 3),
                (3, 4, 12),
                (5, 4, 10)
            ]
        );
    }

    #[test]
    pub fn test_ffts() {
        test_fft_bit_reversal(convert_to_complex(
            vec![1.0,0.0, 0.0, 0.0]
        ), &mut vec![
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
        ], 1
    );
        test_fft_bit_reversal(convert_to_complex(
            vec![1.0,2.0,3.0,4.0]
        ), &mut vec![
            Complex::new(10.0, 0.0),
            Complex::new(-2.0, 2.0),
            Complex::new(-2.0, 0.0),
            Complex::new(-2.0, -2.0),
        ], 1
    );
        test_fft_bit_reversal(convert_to_complex(
            vec![1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0]
        ), &mut vec![
            Complex::new(36.000000, 0.000000),
            Complex::new(-4.000000, 9.656854),
            Complex::new(-4.000000, 4.000000),
            Complex::new(-4.000000, 1.656854),
            Complex::new(-4.000000, 0.000000),
            Complex::new(-4.000000, -1.656854),
            Complex::new(-4.000000, -4.000000),
            Complex::new(-4.000000, -9.656854),
        ], 1
    )
    }

    #[test]
    pub fn test_ffts_parallel() {
        test_fft_bit_reversal(convert_to_complex(
            vec![1.0,0.0, 0.0, 0.0]
        ), &mut vec![
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
            Complex::new(1.0, 0.0),
        ], 3
    );
        test_fft_bit_reversal(convert_to_complex(
            vec![1.0,2.0,3.0,4.0]
        ), &mut vec![
            Complex::new(10.0, 0.0),
            Complex::new(-2.0, 2.0),
            Complex::new(-2.0, 0.0),
            Complex::new(-2.0, -2.0),
        ], 3
    );
        test_fft_bit_reversal(convert_to_complex(
            vec![1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0]
        ), &mut vec![
            Complex::new(36.000000, 0.000000),
            Complex::new(-4.000000, 9.656854),
            Complex::new(-4.000000, 4.000000),
            Complex::new(-4.000000, 1.656854),
            Complex::new(-4.000000, 0.000000),
            Complex::new(-4.000000, -1.656854),
            Complex::new(-4.000000, -4.000000),
            Complex::new(-4.000000, -9.656854),
        ], 7
    )
    }
}