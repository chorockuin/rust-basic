/*
컴파일 타임에 동시성 문제를 발견할 수 있도록 하는 것이 목표
1. 쓰레드 생성 기초
2. 쓰레드 간 메세지 전달
3. 각 쓰레드에서의 공유 데이터 접근
4. 동시성 관련 Sync, Send 트레잇
*/

pub fn sample() {
    base();
    join();
    closure();
    channel();
    send_vector_vals();
    mutex();
    thread_mutex();
}

use std::thread;
use std::time::Duration;
fn base() {
    /*
    쓰레드를 생성하는 spawn() 함수에 쓰레드의 동작을 나타내는 클로져(익명 함수)를 넘김으로써 쓰레드 생성
    단, 클로저에 넘기는 파라미터 개수는 0개로 고정되어 있기 때문에 ||로 표시
    */
    thread::spawn(|| {
        for i in 1..10 {
            println!("hi number {} from spawned thread!", i);
            thread::sleep(Duration::from_millis(1)); // context switching
        }
    });

    for i in 1..5 {
        println!("hi number {} from the main thread!", i);
        thread::sleep(Duration::from_millis(1));
    }
}

fn join() {
    let handle = thread::spawn(|| {
        for i in 1..10 {
            println!("hi number {} from the spawned thread!", i);
            thread::sleep(Duration::from_millis(1));
        }
    });

    // handle.join().unwrap();

    for i in 1..5 {
        println!("hi number {} from the main thread!", i);
        thread::sleep(Duration::from_millis(1));
    }

    handle.join().unwrap(); // 생성한 쓰레드가 종료되기를 기다림
}

fn closure() {
    fn main() {
        let v = vec![1, 2, 3];
        /*
        spawned thread에서 v값을 사용하려고 함
        main thread, spawned thread에서 v가 공유되므로 당연히 문제 발생의 여지가 있음
        이를 위해 move 키워드를 사용하며, v의 소유권을 spawned thread로 이동 시켜버림
        */
        let handle = thread::spawn(move || {
            println!("Here's a vector: {:?}", v);
        });

        // drop(v); // 이미 thread에서 v의 소유권을 가져갔기 때문에 compile error
    
        handle.join().unwrap();
    }
}

use std::sync::mpsc;
fn channel() {
    // channel을 생성하면 Sender, Receiver를 갖고 있는 튜플 객체 반환
    let (tx, rx) = mpsc::channel();

    // spawned thread는 tx(Sender<String>)를 move로 넘겨받아 "hi" 문자열 전송
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(5000));
        let val = String::from("hi");
        // send하면서 val의 소유권은 main thread로 move됨
        tx.send(val).unwrap();
        // println!("val is {}", val); // 따라서 val은 유효하지 않기 때문에 compile error
    });

    println!("receiving...");
    // main thread는 Receiver를 가지고 수신 대기
    let received = rx.recv().unwrap(); // 동기 대기
    // let received = rx.try_recv().unwrap(); // 비동기 대기
    println!("Got: {}", received);
}

fn send_vector_vals() {
    let (tx, rx) = mpsc::channel();
    /*
    tx는 첫번째 thread에서 move되어 소유권이 넘어가므로 두번째 thread에서는 사용할 수 없음
    따라서 tx를 복사해서 두번째 thread로 move함
    tx, rx를 따로 만들어서 사용해도 되지만, 단일 rx에서 메세지들을 핸들링하고 싶을 때 위와 같이 함
    */
    let tx_clone = mpsc::Sender::clone(&tx);

    thread::spawn(move || {
        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];

        for val in vals {
            tx.send(val).unwrap(); // vals 내 string들의 소유권이 다 main thread로 넘어감
            thread::sleep(Duration::from_secs(1));
        }
    });

    thread::spawn(move || {
        let vals = vec![
            String::from("more"),
            String::from("message"),
            String::from("for"),
            String::from("you"),
        ];

        for val in vals {
            tx_clone.send(val).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    // rx.recv() 하지 않고 rx를 반복자처럼 다룰 수 있음
    // channel이 닫히면 반복도 종료 됨
    for received in rx {
        println!("Got: {}", received);
    }
}

use std::sync::Mutex;

fn mutex() {
    // Mutex<T>는 뜯어보면 스마트포인터다. 결국 스마트포인터 m이 immutable하다는 얘기
    let m = Mutex::new(5);
    {
        // lock()의 리턴값(num)을 내부 값(m)에 대한 가변 참조자 처럼 다룰 수 있음
        let mut num = m.lock().unwrap();
        // 가변 참조자를 역참조해서 값을 변경
        *num = 6;
    } // 스코브 밖으로 나오면서 unlock 됨

    println!("m = {:?}", m);
}

use std::rc::Rc;
use std::sync::Arc;
fn thread_mutex() {
    // let counter = Mutex::new(0); // counter는 int를 담고 있는 Mutex
    // let counter = Rc::new(Mutex::new(0)); // counter는 int를 담고 있는 Mutex의 Reference Counter
    let counter = Arc::new(Mutex::new(0)); // counter는 int를 담고 있는 Mutex의 Atomic Reference Counter
    let mut handles = vec![];

    for _ in 0..10 {
        // 첫번째 loop에서 counter가 move되어 버렸기 때문에 두번째 loop에서 counter를 move시킬 수 없어 컴파일 에러 발생
        // let handle = thread::spawn(move || {
        //     let mut num = counter.lock().unwrap();
        //     *num += 1;
        // });

        /*
        counter를 공유 스마트 포인터인 Rc<>를 사용해 복제한 후 move시켜본다
        그러나 역시나 에러난다. 이유는 counter를 threadsafe하게 move시킬 수 없다는 것
        Rc<>의 read/write가 threadsafe하도록 Send trait이 구현되어 있어야 함
        이를 만족하는 것이 atomic reference counting. 즉, Arc<T> 이다
        */
        // let counter = Rc::clone(&counter);
        // let handle = thread::spawn(move || {
        //     let mut num = counter.lock().unwrap();
        //     *num += 1;
        // });

        /*
        atomic reference counting는 reference counting 시에 threadsafe를 제공함
        참고로 Mutex<T>/Arc<T>의 관계는 마치 RefCell<T>/Rc<T> 의 관계와 비슷하다
        Rc<T>의 내용을 변경하고자 RefCell<T>를 사용했듯이
        Arc<T>의 내용을 변경하고자 Mutex<T>를 사용했다
        */
        let counter = Arc::clone(&counter); // counter 복사. counter가 담고있는 int의 메모리 영역을 복사하는 것이 아님. 그냥 포인터만 복사하는 것
        let handle = thread::spawn(move || { // 복사된 counter가 move됨
            let mut num = counter.lock().unwrap(); // counter가 담고있는, Mutex로 보호되는 int값의 참조자를 가져옴
            *num += 1; // 참조자를 역참조해서 값을 변경함
        });

        handles.push(handle);
    }

    // 모든 쓰레드가 종료되길 기다림
    for handle in handles {
        handle.join().unwrap();
    }

    // count 값 확인
    println!("Result: {}", *counter.lock().unwrap());
}

// Send 트레잇, Sync 트레잇을 구현하면 동시성을 지원하는 type을 만들 수 있음
// Send -> 현재 thread 내의 객체 소유권을 새로 생성하는 thread로 안전하게 move할 수 있도록 구현함
// Sync -> 여러 thread에서 접근할 수 있도록 구현함
// Rust의 대부분 타입은 Send 트레잇을 구현하고 있으나 위에서 보다시피 Rc<T> 같은 경우에는 구현하지 않고 있음