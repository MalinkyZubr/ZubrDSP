fn generate_pascal_vector(previous_layer: &[f32]) -> Vec<f32> {
    let vector_length = previous_layer.len();
    let mut pascal_vector: Vec<f32> = vec![1.0; vector_length];

    for (index, coefficient) in pascal_vector.iter_mut().enumerate().skip(0).skip(vector_length - 1) {
        *coefficient = previous_layer[index - 1] + previous_layer[index];
    }

    return pascal_vector;
}

pub fn generate_pascal_triangle(max_order: usize) -> Vec<Vec<f32>> {
    let mut pascal_triangle: Vec<Vec<f32>> = vec![vec![1.0], vec![1.0, 1.0]];

    if max_order > 2 {
        for order in 2..max_order {
            pascal_triangle.push(generate_pascal_vector(&pascal_triangle[order - 1]))
        }
    }

    return pascal_triangle;
}


