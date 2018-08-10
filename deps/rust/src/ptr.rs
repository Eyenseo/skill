use std::any::TypeId;
use std::boxed::Box;
use std::cell::Cell;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::marker::Unsize;
use std::ops::CoerceUnsized;
use std::ops::{Deref, DerefMut};
use std::ptr;
use std::ptr::NonNull;

/// An error returned by [`RefCell::try_borrow`](struct.RefCell.html#method.try_borrow).
pub struct BorrowError {
    private: (),
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

/// An error returned by [`RefCell::try_borrow_mut`](struct.RefCell.html#method.try_borrow_mut).
pub struct BorrowMutError {
    private: (),
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

struct BorrowRef<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl<'b> BorrowRef<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRef<'b>> {
        match borrow.get() {
            WRITING => None,
            b => {
                borrow.set(b + 1);
                Some(BorrowRef { borrow })
            }
        }
    }
}

impl<'b> Drop for BorrowRef<'b> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(borrow != WRITING && borrow != UNUSED);
        self.borrow.set(borrow - 1);
    }
}

impl<'b> Clone for BorrowRef<'b> {
    #[inline]
    fn clone(&self) -> BorrowRef<'b> {
        // Since this Ref exists, we know the borrow flag
        // is not set to WRITING.
        let borrow = self.borrow.get();
        debug_assert!(borrow != UNUSED);
        // Prevent the borrow counter from overflowing.
        assert!(borrow != WRITING);
        self.borrow.set(borrow + 1);
        BorrowRef {
            borrow: self.borrow,
        }
    }
}

/// Wraps a borrowed reference to a value in a `RefCell` box.
/// A wrapper type for an immutably borrowed value from a `RefCell<T>`.
///
/// See the [module-level documentation](index.html) for more.
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
    /// The `RefCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `Ref::clone(...)`.  A `Clone` implementation or a method would interfere
    /// with the widespread use of `r.borrow().clone()` to clone the contents of
    /// a `RefCell`.
    #[inline]
    pub fn clone(orig: &Ref<'b, T>) -> Ref<'b, T> {
        Ref {
            value: orig.value,
            borrow: orig.borrow.clone(),
        }
    }

    /// Make a new `Ref` for a component of the borrowed data.
    ///
    /// The `RefCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as `Ref::map(...)`.
    /// A method would interfere with methods of the same name on the contents
    /// of a `RefCell` used through `Deref`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::{RefCell, Ref};
    ///
    /// let c = RefCell::new((5, 'b'));
    /// let b1: Ref<(u32, char)> = c.borrow();
    /// let b2: Ref<u32> = Ref::map(b1, |t| &t.0);
    /// assert_eq!(*b2, 5)
    /// ```
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
    /// The `RefCell` is already mutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `RefMut::map(...)`.  A method would interfere with methods of the same
    /// name on the contents of a `RefCell` used through `Deref`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::cell::{RefCell, RefMut};
    ///
    /// let c = RefCell::new((5, 'b'));
    /// {
    ///     let b1: RefMut<(u32, char)> = c.borrow_mut();
    ///     let mut b2: RefMut<u32> = RefMut::map(b1, |t| &mut t.0);
    ///     assert_eq!(*b2, 5);
    ///     *b2 = 42;
    /// }
    /// assert_eq!(*c.borrow(), (42, 'b'));
    /// ```
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
        debug_assert!(borrow == WRITING);
        self.borrow.set(UNUSED);
    }
}

impl<'b> BorrowRefMut<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRefMut<'b>> {
        match borrow.get() {
            UNUSED => {
                borrow.set(WRITING);
                Some(BorrowRefMut { borrow })
            }
            _ => None,
        }
    }
}

/// A wrapper type for a mutably borrowed value from a `RefCell<T>`.
///
/// See the [module-level documentation](index.html) for more.
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

impl<'b, T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<RefMut<'b, U>> for RefMut<'b, T> {}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for RefMut<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.value.fmt(f)
    }
}

//////////////////////////////////////////
// Strong Pointer
//////////////////////////////////////////

// Values [1, MAX-1] represent the number of `Ref` active
// (will not outgrow its range since `usize` is the size of the address space)
type BorrowFlag = usize;
const UNUSED: BorrowFlag = 0;
const WRITING: BorrowFlag = !0;

#[derive(Debug)]
struct MetaData {
    type_id: TypeId, // TODO replace with generated ID?
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

impl<T: 'static> Ptr<T> {
    pub fn new(value: T) -> Ptr<T> {
        let p = Ptr {
            meta: Box::into_raw_non_null(Box::new(MetaData {
                type_id: TypeId::of::<T>(),
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
            let meta = self.meta.as_ref();

            meta.strong.set(meta.strong.get() - 1);
            if meta.strong.get() == 0 {
                Box::from_raw(self.value.as_ptr());

                // remove the implicit "strong weak" pointer now that we've
                // destroyed the contents.
                meta.weak.set(meta.weak.get() - 1);

                if meta.weak.get() == 0 {
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
    pub struct VTable(*const ());

    impl VTable {
        pub fn none() -> VTable {
            VTable(ptr::null())
        }
    }

    /// Represents a trait object's layout. You shouldn't need to use this as a
    /// consumer of the crate but it is required for macro expansion.
    #[doc(hidden)]
    #[repr(C)]
    #[derive(Copy, Clone, Debug)]
    pub struct TraitObject {
        pub data: *const (),
        pub vtable: VTable,
    }
}
pub use self::diggsey::*;

#[macro_export]
macro_rules! ptr_cast_able {
    // Choose the right vtable branch based on wanted TypeId
    (@gen_vtable $id:ident, $struct:ty, {$($trait:ty),*$(,)*}) => {
        if $id == ::std::any::TypeId::of::<$struct>() {
            Some($crate::common::ptr::VTable::none())
        } else $(if $id == ::std::any::TypeId::of::<$trait>() {
            unsafe {
                let x = ::std::ptr::null::<$struct>() as *const $trait;
                Some(::std::mem::transmute::<_, $crate::common::ptr::TraitObject>(x).vtable)
            }
        } else)* {
            None
        }
    };
    //----------------------------------------
    // Empty
    //----------------------------------------
    ($for:ty) => {
        impl $crate::common::ptr::CastAble<$for> for Ptr<$for> {
            fn vtable(&self, id: ::std::any::TypeId) -> Option<$crate::common::ptr::VTable> {
                None
            }
        }
    };
    //----------------------------------------
    // Structs
    //----------------------------------------
    // impl Ptr<Struct>
    // NOTE this has to be infront of the trait rule
    ($for:ty = $traits:tt) => {
        impl $crate::common::ptr::CastAble<$for> for Ptr<$for> {
            fn vtable(&self, id: ::std::any::TypeId) -> Option<$crate::common::ptr::VTable> {
                ptr_cast_able!(@gen_vtable id, $for, $traits)
            }
        }
    };
    //----------------------------------------
    // Traits
    //----------------------------------------
    // Choose the right struct(s) based on TypeId
    (@gen_for_trait $self:ident, $id:ident, ($struct:ty : $traits:tt, $($in:tt)*) -> ($($out:tt)*)) => {
        ptr_cast_able!(@gen_for_trait $self, $id, ($($in)*) -> (
            $($out)*
            if $self.ptr_type_id() == ::std::any::TypeId::of::<$struct>() {
                ptr_cast_able!(@gen_vtable $id, $struct, $traits)
            } else )
        );
    };
    // 'Write' the accumulated output
    (@gen_for_trait $self:ident, $id:ident, ($(,)*) -> ($($out:tt)*)) => {$($out)* {None}};
    // impl Ptr<Trait>
    ($for:ty = $($in:tt)*) => {
        impl $crate::common::ptr::CastAble<$for> for Ptr<$for> {
            fn vtable(&self, id: ::std::any::TypeId) -> Option<$crate::common::ptr::VTable> {
                ptr_cast_able!(@gen_for_trait self, id, ($($in)*,) -> ())
            }
        }
    };
}

pub trait CastAble<T>
where
    T: ?Sized,
{
    fn vtable(&self, id: TypeId) -> Option<VTable>;
}

impl<T> Ptr<T>
where
    T: ?Sized,
{
    pub fn downgrade(&self) -> WeakPtr<T> {
        unsafe {
            let meta = self.meta.as_ref();
            meta.weak.set(meta.weak.get() + 1);
        }
        WeakPtr {
            meta: self.meta,
            value: self.value,
        }
    }

    pub fn nucast<U>(&self) -> Option<Ptr<U>>
    where
        U: ?Sized + 'static,
        Self: CastAble<T>,
    {
        // Adapted from https://github.com/Diggsey/query_interface
        if let Some(vtable) = self.vtable(::std::any::TypeId::of::<U>()) {
            unsafe {
                let data = self.value.as_ptr();
                let mut t = TraitObject {
                    data: data as *const (),
                    vtable,
                };
                let value = NonNull::new_unchecked(
                    // NOTE mut-ref-ptr-to T to mut-ret-ptr-to U
                    *::std::mem::transmute::<_, &mut *mut U>(&mut t),
                );
                let meta = self.meta.as_ref();
                meta.strong.set(meta.strong.get() + 1);
                Some(Ptr {
                    meta: self.meta,
                    value,
                })
            }
        } else {
            None
        }
    }

    pub fn ptr_type_id(&self) -> TypeId {
        unsafe { self.meta.as_ref().type_id }
    }

    pub fn borrow(&self) -> Ref<T> {
        match BorrowRef::new(unsafe { &self.meta.as_ref().borrow }) {
            Some(b) => Ref {
                value: unsafe { &*self.value.as_ref() },
                borrow: b,
            },
            None => panic!("already mutably borrowed"),
        }
    }
    pub fn try_borrow(&self) -> Result<Ref<T>, BorrowError> {
        match BorrowRef::new(unsafe { &self.meta.as_ref().borrow }) {
            Some(b) => Ok(Ref {
                value: unsafe { &*self.value.as_ptr() },
                borrow: b,
            }),
            None => Err(BorrowError { private: () }),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        match BorrowRefMut::new(unsafe { &self.meta.as_ref().borrow }) {
            Some(b) => RefMut {
                value: unsafe { &mut *self.value.as_ptr() },
                borrow: b,
            },
            None => panic!("already mutably borrowed"),
        }
    }

    pub fn try_borrow_mut(&self) -> Result<RefMut<T>, BorrowMutError> {
        match BorrowRefMut::new(unsafe { &self.meta.as_ref().borrow }) {
            Some(b) => Ok(RefMut {
                value: unsafe { &mut *self.value.as_ptr() },
                borrow: b,
            }),
            None => Err(BorrowMutError { private: () }),
        }
    }

    /// Gets the number of [`Weak`][weak] pointers to this value.
    ///
    /// [weak]: struct.Weak.html
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    /// let weak_five = Rc::downgrade(&five);
    ///
    /// assert_eq!(1, Rc::weak_count(&five));
    /// ```
    #[inline]
    pub fn weak_count(&self) -> usize {
        let weak = unsafe { self.meta.as_ref().weak.get() };
        weak - 1
    }

    /// Gets the number of strong (`Rc`) pointers to this value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    /// let also_five = Rc::clone(&five);
    ///
    /// assert_eq!(2, Rc::strong_count(&five));
    /// ```
    #[inline]
    pub fn strong_count(&self) -> usize {
        unsafe { self.meta.as_ref().strong.get() }
    }

    /// Returns true if there are no other `Rc` or [`Weak`][weak] pointers to
    /// this inner value.
    ///
    /// [weak]: struct.Weak.html
    #[inline]
    fn is_unique(&self) -> bool {
        self.weak_count() == 0 && self.strong_count() == 1
    }

    #[inline]
    /// Returns true if the two `Rc`s point to the same value (not
    /// just values that compare as equal).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    /// let same_five = Rc::clone(&five);
    /// let other_five = Rc::new(5);
    ///
    /// assert!(Rc::ptr_eq(&five, &same_five));
    /// assert!(!Rc::ptr_eq(&five, &other_five));
    /// ```
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.value.as_ptr() == other.value.as_ptr()
    }
}

impl<T: ?Sized> Clone for Ptr<T> {
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

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Ptr<U>> for Ptr<T> {}

impl<T: 'static> From<T> for Ptr<T> {
    fn from(t: T) -> Self {
        Ptr::new(t)
    }
}

impl<T: Default + 'static> Default for Ptr<T> {
    /// Creates a new `Rc<T>`, with the `Default` value for `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let x: Rc<i32> = Default::default();
    /// assert_eq!(*x, 0);
    /// ```
    #[inline]
    fn default() -> Ptr<T> {
        Ptr::new(Default::default())
    }
}

impl<T: ?Sized> PartialEq for Ptr<T> {
    /// Equality for two `Rc`s.
    ///
    /// Two `Rc`s are equal if their inner values are equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five == Rc::new(5));
    /// ```
    #[inline(always)]
    fn eq(&self, other: &Ptr<T>) -> bool {
        self.meta == other.meta
    }

    /// Inequality for two `Rc`s.
    ///
    /// Two `Rc`s are unequal if their inner values are unequal.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five != Rc::new(6));
    /// ```
    #[inline(always)]
    fn ne(&self, other: &Ptr<T>) -> bool {
        self.meta != other.meta
    }
}

impl<T: ?Sized> Eq for Ptr<T> {}

impl<T: ?Sized> PartialOrd for Ptr<T> {
    /// Partial comparison for two `Rc`s.
    ///
    /// The two are compared by calling `partial_cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    /// use std::cmp::Ordering;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert_eq!(Some(Ordering::Less), five.partial_cmp(&Rc::new(6)));
    /// ```
    #[inline(always)]
    fn partial_cmp(&self, other: &Ptr<T>) -> Option<Ordering> {
        (self.meta).partial_cmp(&other.meta)
    }

    /// Less-than comparison for two `Rc`s.
    ///
    /// The two are compared by calling `<` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five < Rc::new(6));
    /// ```
    #[inline(always)]
    fn lt(&self, other: &Ptr<T>) -> bool {
        self.meta < other.meta
    }

    /// 'Less than or equal to' comparison for two `Rc`s.
    ///
    /// The two are compared by calling `<=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five <= Rc::new(5));
    /// ```
    #[inline(always)]
    fn le(&self, other: &Ptr<T>) -> bool {
        self.meta <= other.meta
    }

    /// Greater-than comparison for two `Rc`s.
    ///
    /// The two are compared by calling `>` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five > Rc::new(4));
    /// ```
    #[inline(always)]
    fn gt(&self, other: &Ptr<T>) -> bool {
        self.meta > other.meta
    }

    /// 'Greater than or equal to' comparison for two `Rc`s.
    ///
    /// The two are compared by calling `>=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert!(five >= Rc::new(5));
    /// ```
    #[inline(always)]
    fn ge(&self, other: &Ptr<T>) -> bool {
        self.meta >= other.meta
    }
}

impl<T: ?Sized> Ord for Ptr<T> {
    /// Comparison for two `Rc`s.
    ///
    /// The two are compared by calling `cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::rc::Rc;
    /// use std::cmp::Ordering;
    ///
    /// let five = Rc::new(5);
    ///
    /// assert_eq!(Ordering::Less, five.cmp(&Rc::new(6)));
    /// ```
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
// Weak Pointer
//////////////////////////////////////////
pub struct WeakPtr<T>
where
    T: ?Sized,
{
    meta: NonNull<MetaData>,
    value: NonNull<T>,
}

impl<T> WeakPtr<T>
where
    T: ?Sized,
{
    pub fn upgrade(&self) -> Option<Ptr<T>> {
        unsafe {
            let meta = self.meta.as_ref();

            if meta.strong.get() > 0 {
                meta.strong.set(meta.strong.get() + 1);
                let p = Ptr {
                    meta: self.meta,
                    value: self.value,
                };
                return Some(p);
            }
            None
        }
    }
}

impl<T: ?Sized> Drop for WeakPtr<T> {
    fn drop(&mut self) {
        unsafe {
            let meta = self.meta.as_ref();

            meta.weak.set(meta.weak.get() - 1);

            if meta.weak.get() == 0 {
                Box::from_raw(self.meta.as_ptr());
            }
        }
    }
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<WeakPtr<U>> for WeakPtr<T> {}

#[cfg(test)]
mod tests {

    use super::*;

    mod ptr {
        use super::*;

        #[test]
        fn strong_weak_strong() {
            let strong = Ptr::new(42);
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
            let ptr = Ptr::new(42);
            let b1 = ptr.borrow();
            let b2 = ptr.borrow();

            assert_eq!(*b1, 42);
            assert_eq!(*b1, *b2);
        }

        #[test]
        fn single_borrow_mut() {
            let ptr = Ptr::new(42);

            let b1 = ptr.borrow_mut();

            assert_eq!(*b1, 42);
        }

        #[test]
        #[should_panic]
        fn double_borrow_mut() {
            let ptr = Ptr::new(42);

            let b1 = ptr.borrow_mut();
            let b2 = ptr.borrow_mut(); // should panic

            assert_eq!(*b1, *b2);
        }
    }

    mod casts {
        use super::*;

        #[derive(Debug, Clone)]
        struct Foo;

        #[derive(Debug, Clone)]
        struct Bar;

        trait FooT: Debug {
            fn test(&self) -> String {
                "FooT::Default".to_string()
            }
        }
        trait BarT: Debug {
            fn test(&self) -> String {
                "BarT::Default".to_string()
            }
        }

        impl FooT for Foo {
            fn test(&self) -> String {
                "Foo::FooT".to_string()
            }
        }

        impl FooT for Bar {
            fn test(&self) -> String {
                "Bar::FooT".to_string()
            }
        }
        impl BarT for Bar {
            fn test(&self) -> String {
                "Bar::BarT".to_string()
            }
        }

        ptr_cast_able!(Foo = {FooT,});
        ptr_cast_able!(Bar = {FooT, BarT,});
        ptr_cast_able!(FooT = Foo: {FooT}, Bar: {FooT, BarT},);
        ptr_cast_able!(BarT = Bar: {FooT, BarT});

        #[test]
        fn nucast() {
            let bar = Ptr::new(Bar {});
            let foo_t: Option<Ptr<FooT>> = bar.nucast();
            assert!(foo_t.is_some());
            let foo_t = foo_t.unwrap();
            assert_eq!(foo_t.borrow().test(), "Bar::FooT");
            let bar_t = foo_t.nucast::<BarT>();
            assert!(bar_t.is_some());
            let bar_t = bar_t.unwrap();
            assert_eq!(bar_t.borrow().test(), "Bar::BarT");
            let bar_t_bar: Option<Ptr<Bar>> = bar_t.nucast();
            assert!(bar_t_bar.is_some());
            let foo_t_foo: Option<Ptr<Foo>> = foo_t.nucast();
            assert!(foo_t_foo.is_none());

            let foo = Ptr::new(Foo {});
            let foo_t = foo.nucast::<FooT>();
            assert!(foo_t.is_some());
            let foo_t = foo_t.unwrap();
            assert!(foo_t.borrow().test() == "Foo::FooT");
            let bar_t = foo_t.nucast::<BarT>();
            assert!(bar_t.is_none());
            let foo_t_foo = foo_t.nucast::<Foo>();
            assert!(foo_t_foo.is_some());
        }
    }

    mod std {
        use super::*;

        use std::cell::RefCell;

        #[test]
        fn test_clone() {
            let x = Ptr::new(5);
            let y = x.clone();
            *x.borrow_mut() = 20;
            assert_eq!(*y.borrow(), 20);
        }

        #[test]
        fn test_simple() {
            let x = Ptr::new(5);
            assert_eq!(*x.borrow(), 5);
        }

        #[test]
        fn test_simple_clone() {
            let x = Ptr::new(5);
            let y = x.clone();
            assert_eq!(*x.borrow(), 5);
            assert_eq!(*y.borrow(), 5);
        }

        #[test]
        fn test_destructor() {
            let x: Ptr<Box<_>> = Ptr::new(Box::new(5));
            assert_eq!(**x.borrow(), 5);
        }

        #[test]
        fn test_live() {
            let x = Ptr::new(5);
            let y = x.clone().downgrade();
            assert!(y.upgrade().is_some());
        }

        #[test]
        fn test_dead() {
            let x = Ptr::new(5);
            let y = x.clone().downgrade();
            drop(x);
            assert!(y.upgrade().is_none());
        }

        #[test]
        fn weak_self_cyclic() {
            struct Cycle {
                x: RefCell<Option<WeakPtr<Cycle>>>,
            }

            let a = Ptr::new(Cycle {
                x: RefCell::new(None),
            });
            let b = a.clone().downgrade();

            let ab = a.borrow();
            *ab.x.borrow_mut() = Some(b);

            // hopefully we don't double-free (or leak)...
        }

        #[test]
        fn is_unique() {
            let x = Ptr::new(3);
            assert!(Ptr::is_unique(&x));
            let y = x.clone();
            assert!(!Ptr::is_unique(&x));
            drop(y);
            assert!(Ptr::is_unique(&x));
            let w = Ptr::downgrade(&x);
            assert!(!Ptr::is_unique(&x));
            drop(w);
            assert!(Ptr::is_unique(&x));
        }

        #[test]
        fn test_strong_count() {
            let a = Ptr::new(0);
            assert!(Ptr::strong_count(&a) == 1);
            let w = Ptr::downgrade(&a);
            assert!(Ptr::strong_count(&a) == 1);
            let b = w.upgrade().expect("upgrade of live rc failed");
            assert!(Ptr::strong_count(&b) == 2);
            assert!(Ptr::strong_count(&a) == 2);
            drop(w);
            drop(a);
            assert!(Ptr::strong_count(&b) == 1);
            let c = b.clone();
            assert!(Ptr::strong_count(&b) == 2);
            assert!(Ptr::strong_count(&c) == 2);
        }

        #[test]
        fn test_weak_count() {
            let a = Ptr::new(0);
            assert!(Ptr::strong_count(&a) == 1);
            assert!(Ptr::weak_count(&a) == 0);
            let w = Ptr::downgrade(&a);
            assert!(Ptr::strong_count(&a) == 1);
            assert!(Ptr::weak_count(&a) == 1);
            drop(w);
            assert!(Ptr::strong_count(&a) == 1);
            assert!(Ptr::weak_count(&a) == 0);
            let c = a.clone();
            assert!(Ptr::strong_count(&a) == 2);
            assert!(Ptr::weak_count(&a) == 0);
            drop(c);
        }

        //  NOTE not needed
        //  #[test]
        //  fn try_unwrap() {
        //      ...
        //  }

        //  NOTE not needed
        //  #[test]
        //  fn into_from_raw() {
        //      ...
        //  }

        //  NOTE not needed
        //  #[test]
        //  fn test_into_from_raw_unsized() {
        //      ...
        //  }

        //  NOTE not needed
        //  #[test]
        //  fn get_mut() {
        //      ...
        //  }

        //  NOTE There is no copy on write
        //  #[test]
        //  fn test_cowrc_clone_make_unique() {
        //      ...
        //  }

        //  NOTE There is no copy on write
        //  #[test]
        //  fn test_cowrc_clone_unique2() {
        //      ...
        //  }

        //  NOTE There is no copy on write
        //  #[test]
        //  fn test_cowrc_clone_weak() {
        //      ...
        //  }

        //  TODO do we want this?
        //  #[test]
        //  fn test_show() {
        //      let foo = Ptr::new(75);
        //      assert_eq!(format!("{:?}", foo), "75");
        //  }

        //  TODO we want this
        //  #[test]
        //  fn test_unsized() {
        //      let foo: Ptr<[i32]> = Ptr::new([1, 2, 3]);
        //      assert_eq!(foo, foo.clone());
        //  }

        #[test]
        fn test_from_owned() {
            let foo = 123;
            let foo_rc = Ptr::from(foo);
            assert!(123 == *foo_rc.borrow());
        }

        //  TODO do we need this? (for inits?)
        //  #[test]
        //  fn test_new_weak() {
        //      let foo: WeakPtr<usize> = WeakPtr::new();
        //      assert!(foo.upgrade().is_none());
        //  }

        #[test]
        fn test_ptr_eq() {
            let five = Ptr::new(5);
            let same_five = five.clone();
            let other_five = Ptr::new(5);

            assert!(Ptr::ptr_eq(&five, &same_five));
            assert!(!Ptr::ptr_eq(&five, &other_five));
        }

        //  TODO maybe want this
        //  #[test]
        //  fn test_from_str() {
        //      let r: Ptr<str> = Ptr::from("foo");

        //      assert_eq!(&r[..], "foo");
        //  }

        //  TODO maybe want this
        //  #[test]
        //  fn test_copy_from_slice() {
        //      let s: &[u32] = &[1, 2, 3];
        //      let r: Ptr<[u32]> = Ptr::from(s);

        //      assert_eq!(&r[..], [1, 2, 3]);
        //  }

        //  TODO maybe want this
        //  #[test]
        //  fn test_clone_from_slice() {
        //      #[derive(Clone, Debug, Eq, PartialEq)]
        //      struct X(u32);

        //      let s: &[X] = &[X(1), X(2), X(3)];
        //      let r: Ptr<[X]> = Ptr::from(s);

        //      assert_eq!(&r[..], s);
        //  }

        //  TODO maybe want this
        //  #[test]
        //  #[should_panic]
        //  fn test_clone_from_slice_panic() {
        //      use std::string::{String, ToString};

        //      struct Fail(u32, String);

        //      impl Clone for Fail {
        //          fn clone(&self) -> Fail {
        //              if self.0 == 2 {
        //                  panic!();
        //              }
        //              Fail(self.0, self.1.clone())
        //          }
        //      }

        //      let s: &[Fail] = &[
        //          Fail(0, "foo".to_string()),
        //          Fail(1, "bar".to_string()),
        //          Fail(2, "baz".to_string()),
        //      ];

        //      // Should panic, but not cause memory corruption
        //      let r: Ptr<[Fail]> = Ptr::from(s);
        //  }

        //  TODO maybe want this
        //  #[test]
        //  fn test_from_box() {
        //      let b: Box<u32> = Box::new(123);
        //      let r: Ptr<u32> = Ptr::from(b);

        //      assert_eq!(*r, 123);
        //  }

        //  TODO maybe want this
        //  #[test]
        //  fn test_from_box_str() {
        //      use std::string::String;

        //      let s = String::from("foo").into_boxed_str();
        //      let r: Ptr<str> = Ptr::from(s);

        //      assert_eq!(&r[..], "foo");
        //  }

        //  TODO maybe want this
        //  #[test]
        //  fn test_from_box_slice() {
        //      let s = vec![1, 2, 3].into_boxed_slice();
        //      let r: Ptr<[u32]> = Ptr::from(s);

        //      assert_eq!(&r[..], [1, 2, 3]);
        //  }

        //  TODO maybe want this
        //  #[test]
        //  fn test_from_box_trait() {
        //      use std::fmt::Display;
        //      use std::string::ToString;

        //      let b: Box<Display> = Box::new(123);
        //      let r: Ptr<Display> = Ptr::from(b);

        //      assert_eq!(r.to_string(), "123");
        //  }

        //  TODO maybe want this
        //  #[test]
        //  fn test_from_box_trait_zero_sized() {
        //      use std::fmt::Debug;

        //      let b: Box<Debug> = Box::new(());
        //      let r: Ptr<Debug> = Ptr::from(b);

        //      assert_eq!(format!("{:?}", r), "()");
        //  }

        //  TODO maybe want this
        //  #[test]
        //  fn test_from_vec() {
        //      let v = vec![1, 2, 3];
        //      let r: Ptr<[u32]> = Ptr::from(v);

        //      assert_eq!(&r[..], [1, 2, 3]);
        //  }

        //  NOTE we're generating our own cast functions
        //  #[test]
        //  fn test_downcast() {
        //      ...
        //  }
    }
}
