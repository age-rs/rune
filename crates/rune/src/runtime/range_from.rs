use core::cmp::Ordering;
use core::fmt;
use core::ops;

use crate as rune;
use crate::alloc::clone::TryClone;
use crate::Any;

use super::{
    EnvProtocolCaller, FromValue, Inline, ProtocolCaller, Repr, RuntimeError, ToValue, Value,
    VmError, VmErrorKind,
};

/// Type for a from range expression `start..`.
///
/// # Examples
///
/// ```rune
/// use std::ops::RangeFrom;
///
/// let range = 0..;
///
/// assert!(!range.contains(-10));
/// assert!(range.contains(5));
/// assert!(range.contains(10));
/// assert!(range.contains(20));
///
/// assert!(range is RangeFrom);
/// ```
///
/// Ranges can contain any type:
///
/// ```rune
/// let range = 'a'..;
/// assert_eq!(range.start, 'a');
/// range.start = 'b';
/// assert_eq!(range.start, 'b');
/// ```
///
/// Certain ranges can be used as iterators:
///
/// ```rune
/// let range = 'a'..;
/// assert_eq!(range.iter().take(5).collect::<Vec>(), ['a', 'b', 'c', 'd', 'e']);
/// ```
///
/// # Rust examples
///
/// ```rust
/// use rune::runtime::RangeFrom;
///
/// let start = rune::to_value(1)?;
/// let _ = RangeFrom::new(start);
/// # Ok::<_, rune::support::Error>(())
/// ```
#[derive(Any, Clone, TryClone)]
#[try_clone(crate)]
#[rune(crate, constructor, item = ::std::ops)]
pub struct RangeFrom {
    /// The start value of the range.
    #[rune(get, set)]
    pub start: Value,
}

impl RangeFrom {
    /// Construct a new range.
    pub fn new(start: Value) -> Self {
        Self { start }
    }

    /// Iterate over the range.
    ///
    /// # Panics
    ///
    /// This panics if the range is not a well-defined range.
    ///
    /// # Examples
    ///
    /// ```rune
    /// let range = 'a'..;
    /// assert_eq!(range.iter().take(5).collect::<Vec>(), ['a', 'b', 'c', 'd', 'e']);
    /// ```
    ///
    /// Cannot construct an iterator over floats:
    ///
    /// ```rune,should_panic
    /// let range = 1.0..;
    /// range.iter()
    /// ```
    #[rune::function(keep)]
    pub fn iter(&self) -> Result<Value, VmError> {
        let value = match self.start.as_ref() {
            Repr::Inline(Inline::Unsigned(start)) => crate::to_value(RangeFromIter::new(*start..))?,
            Repr::Inline(Inline::Signed(start)) => crate::to_value(RangeFromIter::new(*start..))?,
            Repr::Inline(Inline::Char(start)) => crate::to_value(RangeFromIter::new(*start..))?,
            start => {
                return Err(VmError::new(VmErrorKind::UnsupportedIterRangeFrom {
                    start: start.type_info(),
                }));
            }
        };

        Ok(value)
    }

    /// Build an iterator over the range.
    ///
    /// # Panics
    ///
    /// This panics if the range is not a well-defined range.
    ///
    /// # Examples
    ///
    /// ```rune
    /// let vec = [];
    ///
    /// for value in 'a'.. {
    ///     vec.push(value);
    ///
    ///     if value == 'e' {
    ///        break;
    ///     }
    /// }
    ///
    /// assert_eq!(vec, ['a', 'b', 'c', 'd', 'e']);
    /// ```
    ///
    /// Cannot construct an iterator over floats:
    ///
    /// ```rune,should_panic
    /// let range = 1.0..;
    ///
    /// for value in 1.0 .. {
    /// }
    /// ```
    #[rune::function(keep, protocol = INTO_ITER)]
    pub fn into_iter(&self) -> Result<Value, VmError> {
        self.iter()
    }

    /// Test the range for partial equality.
    ///
    /// # Examples
    ///
    /// ```rune
    /// let range = 'a'..;
    /// assert!(range == ('a'..));
    /// assert!(range != ('b'..));
    ///
    /// let range = 1.0..;
    /// assert!(range == (1.0..));
    /// assert!(range != (f64::NAN..));
    /// assert!((f64::NAN..) != (f64::NAN..));
    /// ```
    #[rune::function(keep, protocol = PARTIAL_EQ)]
    pub fn partial_eq(&self, other: &Self) -> Result<bool, VmError> {
        self.partial_eq_with(other, &mut EnvProtocolCaller)
    }

    pub(crate) fn partial_eq_with(
        &self,
        b: &Self,
        caller: &mut dyn ProtocolCaller,
    ) -> Result<bool, VmError> {
        Value::partial_eq_with(&self.start, &b.start, caller)
    }

    /// Test the range for total equality.
    ///
    /// # Examples
    ///
    /// ```rune
    /// use std::ops::eq;
    ///
    /// let range = 'a'..;
    /// assert!(eq(range, 'a'..));
    /// assert!(!eq(range, 'b'..));
    /// ```
    #[rune::function(keep, protocol = EQ)]
    pub fn eq(&self, other: &Self) -> Result<bool, VmError> {
        self.eq_with(other, &mut EnvProtocolCaller)
    }

    pub(crate) fn eq_with(
        &self,
        b: &Self,
        caller: &mut dyn ProtocolCaller,
    ) -> Result<bool, VmError> {
        Value::eq_with(&self.start, &b.start, caller)
    }

    /// Test the range for partial ordering.
    ///
    /// # Examples
    ///
    /// ```rune
    /// assert!(('a'..) < ('b'..));
    /// assert!(('c'..) > ('b'..));
    /// assert!(!((f64::NAN..) > (f64::INFINITY..)));
    /// assert!(!((f64::NAN..) < (f64::INFINITY..)));
    /// ```
    #[rune::function(keep, protocol = PARTIAL_CMP)]
    pub fn partial_cmp(&self, other: &Self) -> Result<Option<Ordering>, VmError> {
        self.partial_cmp_with(other, &mut EnvProtocolCaller)
    }

    pub(crate) fn partial_cmp_with(
        &self,
        b: &Self,
        caller: &mut dyn ProtocolCaller,
    ) -> Result<Option<Ordering>, VmError> {
        Value::partial_cmp_with(&self.start, &b.start, caller)
    }

    /// Test the range for total ordering.
    ///
    /// # Examples
    ///
    /// ```rune
    /// use std::ops::cmp;
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(cmp('a'.., 'b'..), Ordering::Less);
    /// assert_eq!(cmp('c'.., 'b'..), Ordering::Greater);
    /// ```
    #[rune::function(keep, protocol = CMP)]
    pub fn cmp(&self, other: &Self) -> Result<Ordering, VmError> {
        self.cmp_with(other, &mut EnvProtocolCaller)
    }

    pub(crate) fn cmp_with(
        &self,
        b: &Self,
        caller: &mut dyn ProtocolCaller,
    ) -> Result<Ordering, VmError> {
        Value::cmp_with(&self.start, &b.start, caller)
    }

    /// Test if the range contains the given value.
    ///
    /// The check is performed using the [`PARTIAL_CMP`] protocol.
    ///
    /// # Examples
    ///
    /// ```rune
    /// use std::ops::RangeFrom;
    ///
    /// let range = 0..;
    ///
    /// assert!(!range.contains(-10));
    /// assert!(range.contains(5));
    /// assert!(range.contains(10));
    /// assert!(range.contains(20));
    ///
    /// assert!(range is RangeFrom);
    /// ```
    #[rune::function(keep)]
    pub(crate) fn contains(&self, value: Value) -> Result<bool, VmError> {
        self.contains_with(value, &mut EnvProtocolCaller)
    }

    pub(crate) fn contains_with(
        &self,
        value: Value,
        caller: &mut dyn ProtocolCaller,
    ) -> Result<bool, VmError> {
        Ok(matches!(
            Value::partial_cmp_with(&self.start, &value, caller)?,
            Some(Ordering::Less | Ordering::Equal)
        ))
    }
}

impl fmt::Debug for RangeFrom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}..", self.start)
    }
}

impl<Idx> ToValue for ops::RangeFrom<Idx>
where
    Idx: ToValue,
{
    fn to_value(self) -> Result<Value, RuntimeError> {
        let start = self.start.to_value()?;
        let range = RangeFrom::new(start);
        Ok(Value::new(range)?)
    }
}

impl<Idx> FromValue for ops::RangeFrom<Idx>
where
    Idx: FromValue,
{
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        let range = value.downcast::<RangeFrom>()?;
        let start = Idx::from_value(range.start)?;
        Ok(ops::RangeFrom { start })
    }
}

range_iter!(RangeFrom, RangeFromIter<T>, {
    #[rune::function(instance, keep, protocol = SIZE_HINT)]
    #[inline]
    pub(crate) fn size_hint(&self) -> (u64, Option<u64>) {
        (u64::MAX, None)
    }
});
