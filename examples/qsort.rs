/*
   Rust study program.
   This is multi-thread sample program.

   hidekuno@gmail.com
*/
#![allow(unsafe_code)]
use rand::Rng;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::thread;
use std::time::Instant;

const THRESHOLD: usize = 1024;
const CONCURRENCY: bool = true;
#[allow(dead_code)]
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
#[allow(dead_code)]
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

    if THRESHOLD < (high - low) && CONCURRENCY {
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
    } else {
        if low < r {
            con_qsort(&mut data1, low, r);
        }
        if l < high {
            con_qsort(&mut data2, l, high);
        }
    }
}
#[allow(dead_code)]
fn con2_qsort(ptr_data: &mut Arc<RwLock<Vec<i32>>>, low: usize, high: usize) {
    let (mut l, mut r) = (low, high);
    let mid = (low + high) / 2;
    loop {
        {
            let data = ptr_data.read().unwrap();
            while data[l] < data[mid] {
                l += 1;
            }
        }
        {
            let data = ptr_data.read().unwrap();
            while data[r] > data[mid] {
                r -= 1;
            }
        }
        if l > r {
            break;
        }
        {
            let mut data = ptr_data.write().unwrap();
            data.swap(l, r);
        }
        l += 1;
        if r > 0 {
            r -= 1;
        }
    }
    let mut data1 = ptr_data.clone();
    let mut data2 = ptr_data.clone();

    if THRESHOLD < (high - low) && CONCURRENCY {
        let t1 = thread::spawn(move || {
            if low < r {
                con2_qsort(&mut data1, low, r);
            }
        });
        let t2 = thread::spawn(move || {
            if l < high {
                con2_qsort(&mut data2, l, high);
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
    } else {
        if low < r {
            con2_qsort(&mut data1, low, r);
        }
        if l < high {
            con2_qsort(&mut data2, l, high);
        }
    }
}
#[allow(dead_code)]
unsafe fn con3_qsort(data: &mut Vec<i32>, low: usize, high: usize) {
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
    // bad inefficiency
    let mut data1 = data.clone();
    let mut data2 = data.clone();

    if THRESHOLD < (high - low) && CONCURRENCY {
        let t1 = thread::spawn(move || {
            if low < r {
                con3_qsort(&mut data1, low, r);
            }
        });
        let t2 = thread::spawn(move || {
            if l < high {
                con3_qsort(&mut data2, l, high);
            }
        });
        t1.join().unwrap();
        t2.join().unwrap();
    } else {
        if low < r {
            con3_qsort(&mut data1, low, r);
        }
        if l < high {
            con3_qsort(&mut data2, l, high);
        }
    }
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
    const M: usize = 1000000;
    let mut heap = Vec::new();
    for _ in 0..M {
        let y: i32 = rng.gen();
        heap.push(y.abs() / 100000);
    }

    let start = Instant::now();
    //let mut ptr_data = Arc::new(Mutex::new(heap));
    //con_qsort(&mut ptr_data, 0, M - 1);
    //let mut ptr_data = Arc::new(RwLock::new(heap));
    //con2_qsort(&mut ptr_data, 0, M - 1);

    unsafe {
        con3_qsort(&mut heap, 0, 1000);
    }
    let end = start.elapsed();
    println!("{}.{:06}", end.as_secs(), end.subsec_nanos() / 1_000_000);
}
