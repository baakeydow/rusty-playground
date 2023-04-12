//! rusty_lib
#![deny(missing_docs)]
// #![forbid(unsafe_code)]

pub mod dtkutils;
pub mod dtkexamples;
pub mod dtkchat;
pub mod dtkmongo;
pub mod dtkpocket;

#[cfg(test)]
mod rusty_lib_main_tests {

    #[test]
    fn unsafe_box() {
        let heap_str = Box::new(String::new());
        let heap_ptr = Box::into_raw(heap_str);
    
    
        // when you want managed memory again:
        let box_again = unsafe { Box::from_raw(heap_ptr) };
        println!("Raw pointers: {:p} === {:p}", heap_ptr, box_again);
        unsafe {
            assert_eq!(*heap_ptr, *box_again);
        }
    }

    #[test]
    fn unsafe_num() {
        let mut num = 5;
        let r1 = &num as *const i32;
        let r2 = &mut num as *mut i32;
        unsafe {
            println!("r1 is: {:?}", *r1);
            println!("r2 is: {:?}", *r2);
        }
    }

    #[test]
    fn multiple_owners() {
        use std::rc::Rc;
        use std::cell::RefCell;
        let value = Rc::new(RefCell::new(5));
        let a = Rc::clone(&value);
        let b = Rc::clone(&value);
        *a.borrow_mut() += 1;
        *b.borrow_mut() += 1;
        assert_eq!(*value.borrow(), 7);
    }

    #[test]
    fn simple_combinators() {
        let initial_data = vec!["TEST 0", "test 1", "test 2"];
        let processed_data = initial_data
            .iter()
            .map(|s| s.to_owned())
            .filter(|s| s.to_lowercase().contains("test"))
            .collect::<Vec<_>>();
        assert_eq!(initial_data, processed_data);
    }

    #[test]
    fn read_file() {
        use crate::dtkutils::utils;
        use std::env;
        let mut dir = env::current_dir().unwrap();
        dir.push("src/dtkutils/utils.rs");
        let res = utils::get_file_as_string(dir.to_str().unwrap());
        if res.is_err() {
            panic!("{:#?}", res.err().unwrap());
        }
        assert_eq!(true, res.is_ok());
    }

    #[test]
    fn arr_addr() {
        let arr1 = [1, 2, 3];
        let arr2 = &arr1;
        assert_eq!(&arr1, arr2);
    }

    #[test]
    fn emoji() {
        assert_eq!('\u{1F600}', '😀');
    }

    #[test]
    fn to_double() {
        use crate::dtkutils::utils;
        utils::log_args();
        // IndexMut & MulAssign
        let mut to_double = [1, -2, 42, 0, 100, 84, 21, 51, 23, 64];
        utils::double(&mut to_double);
        utils::double(&mut to_double);
        assert_eq!([4, -8, 168, 0, 400, 336, 84, 204, 92, 256], to_double);
    }
}
