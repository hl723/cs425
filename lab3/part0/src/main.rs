
// use std::convert::TryInto;
// use std::env;
// use std::sync::{Mutex, Arc};
// use std::time::Instant;
// use std::thread;

// extern crate num_cpus;


// fn compute_primes(i: i64) -> i64 {
//     let mut is_prime: bool = true;

//     for j in 2..i-1 {
//         if i % j == 0 {
//             is_prime = false;
//             return 0;
//         }
//     }
//     return 1;
// }

// fn main() {
//     let args: Vec<String> = env::args().collect();
//     let cpus: i64 = num_cpus::get().try_into().unwrap();
//     let n: i64 = args[1].parse().unwrap();
//     // let count = Arc::new(Mutex::new(0 as i64));
//     let mut count = 0;
//     let mut is_prime: bool = true;
//     // let mut thread_handles = vec![];

//     let wall_start = Instant::now();

//     for i in 2..n {
//         // let count = Arc::clone(&count);
//         // let handle = thread::spawn(move || {
//         //     let num_primes: i64 = compute_primes(i);

//         //     if num_primes == 1 {
//         //         let mut num = count.lock().unwrap();
//         //         *num += num_primes;
//         //     }
//         // });
//         // thread_handles.push(handle);
//         is_prime = true;
//         let end: i64 = (i as f64).sqrt() as i64 + 1;
//         // println!("{}, {}", i, end);
//         for j in 2..end {
//             if i % j == 0 {
//                 is_prime = false;
//                 break;
//             }
//         }
//         // println!("{}, {}", i, is_prime);
//         if is_prime == true {
//             count += 1;
//         }
//     }

//     // for handle in thread_handles {
//     //     handle.join().unwrap();
//     // }

//     let elapsed = wall_start.elapsed();
//     println!("{numPrimes}   {time}", numPrimes=count, time=elapsed.as_nanos());
//     // println!("{numPrimes}   {time}", numPrimes=*count.lock().unwrap(), time=elapsed.as_nanos());
// }


use std::convert::TryInto;
use std::env;
use std::sync::{Mutex, Arc};
use std::time::Instant;
use std::thread;

extern crate num_cpus;


fn isprime(n: i64) -> bool {
    if n == 2 || n == 3 {
        return true;
    }
    else if n % 2 == 0 || n % 3 == 0 {
        return false
    }
    
    let mut i: i64 = 5;
    while i * i <= n {
        if (n % i == 0) || (n % (i + 2) == 0) {
            return false;
        }
        i += 6;
    }
    return true;
}

fn compute_primes(start: i64, end: i64) -> i64 {
    let mut counter: i64 = 0;

    for i in start..end {
        if isprime(i) == true {
            counter += 1
        }
    }
    return counter;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cpus: i64 = num_cpus::get().try_into().unwrap();
    let n: i64 = args[1].parse().unwrap();
    let count = Arc::new(Mutex::new(0 as i64));
    let step_size: i64 = (n-2)/cpus;
    let mut thread_handles = vec![];
    let wall_start = Instant::now();

    for thread_num in 0..cpus {
        let count = Arc::clone(&count);
        let handle = thread::spawn(move || {
            let start = (thread_num * step_size) + 2;
            let end = if thread_num != cpus - 1 {start + step_size} else {n-1};
            let num_primes: i64 = compute_primes(start, end);
            let mut num = count.lock().unwrap();
            *num += num_primes;
        });
        thread_handles.push(handle);
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
    
    // let n = compute_primes(2, n);

    let elapsed = wall_start.elapsed();
        
    // println!("{numPrimes}   {time:?}", numPrimes=n, time=elapsed);
    println!("{numPrimes}   {time:?}", numPrimes=*count.lock().unwrap(), time=elapsed.as_nanos());
}
