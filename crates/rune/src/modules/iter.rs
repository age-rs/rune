//! Iterators.

use crate as rune;
use crate::alloc;
use crate::alloc::prelude::*;
use crate::modules::collections::{HashMap, HashSet, VecDeque};
use crate::runtime::range::RangeIter;
use crate::runtime::{
    Address, FromValue, Function, Inline, Object, Output, OwnedTuple, Protocol, Repr, TypeHash,
    Value, Vec, VmError, VmErrorKind,
};
use crate::shared::Caller;
use crate::{docstring, Any, ContextError, Module, Params};

/// Rune support for iterators.
///
/// This module contains types and methods for working with iterators in Rune.
#[rune::module(::std::iter)]
pub fn module() -> Result<Module, ContextError> {
    let mut m = Module::from_meta(self::module__meta)?;

    m.ty::<Rev>()?;
    m.function_meta(Rev::next__meta)?;
    m.function_meta(Rev::next_back__meta)?;
    m.function_meta(Rev::size_hint__meta)?;
    m.function_meta(Rev::len__meta)?;
    m.implement_trait::<Rev>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Rev>(rune::item!(::std::iter::DoubleEndedIterator))?;
    m.implement_trait::<Rev>(rune::item!(::std::iter::ExactSizeIterator))?;

    m.ty::<Chain>()?;
    m.function_meta(Chain::next__meta)?;
    m.function_meta(Chain::next_back__meta)?;
    m.function_meta(Chain::size_hint__meta)?;
    m.function_meta(Chain::len__meta)?;
    m.implement_trait::<Chain>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Chain>(rune::item!(::std::iter::DoubleEndedIterator))?;
    m.implement_trait::<Chain>(rune::item!(::std::iter::ExactSizeIterator))?;

    m.ty::<Enumerate>()?;
    m.function_meta(Enumerate::next__meta)?;
    m.function_meta(Enumerate::next_back__meta)?;
    m.function_meta(Enumerate::size_hint__meta)?;
    m.function_meta(Enumerate::len__meta)?;
    m.implement_trait::<Enumerate>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Enumerate>(rune::item!(::std::iter::DoubleEndedIterator))?;
    m.implement_trait::<Enumerate>(rune::item!(::std::iter::ExactSizeIterator))?;

    m.ty::<Filter>()?;
    m.function_meta(Filter::next__meta)?;
    m.function_meta(Filter::next_back__meta)?;
    m.function_meta(Filter::size_hint__meta)?;
    m.implement_trait::<Filter>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Filter>(rune::item!(::std::iter::DoubleEndedIterator))?;

    m.ty::<Map>()?;
    m.function_meta(Map::next__meta)?;
    m.function_meta(Map::next_back__meta)?;
    m.function_meta(Map::size_hint__meta)?;
    m.function_meta(Map::len__meta)?;
    m.implement_trait::<Map>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Map>(rune::item!(::std::iter::DoubleEndedIterator))?;
    m.implement_trait::<Map>(rune::item!(::std::iter::ExactSizeIterator))?;

    m.ty::<FilterMap>()?;
    m.function_meta(FilterMap::next__meta)?;
    m.function_meta(FilterMap::next_back__meta)?;
    m.implement_trait::<FilterMap>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<FilterMap>(rune::item!(::std::iter::DoubleEndedIterator))?;

    m.ty::<FlatMap>()?;
    m.function_meta(FlatMap::next__meta)?;
    m.function_meta(FlatMap::next_back__meta)?;
    m.function_meta(FlatMap::size_hint__meta)?;
    m.implement_trait::<FlatMap>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<FlatMap>(rune::item!(::std::iter::DoubleEndedIterator))?;

    m.ty::<Peekable>()?;
    m.function_meta(Peekable::next__meta)?;
    m.function_meta(Peekable::next_back__meta)?;
    m.function_meta(Peekable::size_hint__meta)?;
    m.function_meta(Peekable::len__meta)?;
    m.implement_trait::<Peekable>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Peekable>(rune::item!(::std::iter::DoubleEndedIterator))?;
    m.implement_trait::<Peekable>(rune::item!(::std::iter::ExactSizeIterator))?;
    m.function_meta(Peekable::peek__meta)?;

    m.ty::<Skip>()?;
    m.function_meta(Skip::next__meta)?;
    m.function_meta(Skip::next_back__meta)?;
    m.function_meta(Skip::size_hint__meta)?;
    m.function_meta(Skip::len__meta)?;
    m.implement_trait::<Skip>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Skip>(rune::item!(::std::iter::DoubleEndedIterator))?;
    m.implement_trait::<Skip>(rune::item!(::std::iter::ExactSizeIterator))?;

    m.ty::<Take>()?;
    m.function_meta(Take::next__meta)?;
    m.function_meta(Take::next_back__meta)?;
    m.function_meta(Take::size_hint__meta)?;
    m.function_meta(Take::len__meta)?;
    m.implement_trait::<Take>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Take>(rune::item!(::std::iter::DoubleEndedIterator))?;
    m.implement_trait::<Take>(rune::item!(::std::iter::ExactSizeIterator))?;

    {
        let mut t = m.define_trait(["ExactSizeIterator"])?;

        t.docs(docstring! {
            /// An iterator that knows its exact length.
            ///
            /// Many [`Iterator`]s don't know how many times they will iterate, but some do.
            /// If an iterator knows how many times it can iterate, providing access to
            /// that information can be useful. For example, if you want to iterate
            /// backwards, a good start is to know where the end is.
            ///
            /// When implementing an `ExactSizeIterator`, you must also implement
            /// [`Iterator`]. When doing so, the implementation of [`Iterator::size_hint`]
            /// *must* return the exact size of the iterator.
            ///
            /// The [`len`] method has a default implementation, so you usually shouldn't
            /// implement it. However, you may be able to provide a more performant
            /// implementation than the default, so overriding it in this case makes sense.
            ///
            /// Note that this trait is a safe trait and as such does *not* and *cannot*
            /// guarantee that the returned length is correct. This means that `unsafe`
            /// code **must not** rely on the correctness of [`Iterator::size_hint`]. The
            /// unstable and unsafe [`TrustedLen`](super::marker::TrustedLen) trait gives
            /// this additional guarantee.
            ///
            /// [`len`]: ExactSizeIterator::len
            ///
            /// # When *shouldn't* an adapter be `ExactSizeIterator`?
            ///
            /// If an adapter makes an iterator *longer*, then it's usually incorrect for
            /// that adapter to implement `ExactSizeIterator`.  The inner exact-sized
            /// iterator might already be `usize::MAX`-long, and thus the length of the
            /// longer adapted iterator would no longer be exactly representable in `usize`.
            ///
            /// This is why [`Chain<A, B>`](crate::iter::Chain) isn't `ExactSizeIterator`,
            /// even when `A` and `B` are both `ExactSizeIterator`.
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```
            /// // a finite range knows exactly how many times it will iterate
            /// let five = (0..5).iter();
            ///
            /// assert_eq!(five.len(), 5);
            /// ```
        })?;

        t.handler(|cx| {
            _ = cx.find(&Protocol::LEN)?;
            Ok(())
        })?;

        t.function("len")?
            .argument_types::<(Value,)>()?
            .return_type::<usize>()?
            .docs(docstring! {
                /// Returns the exact remaining length of the iterator.
                ///
                /// The implementation ensures that the iterator will return
                /// exactly `len()` more times a [`Some(T)`] value, before
                /// returning [`None`]. This method has a default
                /// implementation, so you usually should not implement it
                /// directly. However, if you can provide a more efficient
                /// implementation, you can do so. See the [trait-level] docs
                /// for an example.
                ///
                /// This function has the same safety guarantees as the
                /// [`Iterator::size_hint`] function.
                ///
                /// [trait-level]: ExactSizeIterator
                /// [`Some(T)`]: Some
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```
                /// // a finite range knows exactly how many times it will iterate
                /// let range = (0..5).iter();
                ///
                /// assert_eq!(range.len(), 5);
                /// let _ = range.next();
                /// assert_eq!(range.len(), 4);
                /// ```
            })?;
    }

    {
        let mut t = m.define_trait(["Iterator"])?;

        t.docs(docstring! {
            /// A trait for dealing with iterators.
        })?;

        t.handler(|cx| {
            let next = cx.find(&Protocol::NEXT)?;
            let next = Caller::<(Value,), 1, Option<Value>>::new(next);

            let size_hint =
                cx.find_or_define(&Protocol::SIZE_HINT, |_: Value| (0usize, None::<usize>))?;

            let size_hint = Caller::<(&Value,), 1, (usize, Option<usize>)>::new(size_hint);

            cx.find_or_define(&Protocol::NTH, {
                let next = next.clone();

                move |iter: Value, mut n: usize| loop {
                    let Some(value) = next.call((iter.clone(),))? else {
                        break Ok(None);
                    };

                    if n == 0 {
                        break Ok::<_, VmError>(Some(value));
                    }

                    n -= 1;
                }
            })?;

            cx.function(&Protocol::INTO_ITER, |value: Value| value)?;

            cx.function("into_iter", |value: Value| value)?;

            {
                let next = next.clone();

                cx.function("count", move |iter: Value| {
                    let mut n = 0usize;

                    loop {
                        if next.call((iter.clone(),))?.is_none() {
                            break Ok::<_, VmError>(n);
                        };

                        n += 1;
                    }
                })?;
            }

            {
                let next = next.clone();

                cx.function("fold", move |iter: Value, mut acc: Value, f: Function| {
                    loop {
                        let Some(value) = next.call((iter.clone(),))? else {
                            break Ok::<_, VmError>(acc);
                        };

                        acc = f.call((acc, value))?;
                    }
                })?;
            }

            {
                let next = next.clone();

                cx.function("reduce", move |iter: Value, f: Function| {
                    let Some(mut acc) = next.call((iter.clone(),))? else {
                        return Ok::<_, VmError>(None);
                    };

                    while let Some(value) = next.call((iter.clone(),))? {
                        acc = f.call((acc, value))?;
                    }

                    Ok(Some(acc))
                })?;
            }

            {
                let next = next.clone();

                cx.function(
                    "find",
                    move |iter: Value, f: Function| -> Result<Option<Value>, VmError> {
                        loop {
                            let Some(value) = next.call((iter.clone(),))? else {
                                break Ok(None);
                            };

                            if f.call::<bool>((value.clone(),))? {
                                break Ok(Some(value));
                            }
                        }
                    },
                )?;
            }

            {
                let next = next.clone();

                cx.function(
                    "any",
                    move |iter: Value, f: Function| -> Result<bool, VmError> {
                        loop {
                            let Some(value) = next.call((iter.clone(),))? else {
                                break Ok(false);
                            };

                            if f.call::<bool>((value.clone(),))? {
                                break Ok(true);
                            }
                        }
                    },
                )?;
            }

            {
                let next = next.clone();

                cx.function(
                    "all",
                    move |iter: Value, f: Function| -> Result<bool, VmError> {
                        loop {
                            let Some(value) = next.call((iter.clone(),))? else {
                                break Ok(true);
                            };

                            if !f.call::<bool>((value.clone(),))? {
                                break Ok(false);
                            }
                        }
                    },
                )?;
            }

            {
                cx.function("chain", |a: Value, b: Value| -> Result<Chain, VmError> {
                    let b = b.protocol_into_iter()?;

                    Ok(Chain {
                        a: Some(a.clone()),
                        b: Some(b.clone()),
                    })
                })?;
                cx.function("enumerate", move |iter: Value| Enumerate { iter, count: 0 })?;
                cx.function("filter", move |iter: Value, f: Function| Filter { iter, f })?;
                cx.function("map", move |iter: Value, f: Function| Map {
                    iter: Some(iter),
                    f,
                })?;
                cx.function("filter_map", move |iter: Value, f: Function| FilterMap {
                    iter: Some(iter),
                    f,
                })?;
                cx.function("flat_map", move |iter: Value, f: Function| FlatMap {
                    map: Map {
                        iter: Some(iter),
                        f,
                    },
                    frontiter: None,
                    backiter: None,
                })?;
                cx.function("peekable", move |iter: Value| Peekable {
                    iter,
                    peeked: None,
                })?;
                cx.function("skip", move |iter: Value, n: usize| Skip { iter, n })?;
                cx.function("take", move |iter: Value, n: usize| Take { iter, n })?;
            }

            {
                let next = next.clone();
                let size_hint = size_hint.clone();

                cx.function(
                    Params::new("collect", [Vec::HASH]),
                    move |iter: Value| -> Result<Vec, VmError> {
                        let (cap, _) = size_hint.call((&iter,))?;
                        let mut vec = Vec::with_capacity(cap)?;

                        while let Some(value) = next.call((iter.clone(),))? {
                            vec.push(value)?;
                        }

                        Ok(vec)
                    },
                )?;
            }

            {
                let next = next.clone();
                let size_hint = size_hint.clone();

                cx.function(
                    Params::new("collect", [VecDeque::HASH]),
                    move |iter: Value| -> Result<VecDeque, VmError> {
                        let (cap, _) = size_hint.call((&iter,))?;
                        let mut vec = Vec::with_capacity(cap)?;

                        while let Some(value) = next.call((iter.clone(),))? {
                            vec.push(value)?;
                        }

                        Ok(VecDeque::from(vec))
                    },
                )?;
            }

            {
                let next = next.clone();
                let size_hint = size_hint.clone();

                cx.function(
                    Params::new("collect", [HashSet::HASH]),
                    move |iter: Value| -> Result<HashSet, VmError> {
                        let (cap, _) = size_hint.call((&iter,))?;
                        let mut set = HashSet::with_capacity(cap)?;

                        while let Some(value) = next.call((iter.clone(),))? {
                            set.insert(value)?;
                        }

                        Ok(set)
                    },
                )?;
            }

            {
                let next = next.with_return::<Option<(Value, Value)>>();
                let size_hint = size_hint.clone();

                cx.function(
                    Params::new("collect", [HashMap::HASH]),
                    move |iter: Value| -> Result<HashMap, VmError> {
                        let (cap, _) = size_hint.call((&iter,))?;
                        let mut map = HashMap::with_capacity(cap)?;

                        while let Some((key, value)) = next.call((iter.clone(),))? {
                            map.insert(key, value)?;
                        }

                        Ok(map)
                    },
                )?;
            }

            {
                let next = next.with_return::<Option<(String, Value)>>();
                let size_hint = size_hint.clone();

                cx.function(
                    Params::new("collect", [Object::HASH]),
                    move |iter: Value| -> Result<Object, VmError> {
                        let (cap, _) = size_hint.call((&iter,))?;
                        let mut map = Object::with_capacity(cap)?;

                        while let Some((key, value)) = next.call((iter.clone(),))? {
                            map.insert(key, value)?;
                        }

                        Ok(map)
                    },
                )?;
            }

            {
                let next = next.clone();
                let size_hint = size_hint.clone();

                cx.function(
                    Params::new("collect", [OwnedTuple::HASH]),
                    move |iter: Value| -> Result<OwnedTuple, VmError> {
                        let (cap, _) = size_hint.call((&iter,))?;
                        let mut vec = alloc::Vec::try_with_capacity(cap)?;

                        while let Some(value) = next.call((iter.clone(),))? {
                            vec.try_push(value)?;
                        }

                        Ok(OwnedTuple::try_from(vec)?)
                    },
                )?;
            }

            {
                let next = next.clone();

                cx.function(
                    Params::new("collect", [String::HASH]),
                    move |iter: Value| {
                        let mut string = String::new();

                        while let Some(value) = next.call((iter.clone(),))? {
                            match value.as_ref() {
                                Repr::Inline(Inline::Char(c)) => {
                                    string.try_push(*c)?;
                                }
                                Repr::Inline(value) => {
                                    return Err(VmError::expected::<String>(value.type_info()));
                                }
                                Repr::Dynamic(value) => {
                                    return Err(VmError::expected::<String>(value.type_info()));
                                }
                                Repr::Any(value) => match value.type_hash() {
                                    String::HASH => {
                                        let s = value.borrow_ref::<String>()?;
                                        string.try_push_str(&s)?;
                                    }
                                    _ => {
                                        return Err(VmError::expected::<String>(value.type_info()));
                                    }
                                },
                            }
                        }

                        Ok(string)
                    },
                )?;
            }

            macro_rules! ops {
                ($ty:ty) => {{
                    cx.function(Params::new("product", [<$ty>::HASH]), |iter: Value| {
                        let mut product = match iter.protocol_next()? {
                            Some(init) => <$ty>::from_value(init)?,
                            None => <$ty>::ONE,
                        };

                        while let Some(v) = iter.protocol_next()? {
                            let v = <$ty>::from_value(v)?;

                            let Some(out) = product.checked_mul(v) else {
                                return Err(VmError::new(VmErrorKind::Overflow));
                            };

                            product = out;
                        }

                        Ok(product)
                    })?;
                }

                {
                    cx.function(
                        Params::new("sum", [<$ty>::HASH]),
                        |iter: Value| -> Result<$ty, VmError> {
                            let mut sum = match iter.protocol_next()? {
                                Some(init) => <$ty>::from_value(init)?,
                                None => <$ty>::ZERO,
                            };

                            while let Some(v) = iter.protocol_next()? {
                                let v = <$ty>::from_value(v)?;

                                let Some(out) = sum.checked_add(v) else {
                                    return Err(VmError::new(VmErrorKind::Overflow));
                                };

                                sum = out;
                            }

                            Ok(sum)
                        },
                    )?;
                }};
            }

            ops!(u64);
            ops!(i64);
            ops!(f64);
            Ok(())
        })?;

        t.function("next")?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<Option<Value>>()?
            .docs(docstring! {
                /// Advances the iterator and returns the next value.
                ///
                /// Returns [`None`] when iteration is finished. Individual iterator
                /// implementations may choose to resume iteration, and so calling `next()`
                /// again may or may not eventually start returning [`Some(Item)`] again at some
                /// point.
                ///
                /// [`Some(Item)`]: Some
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter();
                ///
                /// // A call to next() returns the next value...
                /// assert_eq!(Some(1), iter.next());
                /// assert_eq!(Some(2), iter.next());
                /// assert_eq!(Some(3), iter.next());
                ///
                /// // ... and then None once it's over.
                /// assert_eq!(None, iter.next());
                ///
                /// // More calls may or may not return `None`. Here, they always will.
                /// assert_eq!(None, iter.next());
                /// assert_eq!(None, iter.next());
                /// ```
            })?;

        t.function("nth")?
            .argument_types::<(Value, usize)>()?
            .argument_names(["self", "n"])?
            .return_type::<Option<Value>>()?
            .docs(docstring! {
                /// Returns the `n`th element of the iterator.
                ///
                /// Like most indexing operations, the count starts from zero, so `nth(0)`
                /// returns the first value, `nth(1)` the second, and so on.
                ///
                /// Note that all preceding elements, as well as the returned element, will be
                /// consumed from the iterator. That means that the preceding elements will be
                /// discarded, and also that calling `nth(0)` multiple times on the same iterator
                /// will return different elements.
                ///
                /// `nth()` will return [`None`] if `n` is greater than or equal to the length of the
                /// iterator.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                /// assert_eq!(a.iter().nth(1), Some(2));
                /// ```
                ///
                /// Calling `nth()` multiple times doesn't rewind the iterator:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter();
                ///
                /// assert_eq!(iter.nth(1), Some(2));
                /// assert_eq!(iter.nth(1), None);
                /// ```
                ///
                /// Returning `None` if there are less than `n + 1` elements:
                ///
                /// ```
                /// let a = [1, 2, 3];
                /// assert_eq!(a.iter().nth(10), None);
                /// ```
            })?;

        t.function("size_hint")?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<(usize, Option<usize>)>()?
            .docs(docstring! {
                /// Returns the bounds on the remaining length of the iterator.
                ///
                /// Specifically, `size_hint()` returns a tuple where the first element
                /// is the lower bound, and the second element is the upper bound.
                ///
                /// The second half of the tuple that is returned is an
                /// <code>[Option]<[i64]></code>. A [`None`] here means that either there is no
                /// known upper bound, or the upper bound is larger than [`i64`].
                ///
                /// # Implementation notes
                ///
                /// It is not enforced that an iterator implementation yields the declared
                /// number of elements. A buggy iterator may yield less than the lower bound or
                /// more than the upper bound of elements.
                ///
                /// `size_hint()` is primarily intended to be used for optimizations such as
                /// reserving space for the elements of the iterator, but must not be trusted to
                /// e.g., omit bounds checks in unsafe code. An incorrect implementation of
                /// `size_hint()` should not lead to memory safety violations.
                ///
                /// That said, the implementation should provide a correct estimation, because
                /// otherwise it would be a violation of the trait's protocol.
                ///
                /// The default implementation returns <code>(0, [None])</code> which is correct
                /// for any iterator.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                /// let iter = a.iter();
                ///
                /// assert_eq!(iter.size_hint(), (3u64, Some(3)));
                /// let _ = iter.next();
                /// assert_eq!(iter.size_hint(), (2u64, Some(2)));
                /// ```
                ///
                /// A more complex example:
                ///
                /// ```rune
                /// // The even numbers in the range of zero to nine.
                /// let iter = (0..10).iter().filter(|x| x % 2 == 0);
                ///
                /// // We might iterate from zero to ten times. Knowing that it's five
                /// // exactly wouldn't be possible without executing filter().
                /// assert_eq!(iter.size_hint(), (0, Some(10)));
                ///
                /// // Let's add five more numbers with chain()
                /// let iter = (0..10).iter().filter(|x| x % 2 == 0).chain(15..20);
                ///
                /// // now both bounds are increased by five
                /// assert_eq!(iter.size_hint(), (5, Some(15)));
                /// ```
                ///
                /// Returning `None` for an upper bound:
                ///
                /// ```rune
                /// // an infinite iterator has no upper bound
                /// // and the maximum possible lower bound
                /// let iter = (0..).iter();
                ///
                /// assert_eq!(iter.size_hint(), (u64::MAX, None));
                /// ```
            })?;

        t.function("count")?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<usize>()?
            .docs(docstring! {
                /// Consumes the iterator, counting the number of iterations and returning it.
                ///
                /// This method will call [`next`] repeatedly until [`None`] is encountered,
                /// returning the number of times it saw [`Some`]. Note that [`next`] has to be
                /// called at least once even if the iterator does not have any elements.
                ///
                /// [`next`]: Iterator::next
                ///
                /// # Overflow Behavior
                ///
                /// The method does no guarding against overflows, so counting elements of an
                /// iterator with more than [`i64::MAX`] elements panics.
                ///
                /// # Panics
                ///
                /// This function might panic if the iterator has more than [`i64::MAX`]
                /// elements.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                /// assert_eq!(a.iter().count(), 3);
                ///
                /// let a = [1, 2, 3, 4, 5];
                /// assert_eq!(a.iter().count(), 5);
                /// ```
            })?;

        t.function("fold")?
            .argument_types::<(Value, Value, Function)>()?
            .argument_names(["self", "init", "f"])?
            .return_type::<Value>()?
            .docs(docstring! {
                /// Folds every element into an accumulator by applying an operation, returning
                /// the final result.
                ///
                /// `fold()` takes two arguments: an initial value, and a closure with two
                /// arguments: an 'accumulator', and an element. The closure returns the value
                /// that the accumulator should have for the next iteration.
                ///
                /// The initial value is the value the accumulator will have on the first call.
                ///
                /// After applying this closure to every element of the iterator, `fold()`
                /// returns the accumulator.
                ///
                /// This operation is sometimes called 'reduce' or 'inject'.
                ///
                /// Folding is useful whenever you have a collection of something, and want to
                /// produce a single value from it.
                ///
                /// Note: `fold()`, and similar methods that traverse the entire iterator, might
                /// not terminate for infinite iterators, even on traits for which a result is
                /// determinable in finite time.
                ///
                /// Note: [`reduce()`] can be used to use the first element as the initial
                /// value, if the accumulator type and item type is the same.
                ///
                /// Note: `fold()` combines elements in a *left-associative* fashion. For
                /// associative operators like `+`, the order the elements are combined in is
                /// not important, but for non-associative operators like `-` the order will
                /// affect the final result. For a *right-associative* version of `fold()`, see
                /// [`DoubleEndedIterator::rfold()`].
                ///
                /// # Note to Implementors
                ///
                /// Several of the other (forward) methods have default implementations in
                /// terms of this one, so try to implement this explicitly if it can
                /// do something better than the default `for` loop implementation.
                ///
                /// In particular, try to have this call `fold()` on the internal parts
                /// from which this iterator is composed.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// // the sum of all of the elements of the array
                /// let sum = a.iter().fold(0, |acc, x| acc + x);
                ///
                /// assert_eq!(sum, 6);
                /// ```
                ///
                /// Let's walk through each step of the iteration here:
                ///
                /// | element | acc | x | result |
                /// |---------|-----|---|--------|
                /// |         | 0   |   |        |
                /// | 1       | 0   | 1 | 1      |
                /// | 2       | 1   | 2 | 3      |
                /// | 3       | 3   | 3 | 6      |
                ///
                /// And so, our final result, `6`.
                ///
                /// This example demonstrates the left-associative nature of `fold()`:
                /// it builds a string, starting with an initial value
                /// and continuing with each element from the front until the back:
                ///
                /// ```rune
                /// let numbers = [1, 2, 3, 4, 5];
                ///
                /// let zero = "0";
                ///
                /// let result = numbers.iter().fold(zero, |acc, x| {
                ///     format!("({} + {})", acc, x)
                /// });
                ///
                /// assert_eq!(result, "(((((0 + 1) + 2) + 3) + 4) + 5)");
                /// ```
                ///
                /// It's common for people who haven't used iterators a lot to
                /// use a `for` loop with a list of things to build up a result. Those
                /// can be turned into `fold()`s:
                ///
                /// ```rune
                /// let numbers = [1, 2, 3, 4, 5];
                ///
                /// let result = 0;
                ///
                /// // for loop:
                /// for i in numbers {
                ///     result = result + i;
                /// }
                ///
                /// // fold:
                /// let result2 = numbers.iter().fold(0, |acc, x| acc + x);
                ///
                /// // they're the same
                /// assert_eq!(result, result2);
                /// ```
                ///
                /// [`reduce()`]: Iterator::reduce
            })?;

        t.function("reduce")?
            .argument_types::<(Value, Function)>()?
            .argument_names(["self", "f"])?
            .return_type::<Option<Value>>()?
            .docs(docstring! {
                /// Reduces the elements to a single one, by repeatedly applying a reducing
                /// operation.
                ///
                /// If the iterator is empty, returns [`None`]; otherwise, returns the result of
                /// the reduction.
                ///
                /// The reducing function is a closure with two arguments: an 'accumulator', and
                /// an element. For iterators with at least one element, this is the same as
                /// [`fold()`] with the first element of the iterator as the initial accumulator
                /// value, folding every subsequent element into it.
                ///
                /// [`fold()`]: Iterator::fold
                ///
                /// # Example
                ///
                /// ```rune
                /// let reduced = (1..10).iter().reduce(|acc, e| acc + e).unwrap();
                /// assert_eq!(reduced, 45);
                ///
                /// // Which is equivalent to doing it with `fold`:
                /// let folded = (1..10).iter().fold(0, |acc, e| acc + e);
                /// assert_eq!(reduced, folded);
                /// ```
            })?;

        t.function("find")?
            .argument_types::<(Value, Function)>()?
            .argument_names(["self", "predicate"])?
            .return_type::<Option<Value>>()?
            .docs(docstring! {
                /// Searches for an element of an iterator that satisfies a predicate.
                ///
                /// `find()` takes a closure that returns `true` or `false`. It applies this
                /// closure to each element of the iterator, and if any of them return `true`,
                /// then `find()` returns [`Some(element)`]. If they all return `false`, it
                /// returns [`None`].
                ///
                /// `find()` is short-circuiting; in other words, it will stop processing as
                /// soon as the closure returns `true`.
                ///
                /// If you need the index of the element, see [`position()`].
                ///
                /// [`Some(element)`]: Some
                /// [`position()`]: Iterator::position
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// assert_eq!(a.iter().find(|x| x == 2), Some(2));
                ///
                /// assert_eq!(a.iter().find(|x| x == 5), None);
                /// ```
                ///
                /// Stopping at the first `true`:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter();
                ///
                /// assert_eq!(iter.find(|x| x == 2), Some(2));
                ///
                /// // we can still use `iter`, as there are more elements.
                /// assert_eq!(iter.next(), Some(3));
                /// ```
                ///
                /// Note that `iter.find(f)` is equivalent to `iter.filter(f).next()`.
            })?;

        t.function("any")?
            .argument_types::<(Value, Function)>()?
            .argument_names(["self", "f"])?
            .return_type::<bool>()?
            .docs(docstring! {
                /// Tests if any element of the iterator matches a predicate.
                ///
                /// `any()` takes a closure that returns `true` or `false`. It applies this
                /// closure to each element of the iterator, and if any of them return `true`,
                /// then so does `any()`. If they all return `false`, it returns `false`.
                ///
                /// `any()` is short-circuiting; in other words, it will stop processing as soon
                /// as it finds a `true`, given that no matter what else happens, the result
                /// will also be `true`.
                ///
                /// An empty iterator returns `false`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// assert!(a.iter().any(|x| x > 0));
                ///
                /// assert!(!a.iter().any(|x| x > 5));
                /// ```
                ///
                /// Stopping at the first `true`:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter();
                ///
                /// assert!(iter.any(|x| x != 2));
                ///
                /// // we can still use `iter`, as there are more elements.
                /// assert_eq!(iter.next(), Some(2));
                /// ```
            })?;

        t.function("all")?
            .argument_types::<(Value, Function)>()?
            .argument_names(["self", "f"])?
            .return_type::<bool>()?
            .docs(docstring! {
                /// Tests if every element of the iterator matches a predicate.
                ///
                /// `all()` takes a closure that returns `true` or `false`. It applies this
                /// closure to each element of the iterator, and if they all return `true`, then
                /// so does `all()`. If any of them return `false`, it returns `false`.
                ///
                /// `all()` is short-circuiting; in other words, it will stop processing as soon
                /// as it finds a `false`, given that no matter what else happens, the result
                /// will also be `false`.
                ///
                /// An empty iterator returns `true`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// assert!(a.iter().all(|x| x > 0));
                ///
                /// assert!(!a.iter().all(|x| x > 2));
                /// ```
                ///
                /// Stopping at the first `false`:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter();
                ///
                /// assert!(!iter.all(|x| x != 2));
                ///
                /// // we can still use `iter`, as there are more elements.
                /// assert_eq!(iter.next(), Some(3));
                /// ```
            })?;

        t.function("chain")?
            .argument_types::<(Value, Value)>()?
            .argument_names(["self", "other"])?
            .return_type::<Chain>()?
            .docs(docstring! {
                /// Takes two iterators and creates a new iterator over both in sequence.
                ///
                /// `chain()` will return a new iterator which will first iterate over
                /// values from the first iterator and then over values from the second
                /// iterator.
                ///
                /// In other words, it links two iterators together, in a chain. 🔗
                ///
                /// [`once`] is commonly used to adapt a single value into a chain of other
                /// kinds of iteration.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a1 = [1, 2, 3];
                /// let a2 = [4, 5, 6];
                ///
                /// let iter = a1.iter().chain(a2.iter());
                ///
                /// assert_eq!(iter.next(), Some(1));
                /// assert_eq!(iter.next(), Some(2));
                /// assert_eq!(iter.next(), Some(3));
                /// assert_eq!(iter.next(), Some(4));
                /// assert_eq!(iter.next(), Some(5));
                /// assert_eq!(iter.next(), Some(6));
                /// assert_eq!(iter.next(), None);
                /// ```
                ///
                /// Since the argument to `chain()` uses [`INTO_ITER`], we can pass anything
                /// that can be converted into an [`Iterator`], not just an [`Iterator`] itself.
                /// For example, slices (`[T]`) implement [`INTO_ITER`], and so can be passed to
                /// `chain()` directly:
                ///
                /// ```rune
                /// let s1 = [1, 2, 3];
                /// let s2 = [4, 5, 6];
                ///
                /// let iter = s1.iter().chain(s2);
                ///
                /// assert_eq!(iter.next(), Some(1));
                /// assert_eq!(iter.next(), Some(2));
                /// assert_eq!(iter.next(), Some(3));
                /// assert_eq!(iter.next(), Some(4));
                /// assert_eq!(iter.next(), Some(5));
                /// assert_eq!(iter.next(), Some(6));
                /// assert_eq!(iter.next(), None);
                /// ```
                ///
                /// [`INTO_ITER`]: protocol@INTO_ITER
            })?;

        t.function("enumerate")?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<Enumerate>()?
            .docs(docstring! {
                /// Creates an iterator which gives the current iteration count as well as
                /// the next value.
                ///
                /// The iterator returned yields pairs `(i, val)`, where `i` is the current
                /// index of iteration and `val` is the value returned by the iterator.
                ///
                /// `enumerate()` keeps its count as a usize. If you want to count by a
                /// different sized integer, the zip function provides similar
                /// functionality.
                ///
                /// # Examples
                ///
                /// ```rune
                /// let a = ['a', 'b', 'c'];
                ///
                /// let iter = a.iter().enumerate();
                ///
                /// assert_eq!(iter.next(), Some((0u64, 'a')));
                /// assert_eq!(iter.next(), Some((1u64, 'b')));
                /// assert_eq!(iter.next(), Some((2u64, 'c')));
                /// assert_eq!(iter.next(), None);
                /// ```
            })?;

        t.function("filter")?
            .argument_types::<(Value, Function)>()?
            .argument_names(["self", "filter"])?
            .return_type::<Filter>()?
            .docs(docstring! {
                /// Creates an iterator which uses a closure to determine if an element
                /// should be yielded.
                ///
                /// Given an element the closure must return `true` or `false`. The returned
                /// iterator will yield only the elements for which the closure returns
                /// `true`.
                ///
                /// ```rune
                /// let a = [0, 1, 2];
                ///
                /// let iter = a.iter().filter(|x| x.is_positive());
                ///
                /// assert_eq!(iter.next(), Some(1));
                /// assert_eq!(iter.next(), Some(2));
                /// assert_eq!(iter.next(), None);
                /// ```
            })?;

        t.function("map")?
            .argument_types::<(Value, Function)>()?
            .argument_names(["self", "f"])?
            .return_type::<Map>()?
            .docs(docstring! {
                /// Takes a closure and creates an iterator which calls that closure on each
                /// element.
                ///
                /// `map()` transforms one iterator into another. It produces a new iterator
                /// which calls this closure on each element of the original iterator.
                ///
                /// If you are good at thinking in types, you can think of `map()` like
                /// this: If you have an iterator that gives you elements of some type `A`,
                /// and you want an iterator of some other type `B`, you can use `map()`,
                /// passing a closure that takes an `A` and returns a `B`.
                ///
                /// `map()` is conceptually similar to a `for` loop. However, as `map()` is
                /// lazy, it is best used when you're already working with other iterators.
                /// If you're doing some sort of looping for a side effect, it's considered
                /// more idiomatic to use `for` than `map()`.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter().map(|x| 2 * x);
                ///
                /// assert_eq!(iter.next(), Some(2));
                /// assert_eq!(iter.next(), Some(4));
                /// assert_eq!(iter.next(), Some(6));
                /// assert_eq!(iter.next(), None);
                /// ```
                ///
                /// If you're doing some sort of side effect, prefer `for` to `map()`:
                ///
                /// ```rune
                /// // don't do this:
                /// (0..5).iter().map(|x| println!("{}", x));
                ///
                /// // it won't even execute, as it is lazy. Rust will warn you about this.
                ///
                /// // Instead, use for:
                /// for x in 0..5 {
                ///     println!("{}", x);
                /// }
                /// ```
            })?;

        t.function("filter_map")?
            .argument_types::<(Value, Function)>()?
            .argument_names(["self", "f"])?
            .return_type::<FilterMap>()?
            .docs(docstring! {
                /// Creates an iterator that both filters and maps.
                ///
                /// The returned iterator yields only the `value`s for which the supplied
                /// closure returns `Some(value)`.
                ///
                /// `filter_map` can be used to make chains of [`filter`] and [`map`] more
                /// concise. The example below shows how a `map().filter().map()` can be
                /// shortened to a single call to `filter_map`.
                ///
                /// [`filter`]: Iterator::filter
                /// [`map`]: Iterator::map
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = ["1", "two", "NaN", "four", "5"];
                ///
                /// let iter = a.iter().filter_map(|s| s.parse::<i64>().ok());
                ///
                /// assert_eq!(iter.next(), Some(1));
                /// assert_eq!(iter.next(), Some(5));
                /// assert_eq!(iter.next(), None);
                /// ```
                ///
                /// Here's the same example, but with [`filter`] and [`map`]:
                ///
                /// ```rune
                /// let a = ["1", "two", "NaN", "four", "5"];
                /// let iter = a.iter().map(|s| s.parse::<i64>()).filter(|s| s.is_ok()).map(|s| s.unwrap());
                /// assert_eq!(iter.next(), Some(1));
                /// assert_eq!(iter.next(), Some(5));
                /// assert_eq!(iter.next(), None);
                /// ```
            })?;

        t.function("flat_map")?
            .argument_types::<(Value, Function)>()?
            .argument_names(["self", "f"])?
            .return_type::<FlatMap>()?
            .docs(docstring! {
                /// Creates an iterator that works like map, but flattens nested
                /// structure.
                ///
                /// The [`map`] adapter is very useful, but only when the
                /// closure argument produces values. If it produces an iterator
                /// instead, there's an extra layer of indirection. `flat_map()`
                /// will remove this extra layer on its own.
                ///
                /// You can think of `flat_map(f)` as the semantic equivalent of
                /// [`map`]ping, and then [`flatten`]ing as in
                /// `map(f).flatten()`.
                ///
                /// Another way of thinking about `flat_map()`: [`map`]'s
                /// closure returns one item for each element, and
                /// `flat_map()`'s closure returns an iterator for each element.
                ///
                /// [`map`]: Iterator::map
                /// [`flatten`]: Iterator::flatten
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let words = ["alpha", "beta", "gamma"];
                ///
                /// // chars() returns an iterator
                /// let merged = words.iter().flat_map(|s| s.chars()).collect::<String>();
                /// assert_eq!(merged, "alphabetagamma");
                /// ```
            })?;

        t.function("peekable")?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<Peekable>()?
            .docs(docstring! {
                /// Creates an iterator which can use the [`peek`] method to
                /// look at the next element of the iterator without consuming
                /// it. See their documentation for more information.
                ///
                /// Note that the underlying iterator is still advanced when
                /// [`peek`] are called for the first time: In order to retrieve
                /// the next element, [`next`] is called on the underlying
                /// iterator, hence any side effects (i.e. anything other than
                /// fetching the next value) of the [`next`] method will occur.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let xs = [1, 2, 3];
                ///
                /// let iter = xs.iter().peekable();
                ///
                /// // peek() lets us see into the future
                /// assert_eq!(iter.peek(), Some(1));
                /// assert_eq!(iter.next(), Some(1));
                ///
                /// assert_eq!(iter.next(), Some(2));
                ///
                /// // we can peek() multiple times, the iterator won't advance
                /// assert_eq!(iter.peek(), Some(3));
                /// assert_eq!(iter.peek(), Some(3));
                ///
                /// assert_eq!(iter.next(), Some(3));
                ///
                /// // after the iterator is finished, so is peek()
                /// assert_eq!(iter.peek(), None);
                /// assert_eq!(iter.next(), None);
                /// ```
                ///
                /// [`peek`]: Peekable::peek
                /// [`next`]: Iterator::next
            })?;

        t.function("skip")?
            .argument_types::<(Value, usize)>()?
            .argument_names(["self", "n"])?
            .return_type::<Skip>()?
            .docs(docstring! {
                /// Creates an iterator that skips the first `n` elements.
                ///
                /// `skip(n)` skips elements until `n` elements are skipped or
                /// the end of the iterator is reached (whichever happens
                /// first). After that, all the remaining elements are yielded.
                /// In particular, if the original iterator is too short, then
                /// the returned iterator is empty.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter().skip(2);
                ///
                /// assert_eq!(iter.next(), Some(3));
                /// assert_eq!(iter.next(), None);
                /// ```
            })?;

        t.function("take")?
            .argument_types::<(Value, usize)>()?
            .argument_names(["self", "n"])?
            .return_type::<Take>()?
            .docs(docstring! {
                /// Creates an iterator that yields the first `n` elements, or
                /// fewer if the underlying iterator ends sooner.
                ///
                /// `take(n)` yields elements until `n` elements are yielded or
                /// the end of the iterator is reached (whichever happens
                /// first). The returned iterator is a prefix of length `n` if
                /// the original iterator contains at least `n` elements,
                /// otherwise it contains all of the (fewer than `n`) elements
                /// of the original iterator.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter().take(2);
                ///
                /// assert_eq!(iter.next(), Some(1));
                /// assert_eq!(iter.next(), Some(2));
                /// assert_eq!(iter.next(), None);
                /// ```
                ///
                /// `take()` is often used with an infinite iterator, to make it
                /// finite:
                ///
                /// ```rune
                /// let iter = (0..).iter().take(3);
                ///
                /// assert_eq!(iter.next(), Some(0));
                /// assert_eq!(iter.next(), Some(1));
                /// assert_eq!(iter.next(), Some(2));
                /// assert_eq!(iter.next(), None);
                /// ```
                ///
                /// If less than `n` elements are available, `take` will limit
                /// itself to the size of the underlying iterator:
                ///
                /// ```rune
                /// let v = [1, 2];
                /// let iter = v.iter().take(5);
                /// assert_eq!(iter.next(), Some(1));
                /// assert_eq!(iter.next(), Some(2));
                /// assert_eq!(iter.next(), None);
                /// ```
            })?;

        macro_rules! sum_ops {
            ($ty:ty) => {
                t.function(Params::new("sum", [<$ty>::HASH]))?
                    .argument_types::<(Value,)>()?
                    .argument_names(["self"])?
                    .return_type::<$ty>()?
                    .docs(docstring! {
                        /// Sums the elements of an iterator.
                        ///
                        /// Takes each element, adds them together, and returns
                        /// the result.
                        ///
                        /// An empty iterator returns the zero value of the
                        /// type.
                        ///
                        /// `sum()` can be used to sum numerical built-in types,
                        /// such as `i64`, `float` and `u64`. The first element
                        /// returned by the iterator determines the type being
                        /// summed.
                        ///
                        /// # Panics
                        ///
                        /// When calling `sum()` and a primitive integer type is
                        /// being returned, this method will panic if the
                        /// computation overflows.
                        ///
                        /// # Examples
                        ///
                        /// Basic usage:
                        ///
                        /// ```rune
                        #[doc = concat!(" let a = [1", stringify!($ty), ", 2", stringify!($ty), ", 3", stringify!($ty), "];")]
                        #[doc = concat!(" let sum = a.iter().sum::<", stringify!($ty), ">();")]
                        ///
                        #[doc = concat!(" assert_eq!(sum, 6", stringify!($ty), ");")]
                        /// ```
                    })?;
            };
        }

        sum_ops!(u64);
        sum_ops!(i64);
        sum_ops!(f64);

        macro_rules! integer_product_ops {
            ($ty:ty) => {
                t.function(Params::new("product", [<$ty>::HASH]))?
                    .argument_types::<(Value,)>()?
                    .argument_names(["self"])?
                    .return_type::<$ty>()?
                    .docs(docstring! {
                        /// Iterates over the entire iterator, multiplying all
                        /// the elements
                        ///
                        /// An empty iterator returns the one value of the type.
                        ///
                        /// `sum()` can be used to sum numerical built-in types,
                        /// such as `i64`, `f64` and `u64`. The first element
                        /// returned by the iterator determines the type being
                        /// multiplied.
                        ///
                        /// # Panics
                        ///
                        /// When calling `product()` and a primitive integer
                        /// type is being returned, method will panic if the
                        /// computation overflows.
                        ///
                        /// # Examples
                        ///
                        /// ```rune
                        /// fn factorial(n) {
                        #[doc = concat!("     (1", stringify!($ty), "..=n).iter().product::<", stringify!($ty), ">()")]
                        /// }
                        ///
                        #[doc = concat!(" assert_eq!(factorial(0", stringify!($ty), "), 1", stringify!($ty), ");")]
                        #[doc = concat!(" assert_eq!(factorial(1", stringify!($ty), "), 1", stringify!($ty), ");")]
                        #[doc = concat!(" assert_eq!(factorial(5", stringify!($ty), "), 120", stringify!($ty), ");")]
                        /// ```
                    })?;
            };
        }

        t.function(Params::new("collect", [Vec::HASH]))?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<Vec>()?
            .docs(docstring! {
                /// Collect the iterator as a [`Vec`].
                ///
                /// # Examples
                ///
                /// ```rune
                /// use std::iter::range;
                ///
                /// assert_eq!((0..3).iter().collect::<Vec>(), [0, 1, 2]);
                /// ```
            })?;

        t.function(Params::new("collect", [VecDeque::HASH]))?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<VecDeque>()?
            .docs(docstring! {
                /// Collect the iterator as a [`VecDeque`].
                ///
                /// # Examples
                ///
                /// ```rune
                /// use std::collections::VecDeque;
                ///
                /// assert_eq!((0..3).iter().collect::<VecDeque>(), VecDeque::from::<Vec>([0, 1, 2]));
                /// ```
            })?;

        t.function(Params::new("collect", [HashSet::HASH]))?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<HashSet>()?
            .docs(docstring! {
                /// Collect the iterator as a [`HashSet`].
                ///
                /// # Examples
                ///
                /// ```rune
                /// use std::collections::HashSet;
                ///
                /// let a = (0..3).iter().collect::<HashSet>();
                /// let b = HashSet::from_iter([0, 1, 2]);
                ///
                /// assert_eq!(a, b);
                /// ```
            })?;

        t.function(Params::new("collect", [HashMap::HASH]))?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<HashMap>()?
            .docs(docstring! {
                /// Collect the iterator as a [`HashMap`].
                ///
                /// # Examples
                ///
                /// ```rune
                /// use std::collections::HashMap;
                ///
                /// let actual = (0..3).iter().map(|n| (n, n.to_string())).collect::<HashMap>();
                ///
                /// let expected = HashMap::from_iter([
                ///     (0, "0"),
                ///     (1, "1"),
                ///     (2, "2"),
                /// ]);
                ///
                /// assert_eq!(actual, expected);
                /// ```
            })?;

        t.function(Params::new("collect", [Object::HASH]))?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<HashMap>()?
            .docs(docstring! {
                /// Collect the iterator as an [`Object`].
                ///
                /// # Examples
                ///
                /// ```rune
                /// assert_eq!([("first", 1), ("second", 2)].iter().collect::<Object>(), #{first: 1, second: 2});
                /// ```
            })?;

        t.function(Params::new("collect", [OwnedTuple::HASH]))?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<OwnedTuple>()?
            .docs(docstring! {
                /// Collect the iterator as a [`Tuple`].
                ///
                /// # Examples
                ///
                /// ```rune
                /// assert_eq!((0..3).iter().collect::<Tuple>(), (0, 1, 2));
                /// ```
            })?;

        t.function(Params::new("collect", [String::HASH]))?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<String>()?
            .docs(docstring! {
                /// Collect the iterator as a [`String`].
                ///
                /// # Examples
                ///
                /// ```rune
                /// assert_eq!(["first", "second"].iter().collect::<String>(), "firstsecond");
                /// ```
            })?;

        macro_rules! float_product_ops {
            ($ty:ty) => {
                t.function(Params::new("product", [<$ty>::HASH]))?
                    .argument_types::<(Value,)>()?
                    .argument_names(["self"])?
                    .return_type::<$ty>()?
                    .docs(docstring! {
                        /// Iterates over the entire iterator, multiplying all
                        /// the elements
                        ///
                        /// An empty iterator returns the one value of the type.
                        ///
                        /// `sum()` can be used to sum numerical built-in types,
                        /// such as `i64`, `f64` and `u64`. The first element
                        /// returned by the iterator determines the type being
                        /// multiplied.
                        ///
                        /// # Panics
                        ///
                        /// When calling `product()` and a primitive integer
                        /// type is being returned, method will panic if the
                        /// computation overflows.
                        ///
                        /// # Examples
                        ///
                        /// ```rune
                        /// fn factorial(n) {
                        #[doc = concat!("     (1..=n).iter().map(|n| n as ", stringify!($ty), ").product::<", stringify!($ty), ">()")]
                        /// }
                        ///
                        #[doc = concat!(" assert_eq!(factorial(0), 1", stringify!($ty), ");")]
                        #[doc = concat!(" assert_eq!(factorial(1), 1", stringify!($ty), ");")]
                        #[doc = concat!(" assert_eq!(factorial(5), 120", stringify!($ty), ");")]
                        /// ```
                    })?;
            };
        }

        integer_product_ops!(u64);
        integer_product_ops!(i64);
        float_product_ops!(f64);
    }

    {
        let mut t = m.define_trait(["DoubleEndedIterator"])?;

        t.docs(docstring! {
            /// An iterator able to yield elements from both ends.
            ///
            /// Something that implements `DoubleEndedIterator` has one extra
            /// capability over something that implements [`Iterator`]: the
            /// ability to also take `Item`s from the back, as well as the
            /// front.
            ///
            /// It is important to note that both back and forth work on the
            /// same range, and do not cross: iteration is over when they meet
            /// in the middle.
            ///
            /// In a similar fashion to the [`Iterator`] protocol, once a
            /// `DoubleEndedIterator` returns [`None`] from a [`next_back()`],
            /// calling it again may or may not ever return [`Some`] again.
            /// [`next()`] and [`next_back()`] are interchangeable for this
            /// purpose.
            ///
            /// [`next_back()`]: DoubleEndedIterator::next_back
            /// [`next()`]: Iterator::next
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```
            /// let numbers = [1, 2, 3, 4, 5, 6];
            ///
            /// let iter = numbers.iter();
            ///
            /// assert_eq!(Some(1), iter.next());
            /// assert_eq!(Some(6), iter.next_back());
            /// assert_eq!(Some(5), iter.next_back());
            /// assert_eq!(Some(2), iter.next());
            /// assert_eq!(Some(3), iter.next());
            /// assert_eq!(Some(4), iter.next());
            /// assert_eq!(None, iter.next());
            /// assert_eq!(None, iter.next_back());
            /// ```
        })?;

        t.handler(|cx| {
            let next_back = cx.find(&Protocol::NEXT_BACK)?;

            cx.find_or_define(&Protocol::NTH_BACK, {
                let next_back = next_back.clone();

                move |iterator: Value, mut n: usize| -> Result<Option<Value>, VmError> {
                    loop {
                        let mut memory = [iterator.clone()];
                        next_back.call(&mut memory, Address::ZERO, 1, Output::keep(0))?;
                        let [value] = memory;

                        let Some(value) = Option::<Value>::from_value(value)? else {
                            break Ok(None);
                        };

                        if n == 0 {
                            break Ok(Some(value));
                        }

                        n -= 1;
                    }
                }
            })?;

            cx.raw_function("rev", |stack, addr, len, out| {
                let [value] = stack.slice_at(addr, len)? else {
                    return Err(VmError::new(VmErrorKind::BadArgumentCount {
                        actual: len,
                        expected: 1,
                    }));
                };

                let rev = Rev {
                    value: value.clone(),
                };

                stack.store(out, || rune::to_value(rev))?;
                Ok(())
            })?;

            Ok(())
        })?;

        t.function("next_back")?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<Option<Value>>()?
            .docs(docstring! {
                /// Removes and returns an element from the end of the iterator.
                ///
                /// Returns `None` when there are no more elements.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let numbers = [1, 2, 3, 4, 5, 6];
                ///
                /// let iter = numbers.iter();
                ///
                /// assert_eq!(Some(1), iter.next());
                /// assert_eq!(Some(6), iter.next_back());
                /// assert_eq!(Some(5), iter.next_back());
                /// assert_eq!(Some(2), iter.next());
                /// assert_eq!(Some(3), iter.next());
                /// assert_eq!(Some(4), iter.next());
                /// assert_eq!(None, iter.next());
                /// assert_eq!(None, iter.next_back());
                /// ```
            })?;

        t.function("nth_back")?
            .argument_types::<(Value, usize)>()?
            .argument_names(["self", "n"])?
            .return_type::<Option<Value>>()?
            .docs(docstring! {
                /// Returns the `n`th element from the end of the iterator.
                ///
                /// This is essentially the reversed version of
                /// [`Iterator::nth()`]. Although like most indexing operations,
                /// the count starts from zero, so `nth_back(0)` returns the
                /// first value from the end, `nth_back(1)` the second, and so
                /// on.
                ///
                /// Note that all elements between the end and the returned
                /// element will be consumed, including the returned element.
                /// This also means that calling `nth_back(0)` multiple times on
                /// the same iterator will return different elements.
                ///
                /// `nth_back()` will return [`None`] if `n` is greater than or
                /// equal to the length of the iterator.
                ///
                /// # Examples
                ///
                /// Basic usage:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                /// assert_eq!(a.iter().nth_back(2), Some(1));
                /// ```
                ///
                /// Calling `nth_back()` multiple times doesn't rewind the
                /// iterator:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter();
                ///
                /// assert_eq!(iter.nth_back(1), Some(2));
                /// assert_eq!(iter.nth_back(1), None);
                /// ```
                ///
                /// Returning `None` if there are less than `n + 1` elements:
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                /// assert_eq!(a.iter().nth_back(10), None);
                /// ```
            })?;

        t.function("rev")?
            .argument_types::<(Value,)>()?
            .argument_names(["self"])?
            .return_type::<Rev>()?
            .docs(docstring! {
                /// Reverses an iterator's direction.
                ///
                /// Usually, iterators iterate from left to right. After using `rev()`, an
                /// iterator will instead iterate from right to left.
                ///
                /// This is only possible if the iterator has an end, so `rev()` only works on
                /// double-ended iterators.
                ///
                /// # Examples
                ///
                /// ```rune
                /// let a = [1, 2, 3];
                ///
                /// let iter = a.iter().rev();
                ///
                /// assert_eq!(iter.next(), Some(3));
                /// assert_eq!(iter.next(), Some(2));
                /// assert_eq!(iter.next(), Some(1));
                ///
                /// assert_eq!(iter.next(), None);
                /// ```
            })?;
    }

    m.function_meta(range)?;

    m.ty::<Empty>()?;
    m.function_meta(Empty::next__meta)?;
    m.function_meta(Empty::next_back__meta)?;
    m.function_meta(Empty::size_hint__meta)?;
    m.implement_trait::<Empty>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Empty>(rune::item!(::std::iter::DoubleEndedIterator))?;
    m.function_meta(empty)?;

    m.ty::<Once>()?;
    m.function_meta(Once::next__meta)?;
    m.function_meta(Once::next_back__meta)?;
    m.function_meta(Once::size_hint__meta)?;
    m.implement_trait::<Once>(rune::item!(::std::iter::Iterator))?;
    m.implement_trait::<Once>(rune::item!(::std::iter::DoubleEndedIterator))?;
    m.function_meta(once)?;
    Ok(m)
}

/// Construct an iterator which produces no values.
///
/// # Examples
///
/// ```rune
/// use std::iter::empty;
///
/// assert!(empty().next().is_none());
/// assert_eq!(empty().collect::<Vec>(), []);
/// ```
#[rune::function]
fn empty() -> Empty {
    Empty
}

#[derive(Any)]
#[rune(item = ::std::iter)]
struct Empty;

impl Empty {
    #[rune::function(keep, protocol = NEXT)]
    fn next(&mut self) -> Option<Value> {
        None
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    fn next_back(&mut self) -> Option<Value> {
        None
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}

/// Construct an iterator which produces a single `value` once.
///
/// # Examples
///
/// ```rune
/// use std::iter::once;
///
/// assert!(once(42).next().is_some());
/// assert_eq!(once(42).collect::<Vec>(), [42]);
/// ```
#[rune::function]
fn once(value: Value) -> Once {
    Once { value: Some(value) }
}

#[derive(Any)]
#[rune(item = ::std::iter)]
struct Once {
    value: Option<Value>,
}

impl Once {
    #[rune::function(keep, protocol = NEXT)]
    fn next(&mut self) -> Option<Value> {
        self.value.take()
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    fn next_back(&mut self) -> Option<Value> {
        self.value.take()
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = usize::from(self.value.is_some());
        (len, Some(len))
    }
}

/// Produce an iterator which starts at the range `start` and ends at the value
/// `end` (exclusive).
///
/// # Examples
///
/// ```rune
/// use std::iter::range;
///
/// assert!(range(0, 3).next().is_some());
/// assert_eq!(range(0, 3).collect::<Vec>(), [0, 1, 2]);
/// ```
#[rune::function(deprecated = "Use the `<from>..<to>` operator instead")]
fn range(start: i64, end: i64) -> RangeIter<i64> {
    RangeIter::new(start..end)
}

/// Fuse the iterator if the expression is `None`.
macro_rules! fuse {
    ($self:ident . $iter:ident . $($call:tt)+) => {
        match $self.$iter {
            Some(ref mut iter) => match iter.$($call)+? {
                None => {
                    $self.$iter = None;
                    None
                }
                item => item,
            },
            None => None,
        }
    };
}

/// Try an iterator method without fusing,
/// like an inline `.as_mut().and_then(...)`
macro_rules! maybe {
    ($self:ident . $iter:ident . $($call:tt)+) => {
        match $self.$iter {
            Some(ref mut iter) => iter.$($call)+?,
            None => None,
        }
    };
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct Chain {
    a: Option<Value>,
    b: Option<Value>,
}

impl Chain {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<Value>, VmError> {
        Ok(match fuse!(self.a.protocol_next()) {
            None => maybe!(self.b.protocol_next()),
            item => item,
        })
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<Value>, VmError> {
        Ok(match fuse!(self.b.protocol_next_back()) {
            None => maybe!(self.a.protocol_next_back()),
            item => item,
        })
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    #[inline]
    fn size_hint(&self) -> Result<(usize, Option<usize>), VmError> {
        match self {
            Self {
                a: Some(a),
                b: Some(b),
            } => {
                let (a_lower, a_upper) = a.protocol_size_hint()?;
                let (b_lower, b_upper) = b.protocol_size_hint()?;

                let lower = a_lower.saturating_add(b_lower);

                let upper = match (a_upper, b_upper) {
                    (Some(x), Some(y)) => x.checked_add(y),
                    _ => None,
                };

                Ok((lower, upper))
            }
            Self {
                a: Some(a),
                b: None,
            } => a.protocol_size_hint(),
            Self {
                a: None,
                b: Some(b),
            } => b.protocol_size_hint(),
            Self { a: None, b: None } => Ok((0, Some(0))),
        }
    }

    #[rune::function(keep, protocol = LEN)]
    #[inline]
    fn len(&self) -> Result<usize, VmError> {
        match self {
            Self {
                a: Some(a),
                b: Some(b),
            } => {
                let a_len = a.protocol_len()?;
                let b_len = b.protocol_len()?;
                Ok(a_len.saturating_add(b_len))
            }
            Self {
                a: Some(a),
                b: None,
            } => a.protocol_len(),
            Self {
                a: None,
                b: Some(b),
            } => b.protocol_len(),
            Self { a: None, b: None } => Ok(0),
        }
    }
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct Enumerate {
    iter: Value,
    count: usize,
}

impl Enumerate {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<(usize, Value)>, VmError> {
        if let Some(value) = self.iter.protocol_next()? {
            let i = self.count;
            self.count += 1;
            return Ok(Some((i, value)));
        }

        Ok(None)
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<(usize, Value)>, VmError> {
        if let Some(value) = self.iter.protocol_next_back()? {
            let len = self.iter.protocol_len()?;
            return Ok(Some((self.count + len, value)));
        }

        Ok(None)
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    #[inline]
    fn size_hint(&self) -> Result<(usize, Option<usize>), VmError> {
        self.iter.protocol_size_hint()
    }

    #[rune::function(keep, protocol = LEN)]
    #[inline]
    fn len(&self) -> Result<usize, VmError> {
        self.iter.protocol_len()
    }
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct Filter {
    iter: Value,
    f: Function,
}

impl Filter {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<Value>, VmError> {
        while let Some(value) = self.iter.protocol_next()? {
            if self.f.call::<bool>((value.clone(),))? {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<Value>, VmError> {
        while let Some(value) = self.iter.protocol_next_back()? {
            if self.f.call::<bool>((value.clone(),))? {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    #[inline]
    fn size_hint(&self) -> Result<(usize, Option<usize>), VmError> {
        let (_, hi) = self.iter.protocol_size_hint()?;
        Ok((0, hi))
    }
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct Map {
    iter: Option<Value>,
    f: Function,
}

impl Map {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<Value>, VmError> {
        if let Some(value) = fuse!(self.iter.protocol_next()) {
            return Ok(Some(self.f.call::<Value>((value.clone(),))?));
        }

        Ok(None)
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<Value>, VmError> {
        if let Some(value) = fuse!(self.iter.protocol_next_back()) {
            return Ok(Some(self.f.call::<Value>((value.clone(),))?));
        }

        Ok(None)
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    #[inline]
    fn size_hint(&self) -> Result<(usize, Option<usize>), VmError> {
        let Some(iter) = &self.iter else {
            return Ok((0, Some(0)));
        };

        iter.protocol_size_hint()
    }

    #[rune::function(keep, protocol = LEN)]
    #[inline]
    fn len(&self) -> Result<usize, VmError> {
        let Some(iter) = &self.iter else {
            return Ok(0);
        };

        iter.protocol_len()
    }
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct FilterMap {
    iter: Option<Value>,
    f: Function,
}

impl FilterMap {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<Value>, VmError> {
        while let Some(value) = fuse!(self.iter.protocol_next()) {
            if let Some(value) = self.f.call::<Option<Value>>((value.clone(),))? {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<Value>, VmError> {
        while let Some(value) = fuse!(self.iter.protocol_next_back()) {
            if let Some(value) = self.f.call::<Option<Value>>((value.clone(),))? {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct FlatMap {
    map: Map,
    frontiter: Option<Value>,
    backiter: Option<Value>,
}

impl FlatMap {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<Value>, VmError> {
        loop {
            if let Some(iter) = &mut self.frontiter {
                match iter.protocol_next()? {
                    None => self.frontiter = None,
                    item @ Some(_) => return Ok(item),
                }
            }

            let Some(value) = self.map.next()? else {
                return Ok(match &mut self.backiter {
                    Some(backiter) => backiter.protocol_next()?,
                    None => None,
                });
            };

            self.frontiter = Some(value.protocol_into_iter()?)
        }
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<Value>, VmError> {
        loop {
            if let Some(ref mut iter) = self.backiter {
                match iter.protocol_next_back()? {
                    None => self.backiter = None,
                    item @ Some(_) => return Ok(item),
                }
            }

            let Some(value) = self.map.next_back()? else {
                return Ok(match &mut self.frontiter {
                    Some(frontiter) => frontiter.protocol_next_back()?,
                    None => None,
                });
            };

            self.backiter = Some(value.protocol_into_iter()?);
        }
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    #[inline]
    fn size_hint(&self) -> Result<(usize, Option<usize>), VmError> {
        let (flo, fhi) = match &self.frontiter {
            Some(iter) => iter.protocol_size_hint()?,
            None => (0, Some(0)),
        };

        let (blo, bhi) = match &self.backiter {
            Some(iter) => iter.protocol_size_hint()?,
            None => (0, Some(0)),
        };

        let lo = flo.saturating_add(blo);

        Ok(match (self.map.size_hint()?, fhi, bhi) {
            ((0, Some(0)), Some(a), Some(b)) => (lo, a.checked_add(b)),
            _ => (lo, None),
        })
    }
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct Peekable {
    iter: Value,
    peeked: Option<Option<Value>>,
}

impl Peekable {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<Value>, VmError> {
        Ok(match self.peeked.take() {
            Some(v) => v,
            None => self.iter.protocol_next()?,
        })
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<Value>, VmError> {
        Ok(match self.peeked.as_mut() {
            Some(v @ Some(_)) => self.iter.protocol_next_back()?.or_else(|| v.take()),
            Some(None) => None,
            None => self.iter.protocol_next_back()?,
        })
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    #[inline]
    fn size_hint(&self) -> Result<(usize, Option<usize>), VmError> {
        let peek_len = match &self.peeked {
            Some(None) => return Ok((0, Some(0))),
            Some(Some(_)) => 1,
            None => 0,
        };

        let (lo, hi) = self.iter.protocol_size_hint()?;
        let lo = lo.saturating_add(peek_len);

        let hi = match hi {
            Some(x) => x.checked_add(peek_len),
            None => None,
        };

        Ok((lo, hi))
    }

    #[rune::function(keep, protocol = LEN)]
    #[inline]
    fn len(&self) -> Result<usize, VmError> {
        let peek_len = match &self.peeked {
            Some(None) => return Ok(0),
            Some(Some(_)) => 1,
            None => 0,
        };

        let len = self.iter.protocol_len()?;
        Ok(len.saturating_add(peek_len))
    }

    /// Returns a reference to the `next()` value without advancing the iterator.
    ///
    /// Like [`next`], if there is a value, it is wrapped in a `Some(T)`. But if the
    /// iteration is over, `None` is returned.
    ///
    /// [`next`]: Iterator::next
    ///
    /// Because `peek()` returns a reference, and many iterators iterate over
    /// references, there can be a possibly confusing situation where the return
    /// value is a double reference. You can see this effect in the examples below.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rune
    /// let xs = [1, 2, 3];
    ///
    /// let iter = xs.iter().peekable();
    ///
    /// // peek() lets us see into the future
    /// assert_eq!(iter.peek(), Some(1));
    /// assert_eq!(iter.next(), Some(1));
    ///
    /// assert_eq!(iter.next(), Some(2));
    ///
    /// // The iterator does not advance even if we `peek` multiple times
    /// assert_eq!(iter.peek(), Some(3));
    /// assert_eq!(iter.peek(), Some(3));
    ///
    /// assert_eq!(iter.next(), Some(3));
    ///
    /// // After the iterator is finished, so is `peek()`
    /// assert_eq!(iter.peek(), None);
    /// assert_eq!(iter.next(), None);
    /// ```
    #[rune::function(keep)]
    #[inline]
    fn peek(&mut self) -> Result<Option<Value>, VmError> {
        if let Some(v) = &self.peeked {
            return Ok(v.clone());
        }

        let value = self.iter.protocol_next()?;
        self.peeked = Some(value.clone());
        Ok(value)
    }
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct Skip {
    iter: Value,
    n: usize,
}

impl Skip {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<Value>, VmError> {
        if self.n > 0 {
            let old_n = self.n;
            self.n = 0;

            for _ in 0..old_n {
                match self.iter.protocol_next()? {
                    Some(..) => (),
                    None => return Ok(None),
                }
            }
        }

        self.iter.protocol_next()
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<Value>, VmError> {
        Ok(if self.len()? > 0 {
            self.iter.protocol_next_back()?
        } else {
            None
        })
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    #[inline]
    fn size_hint(&self) -> Result<(usize, Option<usize>), VmError> {
        let (lower, upper) = self.iter.protocol_size_hint()?;
        let lower = lower.saturating_sub(self.n);
        let upper = upper.map(|x| x.saturating_sub(self.n));
        Ok((lower, upper))
    }

    #[rune::function(keep, protocol = LEN)]
    #[inline]
    fn len(&self) -> Result<usize, VmError> {
        let len = self.iter.protocol_len()?;
        Ok(len.saturating_sub(self.n))
    }
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct Take {
    iter: Value,
    n: usize,
}

impl Take {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<Value>, VmError> {
        if self.n == 0 {
            return Ok(None);
        }

        self.n -= 1;
        self.iter.protocol_next()
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<Value>, VmError> {
        if self.n == 0 {
            Ok(None)
        } else {
            let n = self.n;
            self.n -= 1;
            let len = self.iter.protocol_len()?;
            self.iter.protocol_nth_back(len.saturating_sub(n))
        }
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    #[inline]
    fn size_hint(&self) -> Result<(usize, Option<usize>), VmError> {
        if self.n == 0 {
            return Ok((0, Some(0)));
        }

        let (lower, upper) = self.iter.protocol_size_hint()?;

        let lower = lower.min(self.n);

        let upper = match upper {
            Some(x) if x < self.n => Some(x),
            _ => Some(self.n),
        };

        Ok((lower, upper))
    }

    #[rune::function(keep, protocol = LEN)]
    #[inline]
    fn len(&self) -> Result<usize, VmError> {
        if self.n == 0 {
            return Ok(0);
        }

        let len = self.iter.protocol_len()?;
        Ok(len.min(self.n))
    }
}

#[derive(Any, Debug)]
#[rune(item = ::std::iter)]
struct Rev {
    value: Value,
}

impl Rev {
    #[rune::function(keep, protocol = NEXT)]
    #[inline]
    fn next(&mut self) -> Result<Option<Value>, VmError> {
        self.value.protocol_next_back()
    }

    #[rune::function(keep, protocol = NEXT_BACK)]
    #[inline]
    fn next_back(&mut self) -> Result<Option<Value>, VmError> {
        self.value.protocol_next()
    }

    #[rune::function(keep, protocol = SIZE_HINT)]
    #[inline]
    fn size_hint(&self) -> Result<(usize, Option<usize>), VmError> {
        self.value.protocol_size_hint()
    }

    #[rune::function(keep, protocol = LEN)]
    #[inline]
    fn len(&self) -> Result<usize, VmError> {
        self.value.protocol_len()
    }
}

pub(crate) trait CheckedOps: Sized {
    const ONE: Self;
    const ZERO: Self;

    fn checked_add(self, value: Self) -> Option<Self>;
    fn checked_mul(self, value: Self) -> Option<Self>;
}

impl CheckedOps for i64 {
    const ONE: Self = 1;
    const ZERO: Self = 0;

    #[inline]
    fn checked_add(self, value: Self) -> Option<Self> {
        i64::checked_add(self, value)
    }

    #[inline]
    fn checked_mul(self, value: Self) -> Option<Self> {
        i64::checked_mul(self, value)
    }
}

impl CheckedOps for u64 {
    const ONE: Self = 1;
    const ZERO: Self = 0;

    #[inline]
    fn checked_add(self, value: Self) -> Option<Self> {
        u64::checked_add(self, value)
    }

    #[inline]
    fn checked_mul(self, value: Self) -> Option<Self> {
        u64::checked_mul(self, value)
    }
}

impl CheckedOps for f64 {
    const ONE: Self = 1.0;
    const ZERO: Self = 0.0;

    #[inline]
    fn checked_add(self, value: Self) -> Option<Self> {
        Some(self + value)
    }

    #[inline]
    fn checked_mul(self, value: Self) -> Option<Self> {
        Some(self * value)
    }
}
