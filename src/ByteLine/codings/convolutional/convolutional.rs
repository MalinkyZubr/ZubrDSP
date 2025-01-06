pub struct Convolutinal { // basically, more of any of these is better error correction, worse computational performance
    context: u8,
    context_size: u8, // k
    input_bits: u8,
    output_polynomials: Vec<u8>, // len is num output bits. R = input / output
}

impl Convolutionl {
    pub fn new(context_size: u8, input_bits: u8, output_polynomials: Vec<u8>) -> Convolutinal { // remember polynomials with coefficients 0 or 1, each term representing positions in the context. Modulo 2 addition of terms. LSB represented by 1 in 1 + x + x^2 for instance
        if context_size > 8 || context_size < 2 {
            panic!("You must pass a context size between 8 and 2 inclusive.");
        }
        if input_bits > context_size - 1 || input_bits < 1 {
            panic!("You must pass an input bits size between 1 and 1 less than context size, inclusive");
        }
        if output_polynomials.len() > 5 || output_polynomials < 2 {
            panic!("You must pass at least 2 and less than 5 polynomials");
        }
        for polynomial in output_polynomials.iter() {
            if !self.polynomial_formatting_okay(&context_size, &polynomial) {
                panic!("Polynomials cannot access nonexistant space in the context. The value of your output polynomial byte cannot exceed the aggregate of 'context-size' least significant bytes in the context");
            }
        }

        Convolutinal {
            context: 0,
            context_size,
            input_bits,
            output_polynomials
        }
    }

    fn polynomial_formatting_okay(&self, context_size: &u8, output_polynomials: &Vec<u8>) -> bool {
        return output_polynomials <= 2.pow(context_size)
    }

    fn gcd_euclidean(&self, values: &Vec<u8>) -> u8{
        let mut a: u8 = values[0];
        let mut b: u8 = values[1];
        
        if b > a {
            std::mem::swap(&mut a, &mut b);
        }

        while a != 0 && b != 0 {
            let remainder: u8 = a % b;
            a = b;
            b = remainder;
        }

        return a + b;
    }

    fn euclidean_set(&self, input_set: &Vec<u8>) -> u8 { // repeat this 
        if input_set.len() > 1 {
            let mut output_set: Vec<u8> = Vec::new();
            let mut slice_vector: Vec<u8> = vec![0,0];
            let mut start_index: usize = 0;
            let input_set_len = input_set.len();

            while start_index < input_set_len - 1 {
                let mut relation_index = start_index + 1;
                
                while relation_index < input_set_len {
                    slice_vector[0] = input_set[start_index];
                    slice_vector[1] = input_set[relation_index];
                    let gcd = self.gcd_euclidean(&slice_vector);
                    if !output_set.contains(gcd) {
                        output_set.push(gcd);
                    }
                    relation_index += 1;
                }

                start_index += 1;
            }

            return self.euclidean_set(&output_set);
        }
        else {
            return input_set[0];
        }
    }

    fn is_catastrophic(&self, output_polynomials: &Vec<u8>) { // relate every element to the other

    }
}