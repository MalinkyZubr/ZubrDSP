use crate::ByteLine::codings::convolutional::nonsystematic::trellis::*;
use crate::ByteLine::codings::opts::*;

use super::encoder;




pub struct HiddenMarkovModel {
    pub max_state: u8,
    pub transition_matrix: Vec<Vec<f32>>,
    pub emission_matrix: Vec<Vec<u8>>,
    pub start_probabilities: Vec<f32>
}


impl HiddenMarkovModel {
    pub fn new(max_state: u8, transition_matrix: Vec<Vec<f32>>, emission_matrix: Vec<Vec<u8>>, start_probabilities: Vec<f32>) -> Self {
        return HiddenMarkovModel { // add some checks here
            max_state,
            transition_matrix, 
            emission_matrix,
            start_probabilities
        }
    }
}


pub struct ViterbiOpCore {
    viterbi_matrix: Vec<Vec<f32>>,
    state_paths: Vec<Vec<u8>>,
    newpath_buffer: Vec<Vec<u8>>,
    current_start_probabilities: Vec<f32>,
    markov_model: HiddenMarkovModel,
    max_hamming_distance: u8
}

impl ViterbiOpCore {
    pub fn new(buffer_size: usize, encoder_lookup: &ConvolutionalEncoderLookup) -> Self { // expect unpacked input observations
        let num_states = encoder_lookup.encoding_lookup.len();
        let viterbi_matrix: Vec<Vec<f32>> = vec![vec![0.0; num_states]; buffer_size];

        let mut state_paths: Vec<Vec<u8>> = vec![vec![0; buffer_size]; num_states];
        let mut newpath_buffer: Vec<Vec<u8>> = vec![vec![0; buffer_size]; num_states];

        let mut start_probabilities: Vec<f32> = vec![0.0; num_states];
        start_probabilities[0] = 1.0;

        let markov_model: HiddenMarkovModel = HiddenMarkovModel::new(num_states as u8, encoder_lookup.to_transition_matrix(), start_probabilities);

        let current_start_probabilities = markov_model.start_probabilities.clone();

        ViterbiOpCore { viterbi_matrix, state_paths, newpath_buffer, current_start_probabilities, markov_model }
    }

    fn compute_transition_probability(&self, observation: &u8, state: &u8) -> f32 {

    }

    fn compute_time_0_probabilities(&mut self, observations: &[u8]) {
        for state in 0..self.markov_model.max_state { // initialize start probabilities
            self.viterbi_matrix[0][state as usize] = self.current_start_probabilities[state as usize] * 
                self.markov_model.transition_matrix[state][]
                //self.markov_model.emission_probabilities[state as usize][observations[0] as usize];
            self.state_paths[state as usize][0] = state;
        }
    }

    fn inductive_step(&mut self, observations: &[u8]) {
        for time_step in 1..observations.len() {
            for state in 0..self.markov_model.max_state {
                let mut max_probability: f32 = 0.0;
                let mut max_state: u8 = 0;

                for state_internal in 0..self.markov_model.max_state { // find the highest probability source for the state at this time
                    let probability = 
                        self.viterbi_matrix[time_step - 1][state_internal as usize] * 
                        self.markov_model.transition_matrix[state_internal as usize][state as usize] * 
                        self.markov_model.emission_probabilities[state as usize][observations[time_step] as usize];

                    if probability > max_probability {
                        max_probability = probability;
                        max_state = state_internal;
                    }
                }

                self.viterbi_matrix[time_step][state as usize] = max_probability;
                let mut temp_path = self.state_paths[max_state as usize].clone();
                temp_path[time_step] = state;

                self.newpath_buffer[state as usize] = temp_path;
            }

            self.state_paths = self.newpath_buffer.clone();
        }
    }

    fn identify_best_path(&mut self) -> (f32, Vec<u8>) {
        let final_probabilities = self.viterbi_matrix.iter().last().unwrap();
        let mut highest_probability_final_state: u8 = 0;
        let mut highest_final_probability = 0.0;

        for state in 0..self.markov_model.max_state {
            if final_probabilities[state as usize] > highest_final_probability {
                highest_final_probability = final_probabilities[state as usize];
                highest_probability_final_state = state;
            }
        }

        return (highest_final_probability, self.state_paths[highest_probability_final_state as usize].clone());
    }

    fn update_start_probabilities(&mut self) {
        self.current_start_probabilities = self.viterbi_matrix.last().unwrap().clone();
    }

    pub fn viterbi(&mut self, observations: &[u8]) -> (f32, Vec<u8>) {
        self.compute_time_0_probabilities(observations);
        self.inductive_step(observations);

        self.update_start_probabilities();

        return self.identify_best_path();
    }
}