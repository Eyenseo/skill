// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
//

use std::any::TypeId;
use std::boxed::Box;
use std::cell::Cell;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::intrinsics::abort;
use std::marker::Unsize;
use std::ops::CoerceUnsized;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::NonNull;

use ptr::VALID_CASTS;

// NOTE std code adapted from 1.29.0
// NOTE depends on generated vtable lookup table

//////////////////////////////////////////
// Ptr - custom
//////////////////////////////////////////
#[derive(Debug)]
struct MetaData {
    cast_id: usize,
    strong: Cell<usize>,
    weak: Cell<usize>,
    borrow: Cell<BorrowFlag>,
}

pub struct Ptr<T>
where
    T: ?Sized,
{
    meta: NonNull<MetaData>,
    value: NonNull<T>,
}

impl<T> Ptr<T>
where
    // needed to obtain vtable lookup table index of original type
    T: 'static + CastAble,
{
    pub(crate) fn new(value: T) -> Ptr<T> {
        let p = Ptr {
            meta: Box::into_raw_non_null(Box::new(MetaData {
                cast_id: T::cast_id(),
                strong: Cell::new(1),
                weak: Cell::new(1),
                borrow: Cell::new(UNUSED),
            })),
            value: Box::into_raw_non_null(Box::new(value)), // NOTE this is bad for cache - would be nice if the box could be removed
        };
        p
    }
}

impl<T: ?Sized> Drop for Ptr<T> {
    fn drop(&mut self) {
        unsafe {
            self.dec_strong();
            if self.strong() == 0 {
                // destroy the contained object
                Box::from_raw(self.value.as_ptr());

                // remove the implicit "strong weak" pointer now that we've
                // destroyed the contents.
                self.dec_weak();

                if self.weak() == 0 {
                    Box::from_raw(self.meta.as_ptr());
                }
            }
        }
    }
}

#[macro_use]
mod diggsey {
    // Adapted from https://github.com/Diggsey/query_interface
    use super::*;

    /// Represents a trait object's vtable pointer. You shouldn't need to use this as a
    /// consumer of the crate but it is required for macro expansion.
    #[doc(hidden)]
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub(crate) struct VTable(*const ());

    unsafe impl Sync for VTable {}

    impl VTable {
        pub(crate) fn none() -> VTable {
            VTable(ptr::null())
        }
    }

    /// Represents a trait object's layout.
    #[doc(hidden)]
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub(crate) struct TraitObject {
        pub(crate) data: *const (),
        pub(crate) vtable: VTable,
    }
}

pub(crate) use self::diggsey::*;

pub trait CastAble {
    /// # Returns
    /// id / index into the vtable lookup table
    fn cast_id() -> usize;
}

impl<T> Ptr<T>
where
    T: ?Sized,
{
    /// Casts any `Ptr<T>` to `Ptr<U>` if the csat is valid base on the vtable lookup table
    pub fn cast<U>(&self) -> Option<Ptr<U>>
    where
        // Bound of U;
        //   U can be Sized, or Unsized
        //   U may only have references that live as long as the whole program
        //   U has to implement CastAble which is needed to call cast_id()
        U: ?Sized + 'static + CastAble,
    {
        // Obtain the lookup table index of Self
        let own_id = unsafe { self.meta.as_ref() }.cast_id;
        // Access the lookup table with the index of Self and U
        if let Some(vtable) = VALID_CASTS[own_id][U::cast_id()] {
            unsafe {
                let mut t = TraitObject {
                    // value is either a pointer to a Sized type e.g. Struct or of a trait object.
                    // In both cases points the pointer to data, a Sized type
                    data: self.value.as_ptr() as *const (),
                    vtable, // Set the vtable of U
                };
                let value = NonNull::new_unchecked(
                    // Reinterpret the memory pointed to by ts reference;
                    //   If U is Unsized, a Trait, then the type of the reference will
                    //     be reinterpreted to a real trait object of type U
                    //   If U is a Sized type, e.g. Struct the reference will be
                    //     reinterpreted as reference to U
                    //     The vtable pointer will be 'cut off' in this case
                    // NOTE mut-ref-ptr-to T to mut-ref-ptr-to U
                    *::std::mem::transmute::<_, &mut *mut U>(&mut t),
                );
                let meta = self.meta.as_ref();
                // Increase the strong counter for the reference counting
                meta.strong.set(meta.strong.get() + 1);
                // Construct a new Ptr<U> with the newly casted value and shared meta data
                Some(Ptr {
                    meta: self.meta,
                    value,
                })
            }
        } else {
            None // Return None to indicate a invalid cast
        }
    }
}

impl<T: ?Sized> Clone for Ptr<T> {
    /// Makes a clone of the `Ptr` pointer.
    ///
    /// This creates another pointer to the same inner value, increasing the
    /// strong reference count.
    #[inline]
    fn clone(&self) -> Ptr<T> {
        unsafe {
            let meta = self.meta.as_ref();
            meta.strong.set(meta.strong.get() + 1);
        }
        let p = Ptr {
            meta: self.meta,
            value: self.value,
        };
        p
    }
}

impl<T: ?Sized> !Sync for Ptr<T> {}
impl<T: ?Sized> !Send for Ptr<T> {}

impl<T: ?Sized> PartialEq for Ptr<T> {
    /// Equality for two `Ptr`s.
    ///
    /// Two `Ptr`s are equal if their inner meta data are equal.
    #[inline(always)]
    fn eq(&self, other: &Ptr<T>) -> bool {
        self.meta == other.meta
    }

    /// Inequality for two `Ptr`s.
    ///
    /// Two `Rc`s are unequal if their inner meta data are unequal.
    #[inline(always)]
    fn ne(&self, other: &Ptr<T>) -> bool {
        self.meta != other.meta
    }
}

impl<T: ?Sized> Eq for Ptr<T> {}

impl<T: ?Sized> PartialOrd for Ptr<T> {
    /// Partial comparison for two `Ptr`s.
    ///
    /// The two are compared by calling `partial_cmp()` on their meta data.
    #[inline(always)]
    fn partial_cmp(&self, other: &Ptr<T>) -> Option<Ordering> {
        (self.meta).partial_cmp(&other.meta)
    }

    /// Less-than comparison for two `Ptr`s.
    ///
    /// The two are compared by calling `<` on their meta data.
    #[inline(always)]
    fn lt(&self, other: &Ptr<T>) -> bool {
        self.meta < other.meta
    }

    /// 'Less than or equal to' comparison for two `Ptr`s.
    ///
    /// The two are compared by calling `<=` on their meta data.
    #[inline(always)]
    fn le(&self, other: &Ptr<T>) -> bool {
        self.meta <= other.meta
    }

    /// Greater-than comparison for two `Ptr`s.
    ///
    /// The two are compared by calling `>` on their meta data.
    #[inline(always)]
    fn gt(&self, other: &Ptr<T>) -> bool {
        self.meta > other.meta
    }

    /// 'Greater than or equal to' comparison for two `Ptr`s.
    ///
    /// The two are compared by calling `>=` on their meta data.
    #[inline(always)]
    fn ge(&self, other: &Ptr<T>) -> bool {
        self.meta >= other.meta
    }
}

impl<T: ?Sized> Ord for Ptr<T> {
    /// Comparison for two `Ptr`s.
    ///
    /// The two are compared by calling `cmp()` on their meta data.
    #[inline]
    fn cmp(&self, other: &Ptr<T>) -> Ordering {
        (self.meta).cmp(&other.meta)
    }
}

impl<T: ?Sized> Hash for Ptr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.meta).hash(state);
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Ptr<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { self.meta.as_ref() }.fmt(f)?;
        unsafe { self.value.as_ref() }.fmt(f)?;
        Ok(())
    }
}

impl<T: ?Sized> fmt::Debug for Ptr<T> {
    default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { self.meta.as_ref() }.fmt(f)?;
        f.write_str("value: { ")?;
        unsafe {
            f.write_str(std::intrinsics::type_name::<T>())?;
        }
        f.write_str(" }")?;
        Ok(())
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for Ptr<T> {
    default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { self.value.as_ref().fmt(f)? }
        Ok(())
    }
}

//////////////////////////////////////////
// WeakPtr - custom
//////////////////////////////////////////
pub struct WeakPtr<T>
where
    T: ?Sized,
{
    meta: NonNull<MetaData>,
    value: NonNull<T>,
}

impl<T: ?Sized> !Sync for WeakPtr<T> {}

impl<T: ?Sized> PartialEq for WeakPtr<T> {
    /// Equality for two `WeakPtr`s.
    ///
    /// Two `WeakPtr`s are equal if their inner meta data are equal.
    #[inline(always)]
    fn eq(&self, other: &WeakPtr<T>) -> bool {
        self.meta == other.meta
    }

    /// Inequality for two `WeakPtr`s.
    ///
    /// Two `Rc`s are unequal if their inner meta data are unequal.
    #[inline(always)]
    fn ne(&self, other: &WeakPtr<T>) -> bool {
        self.meta != other.meta
    }
}

impl<T: ?Sized> Eq for WeakPtr<T> {}

impl<T: ?Sized> PartialOrd for WeakPtr<T> {
    /// Partial comparison for two `WeakPtr`s.
    ///
    /// The two are compared by calling `partial_cmp()` on their meta data.
    #[inline(always)]
    fn partial_cmp(&self, other: &WeakPtr<T>) -> Option<Ordering> {
        (self.meta).partial_cmp(&other.meta)
    }

    /// Less-than comparison for two `WeakPtr`s.
    ///
    /// The two are compared by calling `<` on their meta data.
    #[inline(always)]
    fn lt(&self, other: &WeakPtr<T>) -> bool {
        self.meta < other.meta
    }

    /// 'Less than or equal to' comparison for two `WeakPtr`s.
    ///
    /// The two are compared by calling `<=` on their meta data.
    #[inline(always)]
    fn le(&self, other: &WeakPtr<T>) -> bool {
        self.meta <= other.meta
    }

    /// Greater-than comparison for two `WeakPtr`s.
    ///
    /// The two are compared by calling `>` on their meta data.
    #[inline(always)]
    fn gt(&self, other: &WeakPtr<T>) -> bool {
        self.meta > other.meta
    }

    /// 'Greater than or equal to' comparison for two `WeakPtr`s.
    ///
    /// The two are compared by calling `>=` on their meta data.
    #[inline(always)]
    fn ge(&self, other: &WeakPtr<T>) -> bool {
        self.meta >= other.meta
    }
}

impl<T: ?Sized> Ord for WeakPtr<T> {
    /// Comparison for two `WeakPtr`s.
    ///
    /// The two are compared by calling `cmp()` on their meta data.
    #[inline]
    fn cmp(&self, other: &WeakPtr<T>) -> Ordering {
        (self.meta).cmp(&other.meta)
    }
}

impl<T: ?Sized> Hash for WeakPtr<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.meta).hash(state);
    }
}

impl<T: ?Sized> fmt::Debug for WeakPtr<T> {
    default fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { self.meta.as_ref() }.fmt(f)?;
        f.write_str("value: { ")?;
        unsafe {
            f.write_str(std::intrinsics::type_name::<T>())?;
        }
        f.write_str(" }")?;
        Ok(())
    }
}

//////////////////////////////////////////
// Glue - custom
//////////////////////////////////////////
impl<T: ?Sized> RcBoxPtr for Ptr<T> {
    #[inline(always)]
    fn inner(&self) -> &MetaData {
        unsafe { self.meta.as_ref() }
    }
}

impl<T: ?Sized> RcBoxPtr for WeakPtr<T> {
    #[inline(always)]
    fn inner(&self) -> &MetaData {
        unsafe { self.meta.as_ref() }
    }
}

impl RcBoxPtr for MetaData {
    #[inline(always)]
    fn inner(&self) -> &MetaData {
        self
    }
}

//////////////////////////////////////////
// STD
//////////////////////////////////////////
//////////////////////////////////////////
// RefCell Errors
//////////////////////////////////////////
pub struct BorrowError {
    _private: (),
}

impl Debug for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BorrowError").finish()
    }
}

impl Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt("already mutably borrowed", f)
    }
}

pub struct BorrowMutError {
    _private: (),
}

impl Debug for BorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BorrowMutError").finish()
    }
}

impl Display for BorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt("already borrowed", f)
    }
}

//////////////////////////////////////////
// RefCell Refs
//////////////////////////////////////////
struct BorrowRef<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl<'b> BorrowRef<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRef<'b>> {
        let b = borrow.get();
        if is_writing(b) || b == isize::max_value() {
            // If there's currently a writing borrow, or if incrementing the
            // refcount would overflow into a writing borrow.
            None
        } else {
            borrow.set(b + 1);
            Some(BorrowRef { borrow })
        }
    }
}

impl<'b> Drop for BorrowRef<'b> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(is_reading(borrow));
        self.borrow.set(borrow - 1);
    }
}

impl<'b> Clone for BorrowRef<'b> {
    #[inline]
    fn clone(&self) -> BorrowRef<'b> {
        // Since this Ref exists, we know the borrow flag
        // is a reading borrow.
        let borrow = self.borrow.get();
        debug_assert!(is_reading(borrow));
        // Prevent the borrow counter from overflowing into
        // a writing borrow.
        assert!(borrow != isize::max_value());
        self.borrow.set(borrow + 1);
        BorrowRef {
            borrow: self.borrow,
        }
    }
}

/// Wraps a borrowed reference to a value in a `Ptr` box.
/// A wrapper type for an immutably borrowed value from a `Ptr<T>`.
pub struct Ref<'b, T: ?Sized + 'b> {
    value: &'b T,
    borrow: BorrowRef<'b>,
}

impl<'b, T: ?Sized> Deref for Ref<'b, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<'b, T: ?Sized> Ref<'b, T> {
    /// Copies a `Ref`.
    ///
    /// The `Ptr` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `Ref::clone(...)`.  A `Clone` implementation or a method would interfere
    /// with the widespread use of `r.borrow().clone()` to clone the contents of
    /// a `Ptr`.
    #[inline]
    pub fn clone(orig: &Ref<'b, T>) -> Ref<'b, T> {
        Ref {
            value: orig.value,
            borrow: orig.borrow.clone(),
        }
    }

    /// Make a new `Ref` for a component of the borrowed data.
    ///
    /// The `Ptr` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as `Ref::map(...)`.
    /// A method would interfere with methods of the same name on the contents
    /// of a `Ptr` used through `Deref`.
    #[inline]
    pub fn map<U: ?Sized, F>(orig: Ref<'b, T>, f: F) -> Ref<'b, U>
    where
        F: FnOnce(&T) -> &U,
    {
        Ref {
            value: f(orig.value),
            borrow: orig.borrow,
        }
    }
}

//#[unstable(feature = "coerce_unsized", issue = "27732")]
impl<'b, T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Ref<'b, U>> for Ref<'b, T> {}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for Ref<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<'b, T: ?Sized> RefMut<'b, T> {
    /// Make a new `RefMut` for a component of the borrowed data, e.g. an enum
    /// variant.
    ///
    /// The `Ptr` is already mutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RefMut::map(...)`.  A method would interfere with methods of the same
    /// name on the contents of a `Ptr` used through `Deref`.
    #[inline]
    pub fn map<U: ?Sized, F>(orig: RefMut<'b, T>, f: F) -> RefMut<'b, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        // FIXME(nll-rfc#40): fix borrow-check
        let RefMut { value, borrow } = orig;
        RefMut {
            value: f(value),
            borrow,
        }
    }
}

struct BorrowRefMut<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl<'b> Drop for BorrowRefMut<'b> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(is_writing(borrow));
        self.borrow.set(borrow + 1);
    }
}

impl<'b> BorrowRefMut<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRefMut<'b>> {
        // NOTE: Unlike BorrowRefMut::clone, new is called to create the initial
        // mutable reference, and so there must currently be no existing
        // references. Thus, while clone increments the mutable refcount, here
        // we explicitly only allow going from UNUSED to UNUSED - 1.
        match borrow.get() {
            UNUSED => {
                borrow.set(UNUSED - 1);
                Some(BorrowRefMut { borrow })
            }
            _ => None,
        }
    }
    // Clone a `BorrowRefMut`.
    //
    // This is only valid if each `BorrowRefMut` is used to track a mutable
    // reference to a distinct, nonoverlapping range of the original object.
    // This isn't in a Clone impl so that code doesn't call this implicitly.
    #[inline]
    fn clone(&self) -> BorrowRefMut<'b> {
        let borrow = self.borrow.get();
        debug_assert!(is_writing(borrow));
        // Prevent the borrow counter from underflowing.
        assert!(borrow != isize::min_value());
        self.borrow.set(borrow - 1);
        BorrowRefMut {
            borrow: self.borrow,
        }
    }
}

/// A wrapper type for a mutably borrowed value from a `Ptr<T>`.
pub struct RefMut<'b, T: ?Sized + 'b> {
    value: &'b mut T,
    borrow: BorrowRefMut<'b>,
}

impl<'b, T: ?Sized> Deref for RefMut<'b, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        self.value
    }
}

impl<'b, T: ?Sized> DerefMut for RefMut<'b, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}

//#[unstable(feature = "coerce_unsized", issue = "27732")]
impl<'b, T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<RefMut<'b, U>> for RefMut<'b, T> {}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for RefMut<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

//////////////////////////////////////////
// Ptr - std
//////////////////////////////////////////
// Positive values represent the number of `Ref` active. Negative values
// represent the number of `RefMut` active. Multiple `RefMut`s can only be
// active at a time if they refer to distinct, nonoverlapping components of a
// `Ptr` (e.g., different ranges of a slice).
//
// `Ref` and `RefMut` are both two words in size, and so there will likely never
// be enough `Ref`s or `RefMut`s in existence to overflow half of the `usize`
// range. Thus, a `BorrowFlag` will probably never overflow or underflow.
// However, this is not a guarantee, as a pathological program could repeatedly
// create and then mem::forget `Ref`s or `RefMut`s. Thus, all code must
// explicitly check for overflow and underflow in order to avoid unsafety, or at
// least behave correctly in the event that overflow or underflow happens (e.g.,
// see BorrowRef::new).
type BorrowFlag = isize;

const UNUSED: BorrowFlag = 0;

#[inline(always)]
fn is_writing(x: BorrowFlag) -> bool {
    x < UNUSED
}

#[inline(always)]
fn is_reading(x: BorrowFlag) -> bool {
    x > UNUSED
}

impl<T> Ptr<T>
where
    T: ?Sized,
{
    /// Creates a new [`WeakPtr`][weak] pointer to this value.
    pub fn downgrade(&self) -> WeakPtr<T> {
        self.inc_weak();
        // Make sure we do not create a dangling Weak
        debug_assert!(!is_dangling(self.value));
        WeakPtr {
            meta: self.meta,
            value: self.value,
        }
    }

    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    #[inline]
    pub fn borrow(&self) -> Ref<T> {
        self.try_borrow().expect("already mutably borrowed")
    }

    /// Immutably borrows the wrapped value, returning an error if the value is currently mutably
    /// borrowed.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple immutable borrows can be
    /// taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    #[inline]
    pub fn try_borrow(&self) -> Result<Ref<T>, BorrowError> {
        match BorrowRef::new(&unsafe { self.meta.as_ref() }.borrow) {
            Some(b) => Ok(Ref {
                value: unsafe { &*self.value.as_ptr() },
                borrow: b,
            }),
            None => Err(BorrowError { _private: () }),
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` or all `RefMut`s derived
    /// from it exit scope. The value cannot be borrowed while this borrow is
    /// active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed. For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    #[inline]
    pub fn borrow_mut(&self) -> RefMut<T> {
        self.try_borrow_mut().expect("already borrowed")
    }

    /// Mutably borrows the wrapped value, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts until the returned `RefMut` or all `RefMut`s derived
    /// from it exit scope. The value cannot be borrowed while this borrow is
    /// active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    #[inline]
    pub fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError> {
        match BorrowRefMut::new(&unsafe { self.meta.as_ref() }.borrow) {
            Some(b) => Ok(RefMut {
                value: unsafe { &mut *self.value.as_ptr() },
                borrow: b,
            }),
            None => Err(BorrowMutError { _private: () }),
        }
    }

    /// Gets the number of [`WeakPtr`][weak] pointers to this value.
    #[inline]
    pub fn weak_count(&self) -> usize {
        let weak = unsafe { self.meta.as_ref().weak.get() };
        weak - 1
    }

    /// Gets the number of strong (`Ptr`) pointers to this value.
    #[inline]
    pub fn strong_count(&self) -> usize {
        unsafe { self.meta.as_ref().strong.get() }
    }

    /// Returns true if there are no other `Ptr` or [`WeakPtr`][weak] pointers to
    /// this inner value.
    #[inline]
    fn is_unique(&self) -> bool {
        self.weak_count() == 0 && self.strong_count() == 1
    }
    #[inline]
    /// Returns true if the two `Ptr`s point to the same value (not
    /// just values that compare as equal).
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.value.as_ptr() == other.value.as_ptr()
    }
}

//#[unstable(feature = "coerce_unsized", issue = "27732")]
impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Ptr<U>> for Ptr<T> {}

impl<T> From<T> for Ptr<T>
where
    T: 'static + CastAble,
{
    fn from(t: T) -> Self {
        Ptr::new(t)
    }
}

impl<T> Default for Ptr<T>
where
    T: Default + 'static + CastAble,
{
    /// Creates a new `Ptr<T>`, with the `Default` value for `T`.
    #[inline]
    fn default() -> Ptr<T> {
        Ptr::new(Default::default())
    }
}

pub(crate) fn is_dangling<T: ?Sized>(ptr: NonNull<T>) -> bool {
    let address = ptr.as_ptr() as *mut () as usize;
    address == std::usize::MAX
}

//////////////////////////////////////////
// Weak Pointer - std
//////////////////////////////////////////
impl<T> WeakPtr<T>
where
    T: ?Sized,
{
    pub fn upgrade(&self) -> Option<Ptr<T>> {
        let inner = self.inner();
        if inner.strong() == 0 {
            None
        } else {
            inner.inc_strong();
            Some(Ptr {
                meta: self.meta,
                value: self.value,
            })
        }
    }
}

impl<T: ?Sized> Drop for WeakPtr<T> {
    /// Drops the `Weak` pointer.
    fn drop(&mut self) {
        let inner = self.inner();
        inner.dec_weak();
        // the weak count starts at 1, and will only go to zero if all
        // the strong pointers have disappeared.
        if inner.weak() == 0 {
            unsafe {
                Box::from_raw(self.meta.as_ptr());
            }
        }
    }
}

impl<T: ?Sized> Clone for WeakPtr<T> {
    /// Makes a clone of the `WeakPtr` pointer that points to the same value.
    #[inline]
    fn clone(&self) -> WeakPtr<T> {
        self.inner().inc_weak();
        WeakPtr {
            value: self.value,
            meta: self.meta,
        }
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<WeakPtr<U>> for WeakPtr<T> {}

//////////////////////////////////////////
// Glue - std
//////////////////////////////////////////
// NOTE: We checked_add here to deal with mem::forget safely. In particular
// if you mem::forget Rcs (or Weaks), the ref-count can overflow, and then
// you can free the allocation while outstanding Rcs (or Weaks) exist.
// We abort because this is such a degenerate scenario that we don't care about
// what happens -- no real program should ever experience this.
//
// This should have negligible overhead since you don't actually need to
// clone these much in Rust thanks to ownership and move-semantics.

#[doc(hidden)]
trait RcBoxPtr {
    fn inner(&self) -> &MetaData;

    #[inline]
    fn strong(&self) -> usize {
        self.inner().strong.get()
    }

    #[inline]
    fn inc_strong(&self) {
        self.inner().strong.set(
            self.strong()
                .checked_add(1)
                .unwrap_or_else(|| unsafe { abort() }),
        );
    }

    #[inline]
    fn dec_strong(&self) {
        self.inner().strong.set(self.strong() - 1);
    }

    #[inline]
    fn weak(&self) -> usize {
        self.inner().weak.get()
    }

    #[inline]
    fn inc_weak(&self) {
        self.inner().weak.set(
            self.weak()
                .checked_add(1)
                .unwrap_or_else(|| unsafe { abort() }),
        );
    }

    #[inline]
    fn dec_weak(&self) {
        self.inner().weak.set(self.weak() - 1);
    }
}

//////////////////////////////////////////
// Tests
//////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;

    impl<T> Ptr<T>
    where
        T: 'static,
    {
        fn new_t(value: T) -> Ptr<T> {
            let p = Ptr {
                meta: Box::into_raw_non_null(Box::new(MetaData {
                    cast_id: 0,
                    strong: Cell::new(1),
                    weak: Cell::new(1),
                    borrow: Cell::new(UNUSED),
                })),
                value: Box::into_raw_non_null(Box::new(value)), // NOTE this is bad for cache - would be nice if the box could be removed
            };
            p
        }
    }

    mod ptr {
        use super::*;

        #[test]
        fn strong_weak_strong() {
            let strong = Ptr::new_t(42);
            let weak = strong.clone().downgrade();
            let weak_strong = weak.upgrade();

            assert!(weak_strong.is_some());
            {
                let strong_borrow = strong.borrow();
                let weak_strong = weak_strong.unwrap();
                let weak_borrow = weak_strong.borrow();
                assert_eq!(*weak_borrow, 42);
                assert_eq!(*weak_borrow, *strong_borrow);
            }
        }

        #[test]
        fn double_borrow() {
            let ptr = Ptr::new_t(42);
            let b1 = ptr.borrow();
            let b2 = ptr.borrow();

            assert_eq!(*b1, 42);
            assert_eq!(*b1, *b2);
        }

        #[test]
        fn single_borrow_mut() {
            let ptr = Ptr::new_t(42);

            let b1 = ptr.borrow_mut();

            assert_eq!(*b1, 42);
        }

        #[test]
        #[should_panic]
        fn double_borrow_mut() {
            let ptr = Ptr::new_t(42);

            let b1 = ptr.borrow_mut();
            let b2 = ptr.borrow_mut(); // should panic

            assert_eq!(*b1, *b2);
        }
    }

    mod std {
        use super::*;

        use std::cell::RefCell;

        #[test]
        fn test_clone() {
            let x = Ptr::new_t(RefCell::new(5));
            let y = x.clone();
            *x.borrow().borrow_mut() = 20;
            assert_eq!(*y.borrow().borrow(), 20);
        }

        #[test]
        fn test_simple() {
            let x = Ptr::new_t(5);
            assert_eq!(*x.borrow(), 5);
        }

        #[test]
        fn test_simple_clone() {
            let x = Ptr::new_t(5);
            let y = x.clone();
            assert_eq!(*x.borrow(), 5);
            assert_eq!(*y.borrow(), 5);
        }

        #[test]
        fn test_destructor() {
            let x: Ptr<Box<_>> = Ptr::new_t(Box::new(5));
            assert_eq!(**x.borrow(), 5);
        }

        #[test]
        fn test_live() {
            let x = Ptr::new_t(5);
            let y = x.downgrade();
            assert!(y.upgrade().is_some());
        }

        #[test]
        fn test_dead() {
            let x = Ptr::new_t(5);
            let y = x.downgrade();
            drop(x);
            assert!(y.upgrade().is_none());
        }

        #[test]
        fn weak_self_cyclic() {
            struct Cycle {
                x: Ptr<Option<WeakPtr<Cycle>>>,
            }

            let a = Ptr::new_t(Cycle {
                x: Ptr::new_t(None),
            });
            let b = a.clone().downgrade();
            *a.borrow().x.borrow_mut() = Some(b);

            // hopefully we don't double-free (or leak)...
        }

        #[test]
        fn is_unique() {
            let x = Ptr::new_t(3);
            assert!(x.is_unique());
            let y = x.clone();
            assert!(!x.is_unique());
            drop(y);
            assert!(x.is_unique());
            let w = x.downgrade();
            assert!(!x.is_unique());
            drop(w);
            assert!(x.is_unique());
        }

        #[test]
        fn test_strong_count() {
            let a = Ptr::new_t(0);
            assert_eq!(a.strong_count(), 1);
            let w = a.downgrade();
            assert_eq!(a.strong_count(), 1);
            let b = w.upgrade().expect("upgrade of live rc failed");
            assert_eq!(b.strong_count(), 2);
            assert_eq!(a.strong_count(), 2);
            drop(w);
            drop(a);
            assert_eq!(b.strong_count(), 1);
            let c = b.clone();
            assert_eq!(b.strong_count(), 2);
            assert_eq!(c.strong_count(), 2);
        }

        #[test]
        fn test_weak_count() {
            let a = Ptr::new_t(0);
            assert_eq!(a.strong_count(), 1);
            assert_eq!(a.weak_count(), 0);
            let w = a.downgrade();
            assert_eq!(a.strong_count(), 1);
            assert_eq!(a.weak_count(), 1);
            drop(w);
            assert_eq!(a.strong_count(), 1);
            assert_eq!(a.weak_count(), 0);
            let c = a.clone();
            assert_eq!(a.strong_count(), 2);
            assert_eq!(a.weak_count(), 0);
            drop(c);
        }

        // NOTE not needed
        // #[test]
        // fn try_unwrap() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn into_from_raw() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn test_into_from_raw_unsized() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn get_mut() {
        //     ...
        // }

        // NOTE There is no copy on write
        // #[test]
        // fn test_cowrc_clone_make_unique() {
        //     ...
        // }

        // NOTE There is no copy on write
        // #[test]
        // fn test_cowrc_clone_unique2() {
        //     ...
        // }

        // NOTE There is no copy on write
        // #[test]
        // fn test_cowrc_clone_weak() {
        //     ...
        // }

        // TODO do we want this?
        // #[test]
        // fn test_show() {
        //     let foo = Ptr::new_t(75);
        //     assert_eq!(format!("{:?}", foo), "75");
        // }

        // NOTE not needed
        // #[test]
        // fn test_unsized() {
        //     ...
        // }

        // NOTE this dosn't work as form has the CastAble bound
        //  #[test]
        //  fn test_from_owned() {
        //      let foo = 123;
        //      let foo_rc = Ptr::from(foo);
        //      assert!(123 == *foo_rc.borrow());
        //  }

        // NOTE not needed
        //  #[test]
        //  fn test_new_weak() {
        //      ...
        //  }

        #[test]
        fn test_ptr_eq() {
            let five = Ptr::new_t(5);
            let same_five = five.clone();
            let other_five = Ptr::new_t(5);

            assert!(Ptr::ptr_eq(&five, &same_five));
            assert!(!Ptr::ptr_eq(&five, &other_five));
        }

        // NOTE not needed
        // #[test]
        // fn test_from_str() {
        //     ...
        // }

        // NOTE not needed
        //#[test]
        //fn test_copy_from_slice() {
        //    ...
        //}

        // NOTE not needed
        // #[test]
        // fn test_clone_from_slice() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // #[should_panic]
        // fn test_clone_from_slice_panic() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn test_from_box() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn test_from_box_str() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn test_from_box_slice() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn test_from_box_trait() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn test_from_box_trait_zero_sized() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn test_from_vec() {
        //     ...
        // }

        // NOTE not needed
        // #[test]
        // fn test_downcast() {
        //     ...
        // }
    }
}
