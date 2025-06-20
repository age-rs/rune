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
use crate::shared::FixedVec;
use crate::{Any, TypeHash};

use super::{
    EnvProtocolCaller, Formatter, FromValue, Hasher, ProtocolCaller, Range, RangeFrom, RangeFull,
    RangeInclusive, RangeTo, RangeToInclusive, RawAnyGuard, Ref, RuntimeError, ToValue,
    UnsafeToRef, Value, VmError, VmErrorKind,
};

/// Struct representing a dynamic vector.
///
/// # Examples
///
/// ```
/// let mut vec = rune::runtime::Vec::new();
/// assert!(vec.is_empty());
///
/// vec.push_value(42)?;
/// vec.push_value(true)?;
/// assert_eq!(2, vec.len());
///
/// assert_eq!(Some(42), vec.get_value(0)?);
/// assert_eq!(Some(true), vec.get_value(1)?);
/// assert_eq!(None::<bool>, vec.get_value(2)?);
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
    pub fn set(&mut self, index: usize, value: Value) -> Result<(), VmError> {
        let Some(v) = self.inner.get_mut(index) else {
            return Err(VmError::new(VmErrorKind::OutOfRange {
                index: index.into(),
                length: self.len().into(),
            }));
        };

        *v = value;
        Ok(())
    }

    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the
    /// difference, with each additional slot filled with `value`. If `new_len`
    /// is less than `len`, the `Vec` is simply truncated.
    pub fn resize(&mut self, new_len: usize, value: Value) -> Result<(), VmError> {
        if value.is_inline() {
            self.inner.try_resize(new_len, value)?;
        } else {
            let len = self.inner.len();

            if new_len > len {
                for _ in 0..new_len - len {
                    let value = value.clone_with(&mut EnvProtocolCaller)?;
                    self.inner.try_push(value)?;
                }
            } else {
                self.inner.truncate(new_len);
            }
        }

        Ok(())
    }

    /// Appends an element to the back of a dynamic vector.
    pub fn push(&mut self, value: Value) -> alloc::Result<()> {
        self.inner.try_push(value)
    }

    /// Appends an element to the back of a dynamic vector, converting it as
    /// necessary through the [`ToValue`] trait.
    pub fn push_value<T>(&mut self, value: T) -> Result<(), VmError>
    where
        T: ToValue,
    {
        self.inner.try_push(value.to_value()?)?;
        Ok(())
    }

    /// Get the value at the given index.
    pub fn get<I>(&self, index: I) -> Option<&I::Output>
    where
        I: SliceIndex<[Value]>,
    {
        self.inner.get(index)
    }

    /// Get the given value at the given index.
    pub fn get_value<T>(&self, index: usize) -> Result<Option<T>, VmError>
    where
        T: FromValue,
    {
        let Some(value) = self.inner.get(index) else {
            return Ok(None);
        };

        Ok(Some(T::from_value(value.clone())?))
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
    pub fn insert(&mut self, index: usize, value: Value) -> alloc::Result<()> {
        self.inner.try_insert(index, value)
    }

    /// Extend this vector with something that implements the into_iter
    /// protocol.
    pub fn extend(&mut self, value: Value) -> Result<(), VmError> {
        let mut it = value.into_iter()?;

        while let Some(value) = it.next()? {
            self.push(value)?;
        }

        Ok(())
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
    ) -> Result<(), VmError> {
        let mut it = this.iter().peekable();
        write!(f, "[")?;

        while let Some(value) = it.next() {
            value.debug_fmt_with(f, caller)?;

            if it.peek().is_some() {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")?;
        Ok(())
    }

    pub(crate) fn partial_eq_with(
        a: &[Value],
        b: Value,
        caller: &mut dyn ProtocolCaller,
    ) -> Result<bool, VmError> {
        let mut b = b.into_iter_with(caller)?;

        for a in a {
            let Some(b) = b.next()? else {
                return Ok(false);
            };

            if !Value::partial_eq_with(a, &b, caller)? {
                return Ok(false);
            }
        }

        if b.next()?.is_some() {
            return Ok(false);
        }

        Ok(true)
    }

    pub(crate) fn eq_with(
        a: &[Value],
        b: &[Value],
        eq: fn(&Value, &Value, &mut dyn ProtocolCaller) -> Result<bool, VmError>,
        caller: &mut dyn ProtocolCaller,
    ) -> Result<bool, VmError> {
        if a.len() != b.len() {
            return Ok(false);
        }

        for (a, b) in a.iter().zip(b.iter()) {
            if !eq(a, b, caller)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    pub(crate) fn partial_cmp_with(
        a: &[Value],
        b: &[Value],
        caller: &mut dyn ProtocolCaller,
    ) -> Result<Option<Ordering>, VmError> {
        let mut b = b.iter();

        for a in a.iter() {
            let Some(b) = b.next() else {
                return Ok(Some(Ordering::Greater));
            };

            match Value::partial_cmp_with(a, b, caller)? {
                Some(Ordering::Equal) => continue,
                other => return Ok(other),
            }
        }

        if b.next().is_some() {
            return Ok(Some(Ordering::Less));
        }

        Ok(Some(Ordering::Equal))
    }

    pub(crate) fn cmp_with(
        a: &[Value],
        b: &[Value],
        caller: &mut dyn ProtocolCaller,
    ) -> Result<Ordering, VmError> {
        let mut b = b.iter();

        for a in a.iter() {
            let Some(b) = b.next() else {
                return Ok(Ordering::Greater);
            };

            match Value::cmp_with(a, b, caller)? {
                Ordering::Equal => continue,
                other => return Ok(other),
            }
        }

        if b.next().is_some() {
            return Ok(Ordering::Less);
        }

        Ok(Ordering::Equal)
    }

    /// This is a common get implementation that can be used across linear
    /// types, such as vectors and tuples.
    pub(crate) fn index_get(this: &[Value], index: Value) -> Result<Option<Value>, VmError> {
        let slice: Option<&[Value]> = 'out: {
            if let Some(value) = index.as_any() {
                match value.type_hash() {
                    RangeFrom::HASH => {
                        let range = value.borrow_ref::<RangeFrom>()?;
                        let start = range.start.as_usize()?;
                        break 'out this.get(start..);
                    }
                    RangeFull::HASH => {
                        _ = value.borrow_ref::<RangeFull>()?;
                        break 'out this.get(..);
                    }
                    RangeInclusive::HASH => {
                        let range = value.borrow_ref::<RangeInclusive>()?;
                        let start = range.start.as_usize()?;
                        let end = range.end.as_usize()?;
                        break 'out this.get(start..=end);
                    }
                    RangeToInclusive::HASH => {
                        let range = value.borrow_ref::<RangeToInclusive>()?;
                        let end = range.end.as_usize()?;
                        break 'out this.get(..=end);
                    }
                    RangeTo::HASH => {
                        let range = value.borrow_ref::<RangeTo>()?;
                        let end = range.end.as_usize()?;
                        break 'out this.get(..end);
                    }
                    Range::HASH => {
                        let range = value.borrow_ref::<Range>()?;
                        let start = range.start.as_usize()?;
                        let end = range.end.as_usize()?;
                        break 'out this.get(start..end);
                    }
                    _ => {}
                }
            };

            let index = usize::from_value(index)?;

            let Some(value) = this.get(index) else {
                return Ok(None);
            };

            return Ok(Some(value.clone()));
        };

        let Some(values) = slice else {
            return Ok(None);
        };

        let vec = alloc::Vec::try_from(values)?;
        Ok(Some(Value::vec(vec)?))
    }

    pub(crate) fn hash_with(
        &self,
        hasher: &mut Hasher,
        caller: &mut dyn ProtocolCaller,
    ) -> Result<(), VmError> {
        for value in self.inner.iter() {
            value.hash_with(hasher, caller)?;
        }

        Ok(())
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

impl<T, const N: usize> FromValue for [T; N]
where
    T: FromValue,
{
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        let vec = value.into_ref::<Vec>()?;

        let values = vec.as_slice();

        if values.len() != N {
            return Err(RuntimeError::new(VmErrorKind::ExpectedVecLength {
                actual: vec.len(),
                expected: N,
            }));
        };

        let mut output = FixedVec::<T, N>::new();

        for v in values {
            output.try_push(T::from_value(v.clone())?)?;
        }

        Ok(output.into_inner())
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
