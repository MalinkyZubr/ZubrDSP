pub mod parallel_computation_tester {
    use crate::general::parallel_computation::{ComputeModule, ParallelComputation};
    use std::{sync::{Arc, Mutex, RwLock}, time::Duration};
    use rand::Rng;
    use std::thread;


    fn test_op(input: i64) -> i64 {
        let mut result = input;
        for x in 0..1000 {
            thread::sleep(Duration::from_nanos(10));
            result += 1;
        }

        return result;
    }

    fn appender(output: i64, output_vec: &mut Arc<RwLock<Vec<i64>>>) {
        output_vec.write().unwrap().push(output);
    }

    #[test]
    pub fn test_parallel_computation() {
        let results: Arc<RwLock<Vec<i64>>> = Arc::new(RwLock::new(Vec::new()));

        let test_compute_module: ComputeModule<i64, Arc<RwLock<Vec<i64>>>, i64> = ComputeModule::new(
            test_op,
            Some(appender), 
            Some(results.clone())
        );

        let mut runner = ParallelComputation::new(
            1000, test_compute_module
        );

        let mut rnger = rand::rng();

        let mut start_vec: Vec<i64> = Vec::new();

        for _index in 0..2000 {
            let val = rnger.random_range(0..4000);
            runner.add_task(
                val
            );
            start_vec.push(val);
        }

        runner.start();
        runner.wait_until_complete();
        runner.stop();

        //thread::sleep(Duration::from_millis(1000));

        //thread::sleep(Duration::from_millis(2000));

        let unwrapped_results = results.read().unwrap();

        //dbg!(&unwrapped_results);

        //let mut index = 0;

        for value in start_vec {
            assert!(unwrapped_results.contains(&(value + 1000)));
            //index += 1;
            //dbg!(index);
        }
    }
}

