use core::cmp::Ordering;
use core::fmt;
use core::ops;

use crate as rune;
use crate::alloc::clone::TryClone;
use crate::Any;

use super::{
    EnvProtocolCaller, FromValue, Inline, ProtocolCaller, Repr, RuntimeError, StepsBetween,
    ToValue, Value, VmError, VmErrorKind,
};

/// Type for an inclusive range expression `start..=end`.
///
/// # Examples
///
/// ```rune
/// let range = 0..=10;
///
/// assert!(!range.contains(-10));
/// assert!(range.contains(5));
/// assert!(range.contains(10));
/// assert!(!range.contains(20));
///
/// assert!(range is std::ops::RangeInclusive);
/// ```
///
/// Ranges can contain any type:
///
/// ```rune
/// let range = 'a'..='f';
/// assert_eq!(range.start, 'a');
/// range.start = 'b';
/// assert_eq!(range.start, 'b');
/// assert_eq!(range.end, 'f');
/// range.end = 'g';
/// assert_eq!(range.end, 'g');
/// ```
///
/// Certain ranges can be used as iterators:
///
/// ```rune
/// let range = 'a'..='e';
/// assert_eq!(range.iter().collect::<Vec>(), ['a', 'b', 'c', 'd', 'e']);
/// ```
///
/// # Rust Examples
///
/// ```rust
/// use rune::runtime::RangeInclusive;
///
/// let start = rune::to_value(1)?;
/// let end = rune::to_value(10)?;
/// let _ = RangeInclusive::new(start, end);
/// # Ok::<_, rune::support::Error>(())
/// ```
#[derive(Any, Clone, TryClone)]
#[try_clone(crate)]
#[rune(crate, constructor, item = ::std::ops)]
pub struct RangeInclusive {
    /// The start value of the range.
    #[rune(get, set)]
    pub start: Value,
    /// The end value of the range.
    #[rune(get, set)]
    pub end: Value,
}

impl RangeInclusive {
    /// Construct a new range.
    pub fn new(start: Value, end: Value) -> Self {
        Self { start, end }
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
    /// let range = 'a'..='e';
    /// assert_eq!(range.iter().collect::<Vec>(), ['a', 'b', 'c', 'd', 'e']);
    /// ```
    ///
    /// Cannot construct an iterator over floats:
    ///
    /// ```rune,should_panic
    /// let range = 1.0..=2.0;
    /// range.iter()
    /// ```
    #[rune::function(keep)]
    pub fn iter(&self) -> Result<Value, VmError> {
        let value = match (self.start.as_ref(), self.end.as_ref()) {
            (Repr::Inline(Inline::Unsigned(start)), Repr::Inline(end)) => {
                let end = end.as_integer::<u64>()?;
                rune::to_value(RangeInclusiveIter::new(*start..=end))?
            }
            (Repr::Inline(Inline::Signed(start)), Repr::Inline(end)) => {
                let end = end.as_integer::<i64>()?;
                rune::to_value(RangeInclusiveIter::new(*start..=end))?
            }
            (Repr::Inline(Inline::Char(start)), Repr::Inline(Inline::Char(end))) => {
                rune::to_value(RangeInclusiveIter::new(*start..=*end))?
            }
            (start, end) => {
                return Err(VmError::new(VmErrorKind::UnsupportedIterRangeInclusive {
                    start: start.type_info(),
                    end: end.type_info(),
                }))
            }
        };

        Ok(value)
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
    /// let vec = [];
    ///
    /// for value in 'a'..='e' {
    ///     vec.push(value);
    /// }
    ///
    /// assert_eq!(vec, ['a', 'b', 'c', 'd', 'e']);
    /// ```
    ///
    /// Cannot construct an iterator over floats:
    ///
    /// ```rune,should_panic
    /// for value in 1.0..=2.0 {
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
    /// let range = 'a'..='e';
    /// assert!(range == ('a'..='e'));
    /// assert!(range != ('b'..='e'));
    ///
    /// let range = 1.0..=2.0;
    /// assert!(range == (1.0..=2.0));
    /// assert!(range != (f64::NAN..=2.0));
    /// assert!((f64::NAN..=2.0) != (f64::NAN..=2.0));
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
        if !Value::partial_eq_with(&self.start, &b.start, caller)? {
            return Ok(false);
        }

        Value::partial_eq_with(&self.end, &b.end, caller)
    }

    /// Test the range for total equality.
    ///
    /// # Examples
    ///
    /// ```rune
    /// use std::ops::eq;
    ///
    /// let range = 'a'..='e';
    /// assert!(eq(range, 'a'..='e'));
    /// assert!(!eq(range, 'b'..='e'));
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
        if !Value::eq_with(&self.start, &b.start, caller)? {
            return Ok(false);
        }

        Value::eq_with(&self.end, &b.end, caller)
    }

    /// Test the range for partial ordering.
    ///
    /// # Examples
    ///
    /// ```rune
    /// assert!(('a'..='e') < ('b'..='e'));
    /// assert!(('c'..='e') > ('b'..='e'));
    /// assert!(!((f64::NAN..=2.0) > (f64::INFINITY..=2.0)));
    /// assert!(!((f64::NAN..=2.0) < (f64::INFINITY..=2.0)));
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
        match Value::partial_cmp_with(&self.start, &b.start, caller)? {
            Some(Ordering::Equal) => (),
            other => return Ok(other),
        }

        Value::partial_cmp_with(&self.end, &b.end, caller)
    }

    /// Test the range for total ordering.
    ///
    /// # Examples
    ///
    /// ```rune
    /// use std::ops::cmp;
    /// use std::cmp::Ordering;
    ///
    /// assert_eq!(cmp('a'..='e', 'b'..='e'), Ordering::Less);
    /// assert_eq!(cmp('c'..='e', 'b'..='e'), Ordering::Greater);
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
        match Value::cmp_with(&self.start, &b.start, caller)? {
            Ordering::Equal => (),
            other => return Ok(other),
        }

        Value::cmp_with(&self.end, &b.end, caller)
    }

    /// Test if the range contains the given value.
    ///
    /// The check is performed using the [`PARTIAL_CMP`] protocol.
    ///
    /// # Examples
    ///
    /// ```rune
    /// let range = 0..=10;
    ///
    /// assert!(!range.contains(-10));
    /// assert!(range.contains(5));
    /// assert!(range.contains(10));
    /// assert!(!range.contains(20));
    ///
    /// assert!(range is std::ops::RangeInclusive);
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
        match Value::partial_cmp_with(&self.start, &value, caller)? {
            Some(Ordering::Less | Ordering::Equal) => {}
            _ => return Ok(false),
        }

        Ok(matches!(
            Value::partial_cmp_with(&self.end, &value, caller)?,
            Some(Ordering::Greater | Ordering::Equal)
        ))
    }
}

impl fmt::Debug for RangeInclusive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}..={:?}", self.start, self.end)
    }
}

impl<Idx> ToValue for ops::RangeInclusive<Idx>
where
    Idx: ToValue,
{
    fn to_value(self) -> Result<Value, RuntimeError> {
        let (start, end) = self.into_inner();
        let start = start.to_value()?;
        let end = end.to_value()?;
        Ok(Value::new(RangeInclusive::new(start, end))?)
    }
}

impl<Idx> FromValue for ops::RangeInclusive<Idx>
where
    Idx: FromValue,
{
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        let range = value.downcast::<RangeInclusive>()?;
        let start = Idx::from_value(range.start)?;
        let end = Idx::from_value(range.end)?;
        Ok(start..=end)
    }
}

double_ended_range_iter!(RangeInclusive, RangeInclusiveIter<T>, {
    #[rune::function(instance, keep, protocol = SIZE_HINT)]
    #[inline]
    pub(crate) fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[rune::function(instance, keep, protocol = LEN)]
    #[inline]
    pub(crate) fn len(&self) -> Result<usize, VmError>
    where
        T: Copy + StepsBetween + fmt::Debug,
    {
        let Some(result) = T::steps_between(*self.iter.start(), *self.iter.end()) else {
            return Err(VmError::panic(format!(
                "could not calculate length of range {:?}..={:?}",
                self.iter.start(),
                self.iter.end()
            )));
        };

        Ok(result)
    }
});
