//! Wrapper type for by-address hashing and comparison.
use std::ops::Deref;

#[derive(Copy, Clone, Debug, Default)]
/// Wrapper for pointer types that implements by-address comparison.
pub struct ByAddress<T>(pub T)
where
    T: ?Sized + Deref;

impl<T> ByAddress<T>
where
    T: ?Sized + Deref,
{
    /// Convenience method for pointer casts.
    pub fn addr(&self) -> *const T::Target {
        &*self.0
    }
}

/// Raw pointer equality
impl<T> PartialEq for ByAddress<T>
where
    T: ?Sized + Deref,
{
    fn eq(&self, other: &Self) -> bool {
        self.addr() == other.addr()
    }
}

#[test]
fn check_by_address() {
    use super::*;
    use std::rc::Rc;
    let ref_counted = Rc::new([1, 2, 3]);
    let ref_counted1 = by_address::ByAddress(ref_counted.clone());
    let ref_counted2 = by_address::ByAddress(ref_counted.clone());
    let ref_counted3 = by_address::ByAddress(Rc::new([1, 2, 3]));
    println!("ref_counted1: {:p}, ref_counted2: {:p}", ref_counted1.addr(), ref_counted2.addr());
    assert_eq!(ref_counted1.addr(), ref_counted2.addr());
    println!("ref_counted1: {:p}, ref_counted3: {:p}", ref_counted1.addr(), ref_counted3.addr());
    assert_ne!(ref_counted1.addr(), ref_counted3.addr());
}

#[test]
fn address_as_string() {
    use crate::dtkutils::utils;
    use std::rc::Rc;
    let ref_counted = Rc::new(5);
    let address = utils::get_address_as_string(&ref_counted);
    assert_eq!(address, format!("{ref_counted:p}"));
}
