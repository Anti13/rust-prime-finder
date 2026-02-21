use std::mem;
use std::time::Instant;
use thousands::Separable;

fn main() {

    let limit = 2000_000_000;
    println!("Finding all rpimes up to {}", limit.separate_with_spaces());

    let start = Instant::now();

    let mut is_prime = vec![true; limit + 1];

    is_prime[0] = false;
    is_prime[1] = false;

    let mut p = 2;

    while p*p <= limit {
        if is_prime[p]
        {
            let mut i = p*p;
            while i<= limit {
                is_prime[i] = false;
                i+=p;
            }
        }
        p+=1;
    }

    let duration = start.elapsed();
    let mem_used = (limit + 1) * mem::size_of::<bool>();
    let count = is_prime.iter().filter(|&&p|p).count();

    println!("-------------------------------");
    println!("Found {} primes", count.separate_with_spaces());
    println!("Time taken: {:?}", duration);
    println!("Memory used by is_prime: {} bytes ({:.2} MB)", mem_used, mem_used as f64 / 1024.0 / 1024.0);

}