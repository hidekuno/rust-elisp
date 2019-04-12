/*
   Rust study program.
   This is multi-thread sample program.

   hidekuno@gmail.com
*/

use rand::Rng;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

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
fn con_qsort(ptr_data: &mut Arc<Mutex<Vec<i32>>>, low: usize, high: usize) {
    let (mut l, mut r) = (low, high);
    {
        let mut data = ptr_data.lock().unwrap();
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
            if r > 0 {
                r -= 1;
            }
        }
    }
    let mut data1 = ptr_data.clone();
    let mut data2 = ptr_data.clone();
    let t1 = thread::spawn(move || {
        if low < r {
            con_qsort(&mut data1, low, r);
        }
    });
    let t2 = thread::spawn(move || {
        if l < high {
            con_qsort(&mut data2, l, high);
        }
    });
    t1.join().unwrap();
    t2.join().unwrap();
}

fn main() {
    // stack
    let mut rng = rand::thread_rng();

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
    let mut ptr_data = Arc::new(Mutex::new(heap));
    con_qsort(&mut ptr_data, 0, N - 1);

    let data = ptr_data.lock().unwrap();
    for i in data.iter() {
        println!("{}", i);
    }
}
