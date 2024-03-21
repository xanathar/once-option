use std::string::String;
use std::sync::atomic::{AtomicU32, Ordering};

use alloc::string::ToString;

use crate::OnceOption;

#[test]
fn wrapping_a_thread_join_handle() {
    use std::{thread, time::Duration};

    static SHARED_RESULT: AtomicU32 = AtomicU32::new(5);

    struct SomeType {
        handle: OnceOption<thread::JoinHandle<u32>>,
    }

    impl SomeType {
        fn new() -> Self {
            Self {
                handle: thread::spawn(|| {
                    thread::sleep(Duration::from_millis(50));
                    42
                })
                .into(),
            }
        }

        fn thread_id(&self) -> thread::ThreadId {
            self.handle.thread().id()
        }
    }

    impl Drop for SomeType {
        fn drop(&mut self) {
            SHARED_RESULT.store(self.handle.take().join().unwrap(), Ordering::Release);
        }
    }

    {
        let t = SomeType::new();
        let _ = t.thread_id();
    }

    assert_eq!(SHARED_RESULT.load(Ordering::Acquire), 42);
}

#[test]
fn impl_deref() {
    let s: OnceOption<String> = "test123".to_string().into();
    assert_eq!(s.as_bytes()[2], b's');
}

#[test]
fn impl_deref_mut() {
    let mut s: OnceOption<String> = "test123".to_string().into();
    assert_eq!(String::from("test123"), "test123");

    let t = s.split_off(4);

    assert_eq!(&*s, "test");
    assert_eq!(t, "123");
}

#[test]
fn impl_partial_eq() {
    let s: OnceOption<String> = OnceOption("123".to_string());
    let t: OnceOption<String> = OnceOption(format!("1{}3", 2));

    assert_eq!(s, t);
}

#[test]
fn impl_partial_ord() {
    let s: OnceOption<u32> = OnceOption(12);
    let t: OnceOption<u32> = OnceOption(19);

    assert!(s < t);
    assert!(s <= t);
    assert!(t > s);
    assert!(t >= s);
}

#[test]
fn impl_ord() {
    let mut v = vec![
        OnceOption(27),
        OnceOption(12),
        OnceOption(94),
        OnceOption(19),
    ];

    v.sort();

    assert_eq!(v[0].take(), 12);
    assert_eq!(v[1].take(), 19);
    assert_eq!(v[2].take(), 27);
    assert_eq!(v[3].take(), 94);
}

#[test]
fn impl_display_integer_formats() {
    let f = OnceOption(12648430u64);
    assert_eq!(format!("{}", f), "12648430");
    assert_eq!(format!("{:X}", f), "C0FFEE");
    assert_eq!(format!("{:x}", f), "c0ffee");
    assert_eq!(format!("{:#06o}", f), "0o60177756");
    assert_eq!(format!("{:b}", f), "110000001111111111101110");
}

#[test]
fn impl_display_float_formats() {
    let f = OnceOption(3.141592f64);
    assert_eq!(format!("{}", f), "3.141592");
    assert_eq!(format!("{:E}", f), "3.141592E0");
    assert_eq!(format!("{:e}", f), "3.141592e0");
    assert_eq!(format!("{:0.02}", f), "3.14");
    assert_eq!(format!("{:+0.02}", f), "+3.14");
}

#[test]
fn impl_debug() {
    let v = vec![1, 2, 3];
    let o: OnceOption<_> = v.clone().into();

    assert_eq!(format!("{:?}", o), format!("OnceOption({:?})", v));
    assert_eq!(
        format!("{:?}", OnceOption::<u32>::NONE),
        format!("OnceOption::NONE")
    );
}
