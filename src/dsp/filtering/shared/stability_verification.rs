use num::traits::{Pow};
use num::complex::Complex;
use rand::{self, Rng};



pub fn z_pole_stable(pole: &Complex<f32>) -> bool {
    return pole.norm() < 1.0;
}

pub fn z_poles_stable(poles: Vec<Complex<f32>>) -> bool {
    for pole in poles.iter() {
        if !z_pole_stable(&pole) {
            return false;
        }
    }

    return true;
}

pub fn generate_derivative_coefficients(polynomial: &Vec<f32>) -> Vec<f32> {
    let mut derivative: Vec<f32> = Vec::with_capacity(polynomial.len() - 1);

    for order in 1..polynomial.len() {
        derivative.push(polynomial[order] * (order) as f32);
    }

    return derivative;
}

pub fn identify_root_bounds(polynomial: &Vec<f32>) -> (f32, f32) {
    let leading_coefficient = polynomial.last().unwrap();
    let mut max_value = 0.0;
    
    for coefficient in polynomial[0..polynomial.len()-1].iter() {
        let current_value = coefficient.abs() / leading_coefficient.abs();
        if current_value > max_value {
            max_value = current_value;
        }
    };

    return (-(max_value + 1.0), max_value + 1.0);
}  

pub fn generate_initial_estimations(bounds: &(f32, f32), estimation: &mut Vec<Complex<f32>>, num_estimates: usize) {
    let mut index = 0;
    let mut rng = rand::rng();

    while index < num_estimates {
        let real_component = rng.random_range(bounds.0..bounds.1); // get a real value in the range of possible values
        let imaginary_component = rng.random_range(0.0..((bounds.1.pow(2) - real_component.pow(2)) as f32).sqrt());

        estimation.push(Complex::new(real_component, imaginary_component));
        index += 1;
    }
}

fn aberth_complete(current_estimate: &Vec<Complex<f32>>, next_estimate: &Vec<Complex<f32>>, maximum_error: f32) -> bool {
    for (current, next) in current_estimate.iter().zip(next_estimate) {
        if (current - next).norm().abs() > maximum_error {
            return false;
        }
    }

    return true;
}

pub fn compute_polynomial_image(argument: &Complex<f32>, polynomial: &Vec<f32>) -> Complex<f32> {
    let mut image: Complex<f32> = Complex::new(0.0, 0.0);

    for (order, coefficient) in polynomial.iter().enumerate() {
        image += coefficient * argument.pow(&(order as f32));
    }

    return image;
}

fn compute_aberth_iteration(polynomial: &Vec<f32>, derivative: &Vec<f32>, current_estimate: &Vec<Complex<f32>>) -> Vec<Complex<f32>> {
    let mut new_estimate: Vec<Complex<f32>> = Vec::with_capacity(current_estimate.len());

    for estimate in current_estimate.iter() {
        let numerator: Complex<f32> = compute_polynomial_image(estimate, polynomial) / 
                             compute_polynomial_image(estimate, derivative);

        let mut sum_term: Complex<f32> = Complex::new(0.0, 0.0);

        for root in current_estimate.iter() {
            if (root - estimate).norm() > 0.01 {
                sum_term += 1.0 / (estimate - root);
            }
        };

        let denominator = 1.0 - (numerator * sum_term);
        new_estimate.push(estimate - (numerator / denominator));
    }

    return new_estimate;
}

pub fn compute_polynomial_roots(polynomial: &Vec<f32>, maximum_error: f32) -> Vec<Complex<f32>> {
    let derivative = generate_derivative_coefficients(polynomial);
    let mut estimation: Vec<Complex<f32>> = Vec::with_capacity(polynomial.len());

    let bounds: (f32, f32) = identify_root_bounds(polynomial);

    generate_initial_estimations(&bounds, &mut estimation, polynomial.len() - 1);

    let mut new_estimation = compute_aberth_iteration(polynomial, &derivative, &estimation);

    while !aberth_complete(&estimation, &new_estimation, maximum_error) {
        estimation = new_estimation;
        new_estimation = compute_aberth_iteration(polynomial, &derivative, &estimation);
    }

    return new_estimation;
}

pub fn verify_z_domain_stability(polynomial: &Vec<f32>, maximum_error: f32) -> bool {
    let roots: Vec<Complex<f32>> = compute_polynomial_roots(polynomial, maximum_error);
    return z_poles_stable(roots);
}
