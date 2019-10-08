/*
   Rust study program.
   This is 1st program.

   hidekuno@gmail.com
*/
use rand::Rng;

fn qsort(data: &mut [i32], low: usize, high: usize) {
    let (mut l, mut r) = (low, high);
    let mid = data[((low + high) / 2) as usize];

    loop {
        while data[l] < mid {
            l += 1;
        }
        while data[r] > mid {
            r -= 1;
        }
        if l > r {
            break;
        }
        data.swap(l, r);
        l += 1;
        if 0 < r {
            r -= 1;
        }
    }
    if low < r {
        qsort(data, low, r);
    }
    if l < high {
        qsort(data, l, high);
    }
}

fn main() {
    let mut rng = rand::thread_rng();

    // stack
    const N: usize = 32;
    let mut stack: [i32; N] = [0; N];

    for i in 0..N {
        let y: i32 = rng.gen();
        stack[i] = y.abs() / 100000;
    }
    qsort(&mut stack, 0, N - 1);
    println!("{:?}", stack);

    // heap
    let mut heap = Vec::new();
    for _ in 0..N {
        let y: i32 = rng.gen();
        heap.push(y.abs() / 100000);
    }
    qsort(&mut heap, 0, N - 1);
    println!("{:?}", heap);
}
