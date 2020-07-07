#[macro_use]
extern crate criterion;
extern crate itertools;
extern crate rand;
extern crate rayon;
extern crate rayon_try_fold;

use rand::prelude::*;
use rayon_try_fold::{slice_par_sort, slice_sort_jc_jc};
use std::time::Duration;

use criterion::{Criterion, ParameterizedBenchmark};

const PROBLEM_SIZE: u32 = 100_000_000;

fn sort_benchmarks(c: &mut Criterion) {
    //let sizes: Vec<u32> = vec![100_000];
    let threads: Vec<usize> = vec![30, 40, 50, 60];
    c.bench(
        "random input",
        ParameterizedBenchmark::new(
            "slice sort JC JC",
            |b, nt| {
                b.iter_with_setup(
                    || {
                        let tp = rayon::ThreadPoolBuilder::new()
                            .num_threads(*nt)
                            .build()
                            .expect("Couldn't build thread pool");
                        let mut input = (0..PROBLEM_SIZE).collect::<Vec<_>>();
                        let mut rng = rand::thread_rng();
                        input.shuffle(&mut rng);
                        (tp, input)
                    },
                    |(tp, mut input)| {
                        tp.install(|| {
                            slice_sort_jc_jc(&mut input);
                            input
                        });
                    },
                )
            },
            threads.clone(),
        )
        .with_function("slice sort JC adaptive", |b, nt| {
            b.iter_with_setup(
                || {
                    let tp = rayon::ThreadPoolBuilder::new()
                        .num_threads(*nt)
                        .build()
                        .expect("Couldn't build thread pool");
                    let mut input = (0..PROBLEM_SIZE).collect::<Vec<_>>();
                    let mut rng = rand::thread_rng();
                    input.shuffle(&mut rng);
                    (tp, input)
                },
                |(tp, mut input)| {
                    tp.install(|| {
                        slice_par_sort(&mut input);
                        input
                    });
                },
            )
        }),
    );
}

criterion_group! {
    name = benches;
            config = Criterion::default().sample_size(15).warm_up_time(Duration::from_secs(1)).nresamples(1000);
                targets = sort_benchmarks
}
criterion_main!(benches);