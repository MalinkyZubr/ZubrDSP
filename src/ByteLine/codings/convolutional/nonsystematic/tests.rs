#[cfg(test)]
pub mod ConvolutionalTests {
    use crate::ByteLine::codings::convolutional::nonsystematic::{
        encoder_io::{
            ConvolutionalInputConsumer, ConvolutionalInputProcessor, ConvolutionalOutputByteFactory
        }, 
        params::{
            ConvolutionalParameterError, ConvolutionalParams
        }, 
        trellis::{
            ConvolutionalLookupGenerator, ConvolutionalEncoderLookup, ConvolutionalDecoderLookup, TrellisState, TrellisStateChangeDecode, TrellisStateChangeEncode
        }
    };
    use std::collections::HashMap;

    #[test]
    fn euclidean_test() {
        let test_vec = vec![8, 12];
        assert!(ConvolutionalParams::gcd_euclidean(&test_vec) == 4);

        let test_vec = vec![12, 8];
        assert!(ConvolutionalParams::gcd_euclidean(&test_vec) == 4);

        let test_vec = vec![11, 3];
        assert!(ConvolutionalParams::gcd_euclidean(&test_vec) == 1);
    }

    #[test]
    fn euclidean_set_test() {
        let test_vec = vec![3, 11, 13];
        assert!(ConvolutionalParams::euclidean_set(&test_vec) == 1);

        let test_vec = vec![4, 6, 12, 18, 224, 128];
        assert!(ConvolutionalParams::euclidean_set(&test_vec) == 2);

        let test_vec = vec![4, 8, 12, 16, 224, 128];
        assert!(ConvolutionalParams::euclidean_set(&test_vec) == 4);
    }

    #[test]
    fn convolutional_param_validation() {
        {
            let test_encoder_1: Result<ConvolutionalParams, ConvolutionalParameterError> = ConvolutionalParams::new(9, 2, vec![3,5]);
            if let Some(ConvolutionalParameterError::ContextSizeError(_)) = test_encoder_1.err() {
                assert!(true);
            }
            else {
                assert!(false);
            }

            let test_encoder_2 = ConvolutionalParams::new(1, 2, vec![3,5]);
            if let Some(ConvolutionalParameterError::ContextSizeError(_)) = test_encoder_2.err() {
                assert!(true);
            }
            else {
                assert!(false);
            }
        }

        {
            let test_encoder_3 = ConvolutionalParams::new(5, 5, vec![3,5]);
            if let Some(ConvolutionalParameterError::InputBitError(_)) = test_encoder_3.err() {
                assert!(true);
            }
            else {
                assert!(false);
            }

            let test_encoder_4 = ConvolutionalParams::new(5, 0, vec![3,5]);
            if let Some(ConvolutionalParameterError::InputBitError(_)) = test_encoder_4.err() {
                assert!(true);
            }
            else {
                assert!(false);
            }
        }

        {
            let test_encoder_5 = ConvolutionalParams::new(5, 4, vec![3]);
            if let Some(ConvolutionalParameterError::OutputPolynomialCountError(_)) = test_encoder_5.err() {
                assert!(true);
            }
            else {
                assert!(false);
            }
            let test_encoder_6 = ConvolutionalParams::new(5, 4, vec![3, 5, 1, 7, 11, 13]);
            if let Some(ConvolutionalParameterError::OutputPolynomialCountError(_)) = test_encoder_6.err() {
                assert!(true);
            }
            else {
                assert!(false);
            }
        }
    
        {
            let test_encoder_7 = ConvolutionalParams::new(5, 4, vec![3, 67]);
            if let Some(ConvolutionalParameterError::OutputPolynomialFmtError(_)) = test_encoder_7.err() {
                assert!(true);
            }
            else {
                assert!(false);
            }
        }

        {
            let test_encoder_8 = ConvolutionalParams::new(5, 2, vec![4, 24]);
            if let Some(ConvolutionalParameterError::OutputPolynomialCatastrophicError(_)) = test_encoder_8.as_ref().err() {
                assert!(true);
            }
            else {
                match test_encoder_8 {
                    Ok(convolutional) => {dbg!("Got okay result!");},
                    Err(error) => {dbg!("{}", error);}
                }
                assert!(false, "Got error, or ok");
            }

            let test_encoder_8 = ConvolutionalParams::new(5, 2, vec![4, 24, 2]);
            if let Some(ConvolutionalParameterError::OutputPolynomialCatastrophicError(_)) = test_encoder_8.as_ref().err() {
                assert!(true);
            }
            else {
                match test_encoder_8 {
                    Ok(convolutional) => {dbg!("Got okay result!");},
                    Err(error) => {dbg!("{}", error);}
                }
                assert!(false, "Got error, or ok");
            }
        }
    }

    #[test]
    fn encoder_output_factory_test() {
        {
            let test_params1 = ConvolutionalParams::new(
                8, 
                2, 
                vec![3,5]);
            let mut output_factory: ConvolutionalOutputByteFactory = ConvolutionalOutputByteFactory::new(&test_params1.unwrap());
            let input_stream = vec![3, 3, 3, 3];

            for (idx, value) in input_stream.iter().enumerate() {
                let output = output_factory.append(*value);

                if idx == 3 {
                    dbg!("{}", output.unwrap());
                    assert!(output == Some(255));
                }
                else {
                    assert!(output == None);
                }
            }
        }
        {
            let test_params1 = ConvolutionalParams::new(
                8, 
                4, 
                vec![3,5]);
            let mut output_factory: ConvolutionalOutputByteFactory = ConvolutionalOutputByteFactory::new(&test_params1.unwrap());
            let input_stream = vec![7,2,4,10];

            let mut counter: i32 = 0;
            let correct_values: Vec<u8> = vec![39, 164];

            for value in input_stream.iter() {
                let output = output_factory.append(*value);

                match output {
                    Some(value) => {
                        dbg!("{}", value);
                        let correct_value = correct_values[counter as usize];
                        assert!(output == Some(correct_value));
                        counter += 1;
                    }
                    None => assert!(output == None),
                }
            }
        }
    }

    struct TestProcessor {}

    impl ConvolutionalInputProcessor for TestProcessor {
        fn process(&mut self, input: u8) -> Option<u8> {
            return Some(input);
        }
    }

    #[test]
    fn encoder_input_consumer_test() {
        {
            let test_params1 = ConvolutionalParams::new(
                8, 
                2, 
                vec![3,5]);
            dbg!("{}", &test_params1);
            let mut input_consumer1: ConvolutionalInputConsumer = ConvolutionalInputConsumer::new(
                Box::new(TestProcessor {}), 
                test_params1.unwrap()
            );
            let input_data1 = vec![0b10101010, 0b11110000];
            let output_data1 = input_consumer1.consume(&input_data1);

            assert!(output_data1 == vec![2,2,2,2,0,0,3,3]);
        }
        {
            let test_params = ConvolutionalParams::new(
                8, 
                4, 
                vec![3,5]);
            dbg!("{}", &test_params);
            let mut input_consumer: ConvolutionalInputConsumer = ConvolutionalInputConsumer::new(
                Box::new(TestProcessor {}), 
                test_params.unwrap()
            );
            let input_data = vec![0b10101010, 0b11110000];
            let output_data = input_consumer.consume(&input_data);
    
            assert!(output_data == vec![10, 10, 0, 15]);
        }
    }

    #[test]
    fn encoding_trellis_test() {
        let test_params1 = ConvolutionalParams::new(
            2, 
            1, 
            vec![1, 3]);
        
        let mut test_trellis: crate::ByteLine::codings::convolutional::nonsystematic::trellis::ConvolutionalEncoderLookup = ConvolutionalLookupGenerator::generate_encoding_lookup(&test_params1.unwrap());
        
        let reference_lookup: HashMap<u8, HashMap<u8, TrellisStateChangeEncode>> = [
            (0 as u8, [
                (0, TrellisStateChangeEncode{new_state: 0, }),
                (1, TrellisStateChangeEncode{new_state: 0, })
            ]),
            (1 as u8, [
                (0, ),
                (1, )
            ]),
            (2 as u8, [
                (0, ),
                (1, )
            ]),
            (3 as u8, [
                (0, ),
                (1, )
            ])
        ].into_iter().collect();

    }
}

