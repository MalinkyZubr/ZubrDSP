use std::sync::mpsc;
use num::complex::Complex;

pub trait BufferType {}

impl BufferType for Complex<f32> {}
impl BufferType for Vec<Complex<f32>> {} // make the size runtime based


// Each step is responsible for preparing its data for the next
pub trait OutputAdapter<IN: BufferType, OUT: BufferType> {
    fn push(&mut self);
}

pub struct NonConvertingAdapter<IN: BufferType, OUT: BufferType> {
    in_receiver: mpsc::Receiver<IN>,
    out_sender: mpsc::Sender<OUT>,
}

pub struct TimeToFrequencyAdapter<IN: BufferType, OUT: BufferType> {
    in_receiver: mpsc::Receiver<IN>,
    out_sender: mpsc::Sender<OUT>,
    copy_buffer: Vec<Complex<f32>>,
    // active_buffer: Vec<Complex<f32>>,
    buff_size: usize,
    counter: usize,
}

pub struct FrequencyToTimeAdapter<IN: BufferType, OUT: BufferType> {
    in_receiver: mpsc::Receiver<IN>,
    out_sender: mpsc::Sender<OUT>,
    buff_size: usize,
}

impl<DataType: BufferType> OutputAdapter<DataType, DataType> for NonConvertingAdapter<DataType, DataType> {
    fn push(&mut self) {
        let in_data: DataType = self.in_receiver.recv().unwrap();

        match self.out_sender.send(in_data) {
            Ok(()) => {}
            Err(msg) => {}
        }
    }
}

impl OutputAdapter<Complex<f32>, Vec<Complex<f32>>> for TimeToFrequencyAdapter<Complex<f32>, Vec<Complex<f32>>> {
    fn push(&mut self) {
        let in_data: Complex<f32> = self.in_receiver.recv().unwrap();
        self.copy_buffer[self.counter] = in_data; // dereference for copy
        
        if self.counter == self.buff_size - 1 {
            self.counter = 0;

            match self.out_sender.send(self.copy_buffer.clone()) { // this is inefficient. Later on have a second field with pre-allocated space ready
                // that can store this while consumed by the next step so that it doesnt go out of scope. Pre-allocated cloning
                Ok(vec) => {}
                Err(msg) => {}
            }
        }
        else {
            self.counter += 1;
        }
    }
}

impl OutputAdapter<Vec<Complex<f32>>, Complex<f32>> for FrequencyToTimeAdapter<Vec<Complex<f32>>, Complex<f32>> {
    fn push(&mut self) {        
        let in_data: Vec<Complex<f32>> = self.in_receiver.recv().unwrap();
        let mut index: usize = 0;

        while index < self.buff_size {   
            match self.out_sender.send(in_data[index]) {
                Ok(()) => {}
                Err(msg) => {}
            }

            index += 1;
        }
    }
}
