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

    for s in stack.iter_mut().take(N) {
        let y: i32 = rng.gen();
        *s = y.abs() / 100000;
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

#[allow(dead_code)]
const RESULT: &str = "[5, 8, 12, 16, 18, 21, 23, 25, 28, 29, 30, 45, 46, 46, 50, 54, 55, 58, 59, 59, 62, 62, 68, 85, 85, 85, 86, 86, 87, 91, 95, 97]";

#[test]
fn test_qsort_statck() {
    const N: usize = 32;
    let mut stack: [i32; N] = [
        54, 85, 97, 12, 50, 45, 8, 59, 21, 68, 55, 16, 28, 85, 30, 46, 86, 62, 25, 87, 85, 46, 29,
        58, 23, 5, 86, 95, 62, 18, 59, 91,
    ];
    qsort(&mut stack, 0, N - 1);
    assert_eq!(format!("{:?}", stack), RESULT);
}
#[test]
fn test_qsort_heap() {
    const N: usize = 32;
    let stack: [i32; N] = [
        54, 85, 97, 12, 50, 45, 8, 59, 21, 68, 55, 16, 28, 85, 30, 46, 86, 62, 25, 87, 85, 46, 29,
        58, 23, 5, 86, 95, 62, 18, 59, 91,
    ];

    let mut heap = Vec::new();

    for s in stack.iter().take(N) {
        heap.push(*s);
    }
    qsort(&mut heap, 0, N - 1);
    assert_eq!(format!("{:?}", heap), RESULT);
}
