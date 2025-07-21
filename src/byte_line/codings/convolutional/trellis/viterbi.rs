use crate::byte_line::codings::convolutional::trellis::trellis::*;
use crate::byte_line::codings::opts::*;
use crate::pipeline::api::*;
use std::mem;


enum ViterbiState {
    NotStarted,
    Standard
}


pub struct HiddenMarkovModel {
    pub max_state: u8,
    pub transition_matrix: Vec<Vec<u8>>,
    pub emission_matrix: Vec<Vec<u16>>,
}


impl HiddenMarkovModel {
    pub fn new(max_state: u8, transition_matrix: Vec<Vec<u8>>, emission_matrix: Vec<Vec<u16>>) -> Self {
        return HiddenMarkovModel { // add some checks here
            max_state,
            transition_matrix, 
            emission_matrix
        }
    }
}

pub struct ViterbiOpCore {
    viterbi_matrix: Vec<Vec<u32>>,
    state_paths: Vec<Vec<u8>>,
    newpath_buffer: Vec<Vec<u8>>,
    markov_model: HiddenMarkovModel,
    output_size: u8,
    state_machine: ViterbiState
}

impl ViterbiOpCore {
    pub fn new(buffer_size: usize, encoder_lookup: &ConvolutionalEncoderLookup) -> Self { // expect unpacked input observations
        let output_size = encoder_lookup.output_size;
        let num_states = encoder_lookup.encoding_lookup.len();
        let viterbi_matrix: Vec<Vec<u32>> = vec![vec![0; num_states]; buffer_size];

        let state_paths: Vec<Vec<u8>> = vec![vec![0; buffer_size]; num_states];
        let newpath_buffer: Vec<Vec<u8>> = vec![vec![0; buffer_size]; num_states];

        let markov_model: HiddenMarkovModel = HiddenMarkovModel::new(
            num_states as u8, 
            encoder_lookup.to_transition_matrix(), 
            encoder_lookup.to_emission_matrix());

        ViterbiOpCore { 
            viterbi_matrix, 
            state_paths, 
            newpath_buffer, 
            markov_model, 
            output_size,
            state_machine: ViterbiState::NotStarted
        }
    }

    fn compute_hamming_similarity(&self, observation: &u8, state: &usize, next_state: &usize) -> u32 {
        let transition_output: u16 = self.markov_model.emission_matrix[*state][*next_state];
        return self.output_size as u32 - hamming_distance((*observation, transition_output as u8)) as u32;
    }

    fn get_previous_timestep_similarity(&self, time_step: &usize, state: &usize) -> u32 {
        match time_step {
            0 => self.viterbi_matrix.last().unwrap()[*state],
            _ => return self.viterbi_matrix[time_step - 1][*state]
        }
    }

    fn identify_maximum_similarity_predecessor(&self, target_state: &usize, time_step: &usize, observations: &[u8]) -> (u32, u8) {
        let mut max_similarity: u32 = 0;
        let mut max_state: u8 = 0;
        
        for predecessor_state in 0..self.markov_model.max_state { // find the highest probability source for the state at this time
            if self.markov_model.transition_matrix[predecessor_state as usize][*target_state] == 1 {
                let similarity = 
                    self.get_previous_timestep_similarity(time_step, &(predecessor_state as usize)) + // this is the path parameter of the previous time step for the previous state
                    self.compute_hamming_similarity(&observations[*time_step], &(predecessor_state as usize), target_state);

                if similarity > max_similarity {
                    max_similarity = similarity;
                    max_state = predecessor_state;
                }
            }
        }

        return (max_similarity, max_state)
    }

    fn swap_path_vector_in_place(&mut self, swap_vec: Vec<u8>) {

    }

    fn execute_state_machine(&mut self, time_step: usize, observations: &[u8]) {
        match self.state_machine {
            ViterbiState::Standard => {
                for target_state in 0..self.markov_model.max_state {
                    let (max_similarity, max_state) = self.identify_maximum_similarity_predecessor(&(target_state as usize), &time_step, observations);

                    self.viterbi_matrix[time_step][target_state as usize] = max_similarity;
                    
                    //self.next_state_swap_vector[max_state as usize] = MAX; // might be problem later for large state machines

                    let mut temp_path = self.state_paths[max_state as usize].clone();

                    temp_path[time_step] = target_state;
                    self.newpath_buffer[target_state as usize] = temp_path;
                }

                //dbg!("{}", &self.next_state_swap_vector);
            },
            ViterbiState::NotStarted => {
                self.viterbi_matrix[0][0] = self.compute_hamming_similarity(&observations[0], &0, &0);
                self.state_machine = ViterbiState::Standard;
            }
        };
    }

    fn inductive_step(&mut self, observations: &[u8]) { // O(N^2 + T) N states, T observations
        // the question is: for every state at each time point, what is the most efficient way to get to that state?

        for time_step in 0..observations.len() {
            //dbg!("{}", &self.viterbi_matrix);

            self.execute_state_machine(time_step, observations);

            mem::swap(&mut self.state_paths, &mut self.newpath_buffer);

            //dbg!("{},{}", &self.state_paths, &self.viterbi_matrix);
        }
    }

    fn identify_best_path(&mut self) -> (u32, Vec<u8>) { // these outputs are not repacked. They are the raw states. Must pack back into u8 to be valid data!
        let final_similarities = self.viterbi_matrix.iter().last().unwrap();
        let mut highest_similarity_final_state: u8 = 0;
        let mut highest_final_similarity = 0;

        for state in 0..self.markov_model.max_state {
            if final_similarities[state as usize] > highest_final_similarity {
                highest_final_similarity = final_similarities[state as usize];
                highest_similarity_final_state = state;
            }
        }

        //dbg!("{}", &self.state_paths);

        return (highest_final_similarity, self.state_paths[highest_similarity_final_state as usize].clone());
    }

    pub fn viterbi(&mut self, observations: &[u8]) -> (u32, Vec<u8>) {
        self.inductive_step(observations);
        let best_path = self.identify_best_path();

        let viterbi_final_element = self.viterbi_matrix.len() - 1;
        self.viterbi_matrix.swap(0, viterbi_final_element);

        return best_path;
    }
}

impl PipelineStep<Vec<u8>, Vec<u8>> for ViterbiOpCore {
    fn run(&mut self, input: ReceiveType<Vec<u8>>) -> Result<Vec<u8>, String> {
        match input {
            ReceiveType::Single(value) => Ok(self.viterbi(&value).1),
            _ => Err(String::from("multi receive type not allowed"))
        }
    }
}