#[cfg(feature = "logs")]
fn main() {
    use kvik::prelude::*;
    let pool = rayon_logs::ThreadPoolBuilder::new()
        .build()
        .expect("failed creating pool");
    let (_, log) = pool.logging_install(|| {
        // let's search if the range contains 4_999_999
        let s = (0..10_000_000u64)
            .into_par_iter()
            .map(|e| if e == 4_999_999 { Err(e) } else { Ok(()) })
            // .try_fold(
            //     || Ok(()),
            //     |_, e| if e == 4_999_999 { Err(e) } else { Ok(()) },
            // )
            .size_limit(10_000)
            .by_blocks(std::iter::successors(Some(2usize), |l| {
                Some(l.saturating_mul(2))
            }))
            .rayon(3)
            .try_reduce(|| (), |_, _| Ok(()));
        assert_eq!(s, Err(4_999_999));
    });
    log.save_svg("try_reduce.svg")
        .expect("failed saving execution trace");
}
#[cfg(not(feature = "logs"))]
fn main() {
    println!("you should run me with the logs feature");
}
