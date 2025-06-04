pub mod parallel_computation_tester {
    use rand::Rng;

    use crate::general::parallel_computation::*;
    use std::{sync::{Arc, Mutex, RwLock}, thread, time::Duration};


    fn test_op(input: i64) -> i64 {
        input * 4
    }

    fn appender(output: i64, output_vec: &mut Arc<RwLock<Vec<i64>>>) {
        output_vec.write().unwrap().push(output);
    }

    #[test]
    pub fn test_parallel_computation() {
        let mut results: Arc<RwLock<Vec<i64>>> = Arc::new(RwLock::new(Vec::new()));

        let mut test_compute_module: ComputeModule<i64, Arc<RwLock<Vec<i64>>>, i64> = ComputeModule::new(
            test_op,
            appender, 
            results.clone()
        );

        let mut runner = ParallelComputation::new(
            5, test_compute_module
        );

        let mut rnger = rand::rng();

        let mut start_vec: Vec<i64> = Vec::new();

        for _index in 0..200000 {
            let val = rnger.random_range(0..4000);
            runner.add_task(
                val
            );
            start_vec.push(val);
        }

        runner.start();

        let mut lock = Mutex::new(false);
        let mut em = lock.lock().unwrap();

        while !*em {
            em = runner.is_empty.wait(em).unwrap();
}

        //thread::sleep(Duration::from_millis(2000));

        let unwrapped_results = results.read().unwrap();

        //dbg!(&unwrapped_results);

        for value in start_vec {
            assert!(unwrapped_results.contains(&(value * 4)));
        }

    }
}