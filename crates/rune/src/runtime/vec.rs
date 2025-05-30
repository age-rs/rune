use core::cmp;
use core::cmp::Ordering;
use core::fmt;
use core::ops;
use core::slice;
use core::slice::SliceIndex;

use crate as rune;
use crate::alloc;
use crate::alloc::fmt::TryWrite;
use crate::alloc::prelude::*;
use crate::runtime::slice::Iter;
use crate::{vm_try, Any, TypeHash};

use super::{
    EnvProtocolCaller, Formatter, FromValue, Hasher, ProtocolCaller, Range, RangeFrom, RangeFull,
    RangeInclusive, RangeTo, RangeToInclusive, RawAnyGuard, Ref, RuntimeError, ToValue,
    UnsafeToRef, Value, VmErrorKind, VmResult,
};

/// Struct representing a dynamic vector.
///
/// # Examples
///
/// ```
/// let mut vec = rune::runtime::Vec::new();
/// assert!(vec.is_empty());
///
/// vec.push_value(42).into_result()?;
/// vec.push_value(true).into_result()?;
/// assert_eq!(2, vec.len());
///
/// assert_eq!(Some(42), vec.get_value(0).into_result()?);
/// assert_eq!(Some(true), vec.get_value(1).into_result()?);
/// assert_eq!(None::<bool>, vec.get_value(2).into_result()?);
/// # Ok::<_, rune::support::Error>(())
/// ```
#[derive(Default, Any)]
#[repr(transparent)]
#[rune(item = ::std::vec)]
pub struct Vec {
    inner: alloc::Vec<Value>,
}

impl Vec {
    /// Constructs a new, empty dynamic `Vec`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    ///
    /// ```
    /// use rune::runtime::Vec;
    ///
    /// let mut vec = Vec::new();
    /// ```
    pub const fn new() -> Self {
        Self {
            inner: alloc::Vec::new(),
        }
    }

    /// Sort the vector with the given comparison function.
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&Value, &Value) -> cmp::Ordering,
    {
        self.inner.sort_by(compare)
    }

    /// Construct a new dynamic vector guaranteed to have at least the given
    /// capacity.
    pub fn with_capacity(cap: usize) -> alloc::Result<Self> {
        Ok(Self {
            inner: alloc::Vec::try_with_capacity(cap)?,
        })
    }

    /// Convert into inner rune alloc vector.
    pub fn into_inner(self) -> alloc::Vec<Value> {
        self.inner
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use rune::runtime::{Value, Vec};
    ///
    /// let mut v = Vec::new();
    /// assert!(v.is_empty());
    ///
    /// v.push(rune::to_value(1u32)?);
    /// assert!(!v.is_empty());
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of elements in the dynamic vector, also referred to
    /// as its 'length'.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the number of elements in the dynamic vector, also referred to
    /// as its 'length'.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Set by index
    pub fn set(&mut self, index: usize, value: Value) -> VmResult<()> {
        let Some(v) = self.inner.get_mut(index) else {
            return VmResult::err(VmErrorKind::OutOfRange {
                index: index.into(),
                length: self.len().into(),
            });
        };

        *v = value;
        VmResult::Ok(())
    }

    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the
    /// difference, with each additional slot filled with `value`. If `new_len`
    /// is less than `len`, the `Vec` is simply truncated.
    pub fn resize(&mut self, new_len: usize, value: Value) -> VmResult<()> {
        if value.is_inline() {
            vm_try!(self.inner.try_resize(new_len, value));
        } else {
            let len = self.inner.len();

            if new_len > len {
                for _ in 0..new_len - len {
                    let value = vm_try!(value.clone_with(&mut EnvProtocolCaller));
                    vm_try!(self.inner.try_push(value));
                }
            } else {
                self.inner.truncate(new_len);
            }
        }

        VmResult::Ok(())
    }

    /// Appends an element to the back of a dynamic vector.
    pub fn push(&mut self, value: Value) -> alloc::Result<()> {
        self.inner.try_push(value)
    }

    /// Appends an element to the back of a dynamic vector, converting it as
    /// necessary through the [`ToValue`] trait.
    pub fn push_value<T>(&mut self, value: T) -> VmResult<()>
    where
        T: ToValue,
    {
        vm_try!(self.inner.try_push(vm_try!(value.to_value())));
        VmResult::Ok(())
    }

    /// Get the value at the given index.
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[Value]>,
    {
        self.inner.get(index)
    }

    /// Get the given value at the given index.
    pub fn get_value<T>(&self, index: usize) -> VmResult<Option<T>>
    where
        T: FromValue,
    {
        let value = match self.inner.get(index) {
            Some(value) => value.clone(),
            None => return VmResult::Ok(None),
        };

        VmResult::Ok(Some(vm_try!(T::from_value(value))))
    }

    /// Get the mutable value at the given index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Value> {
        self.inner.get_mut(index)
    }

    /// Removes the last element from a dynamic vector and returns it, or
    /// [`None`] if it is empty.
    pub fn pop(&mut self) -> Option<Value> {
        self.inner.pop()
    }

    /// Removes the element at the specified index from a dynamic vector.
    pub fn remove(&mut self, index: usize) -> Value {
        self.inner.remove(index)
    }

    /// Clears the vector, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity of the
    /// vector.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Inserts an element at position index within the vector, shifting all
    /// elements after it to the right.
    pub fn insert(&mut self, index: usize, value: Value) -> VmResult<()> {
        vm_try!(self.inner.try_insert(index, value));
        VmResult::Ok(())
    }

    /// Extend this vector with something that implements the into_iter
    /// protocol.
    pub fn extend(&mut self, value: Value) -> VmResult<()> {
        let mut it = vm_try!(value.into_iter());

        while let Some(value) = vm_try!(it.next()) {
            vm_try!(self.push(value));
        }

        VmResult::Ok(())
    }

    /// Iterate over the vector.
    ///
    /// # Examples
    ///
    /// ```rune
    /// let vec = [1, 2, 3, 4];
    /// let it = vec.iter();
    ///
    /// assert_eq!(it.next(), Some(1));
    /// assert_eq!(it.next_back(), Some(4));
    /// ```
    #[rune::function(keep, path = Self::iter)]
    pub fn rune_iter(this: Ref<Self>) -> Iter {
        Iter::new(Ref::map(this, |vec| &**vec))
    }

    /// Access the inner values as a slice.
    pub(crate) fn as_slice(&self) -> &[Value] {
        &self.inner
    }

    pub(crate) fn debug_fmt_with(
        this: &[Value],
        f: &mut Formatter,
        caller: &mut dyn ProtocolCaller,
    ) -> VmResult<()> {
        let mut it = this.iter().peekable();
        vm_try!(write!(f, "["));

        while let Some(value) = it.next() {
            vm_try!(value.debug_fmt_with(f, caller));

            if it.peek().is_some() {
                vm_try!(write!(f, ", "));
            }
        }

        vm_try!(write!(f, "]"));
        VmResult::Ok(())
    }

    pub(crate) fn partial_eq_with(
        a: &[Value],
        b: Value,
        caller: &mut dyn ProtocolCaller,
    ) -> VmResult<bool> {
        let mut b = vm_try!(b.into_iter_with(caller));

        for a in a {
            let Some(b) = vm_try!(b.next()) else {
                return VmResult::Ok(false);
            };

            if !vm_try!(Value::partial_eq_with(a, &b, caller)) {
                return VmResult::Ok(false);
            }
        }

        if vm_try!(b.next()).is_some() {
            return VmResult::Ok(false);
        }

        VmResult::Ok(true)
    }

    pub(crate) fn eq_with(
        a: &[Value],
        b: &[Value],
        eq: fn(&Value, &Value, &mut dyn ProtocolCaller) -> VmResult<bool>,
        caller: &mut dyn ProtocolCaller,
    ) -> VmResult<bool> {
        if a.len() != b.len() {
            return VmResult::Ok(false);
        }

        for (a, b) in a.iter().zip(b.iter()) {
            if !vm_try!(eq(a, b, caller)) {
                return VmResult::Ok(false);
            }
        }

        VmResult::Ok(true)
    }

    pub(crate) fn partial_cmp_with(
        a: &[Value],
        b: &[Value],
        caller: &mut dyn ProtocolCaller,
    ) -> VmResult<Option<Ordering>> {
        let mut b = b.iter();

        for a in a.iter() {
            let Some(b) = b.next() else {
                return VmResult::Ok(Some(Ordering::Greater));
            };

            match vm_try!(Value::partial_cmp_with(a, b, caller)) {
                Some(Ordering::Equal) => continue,
                other => return VmResult::Ok(other),
            }
        }

        if b.next().is_some() {
            return VmResult::Ok(Some(Ordering::Less));
        }

        VmResult::Ok(Some(Ordering::Equal))
    }

    pub(crate) fn cmp_with(
        a: &[Value],
        b: &[Value],
        caller: &mut dyn ProtocolCaller,
    ) -> VmResult<Ordering> {
        let mut b = b.iter();

        for a in a.iter() {
            let Some(b) = b.next() else {
                return VmResult::Ok(Ordering::Greater);
            };

            match vm_try!(Value::cmp_with(a, b, caller)) {
                Ordering::Equal => continue,
                other => return VmResult::Ok(other),
            }
        }

        if b.next().is_some() {
            return VmResult::Ok(Ordering::Less);
        }

        VmResult::Ok(Ordering::Equal)
    }

    /// This is a common get implementation that can be used across linear
    /// types, such as vectors and tuples.
    pub(crate) fn index_get(this: &[Value], index: Value) -> VmResult<Option<Value>> {
        let slice: Option<&[Value]> = 'out: {
            if let Some(value) = index.as_any() {
                match value.type_hash() {
                    RangeFrom::HASH => {
                        let range = vm_try!(value.borrow_ref::<RangeFrom>());
                        let start = vm_try!(range.start.as_usize());
                        break 'out this.get(start..);
                    }
                    RangeFull::HASH => {
                        _ = vm_try!(value.borrow_ref::<RangeFull>());
                        break 'out this.get(..);
                    }
                    RangeInclusive::HASH => {
                        let range = vm_try!(value.borrow_ref::<RangeInclusive>());
                        let start = vm_try!(range.start.as_usize());
                        let end = vm_try!(range.end.as_usize());
                        break 'out this.get(start..=end);
                    }
                    RangeToInclusive::HASH => {
                        let range = vm_try!(value.borrow_ref::<RangeToInclusive>());
                        let end = vm_try!(range.end.as_usize());
                        break 'out this.get(..=end);
                    }
                    RangeTo::HASH => {
                        let range = vm_try!(value.borrow_ref::<RangeTo>());
                        let end = vm_try!(range.end.as_usize());
                        break 'out this.get(..end);
                    }
                    Range::HASH => {
                        let range = vm_try!(value.borrow_ref::<Range>());
                        let start = vm_try!(range.start.as_usize());
                        let end = vm_try!(range.end.as_usize());
                        break 'out this.get(start..end);
                    }
                    _ => {}
                }
            };

            let index = vm_try!(usize::from_value(index));

            let Some(value) = this.get(index) else {
                return VmResult::Ok(None);
            };

            return VmResult::Ok(Some(value.clone()));
        };

        let Some(values) = slice else {
            return VmResult::Ok(None);
        };

        let vec = vm_try!(alloc::Vec::try_from(values));
        VmResult::Ok(Some(vm_try!(Value::vec(vec))))
    }

    pub(crate) fn hash_with(
        &self,
        hasher: &mut Hasher,
        caller: &mut dyn ProtocolCaller,
    ) -> VmResult<()> {
        for value in self.inner.iter() {
            vm_try!(value.hash_with(hasher, caller));
        }

        VmResult::Ok(())
    }
}

impl TryClone for Vec {
    fn try_clone(&self) -> alloc::Result<Self> {
        Ok(Self {
            inner: self.inner.try_clone()?,
        })
    }
}

impl fmt::Debug for Vec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(&*self.inner).finish()
    }
}

impl ops::Deref for Vec {
    type Target = [Value];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for Vec {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl IntoIterator for Vec {
    type Item = Value;
    type IntoIter = alloc::vec::IntoIter<Value>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a> IntoIterator for &'a Vec {
    type Item = &'a Value;
    type IntoIter = slice::Iter<'a, Value>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a> IntoIterator for &'a mut Vec {
    type Item = &'a mut Value;
    type IntoIter = slice::IterMut<'a, Value>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl TryFrom<rust_alloc::vec::Vec<Value>> for Vec {
    type Error = alloc::Error;

    #[inline]
    fn try_from(values: rust_alloc::vec::Vec<Value>) -> Result<Self, Self::Error> {
        let mut inner = alloc::Vec::try_with_capacity(values.len())?;

        for value in values {
            inner.try_push(value)?;
        }

        Ok(Self { inner })
    }
}

impl TryFrom<rust_alloc::boxed::Box<[Value]>> for Vec {
    type Error = alloc::Error;

    #[inline]
    fn try_from(inner: rust_alloc::boxed::Box<[Value]>) -> Result<Self, Self::Error> {
        Vec::try_from(inner.into_vec())
    }
}

impl From<alloc::Vec<Value>> for Vec {
    #[inline]
    fn from(inner: alloc::Vec<Value>) -> Self {
        Self { inner }
    }
}

impl<T> FromValue for rust_alloc::vec::Vec<T>
where
    T: FromValue,
{
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        let vec = value.downcast::<Vec>()?;

        let mut output = rust_alloc::vec::Vec::with_capacity(vec.len());

        for value in vec {
            output.push(T::from_value(value)?);
        }

        Ok(output)
    }
}

impl<T> FromValue for alloc::Vec<T>
where
    T: FromValue,
{
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        let vec = value.downcast::<Vec>()?;

        let mut output = alloc::Vec::try_with_capacity(vec.len())?;

        for value in vec {
            output.try_push(T::from_value(value)?)?;
        }

        Ok(output)
    }
}

impl UnsafeToRef for [Value] {
    type Guard = RawAnyGuard;

    #[inline]
    unsafe fn unsafe_to_ref<'a>(value: Value) -> Result<(&'a Self, Self::Guard), RuntimeError> {
        let vec = value.into_ref::<Vec>()?;
        let (vec, guard) = Ref::into_raw(vec);
        Ok((vec.as_ref().as_slice(), guard))
    }
}

impl<T, const N: usize> ToValue for [T; N]
where
    T: ToValue,
{
    #[inline]
    fn to_value(self) -> Result<Value, RuntimeError> {
        let mut inner = alloc::Vec::try_with_capacity(self.len())?;

        for value in self {
            let value = value.to_value()?;
            inner.try_push(value)?;
        }

        Ok(Value::try_from(Vec { inner })?)
    }
}

impl<T> ToValue for alloc::Vec<T>
where
    T: ToValue,
{
    #[inline]
    fn to_value(self) -> Result<Value, RuntimeError> {
        let mut inner = alloc::Vec::try_with_capacity(self.len())?;

        for value in self {
            let value = value.to_value()?;
            inner.try_push(value)?;
        }

        Ok(Value::try_from(Vec { inner })?)
    }
}

impl<T> ToValue for rust_alloc::vec::Vec<T>
where
    T: ToValue,
{
    #[inline]
    fn to_value(self) -> Result<Value, RuntimeError> {
        let mut inner = alloc::Vec::try_with_capacity(self.len())?;

        for value in self {
            let value = value.to_value()?;
            inner.try_push(value)?;
        }

        Ok(Value::try_from(Vec { inner })?)
    }
}
