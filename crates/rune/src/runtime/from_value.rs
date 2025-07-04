use core::cmp::Ordering;

use crate::alloc::{self, String};
use crate::any::AnyMarker;
use crate::hash::Hash;

use super::{Mut, RawAnyGuard, Ref, RuntimeError, Value};

/// Derive macro for the [`FromValue`] trait for converting types from the
/// dynamic `Value` container.
///
/// This can be implemented for structs and variants.
///
/// For structs, this will try to decode any struct-like data into the desired data type:
///
/// ```rust
/// use rune::{FromValue, Vm};
/// use rune::sync::Arc;
///
/// #[derive(Debug, PartialEq, FromValue)]
/// struct Foo {
///     a: u64,
///     b: u64,
/// }
///
/// let mut sources = rune::sources! {
///     entry => {
///         struct Foo {
///             a,
///             b,
///         }
///
///         pub fn main() {
///             Foo { a: 1, b: 2 }
///         }
///     }
/// };
///
/// let unit = rune::prepare(&mut sources).build()?;
/// let unit = Arc::try_new(unit)?;
///
/// let mut vm = Vm::without_runtime(unit)?;
/// let foo = vm.call(["main"], ())?;
/// let foo: Foo = rune::from_value(foo)?;
///
/// assert_eq!(foo, Foo { a: 1, b: 2 });
/// # Ok::<_, rune::support::Error>(())
/// ```
///
/// For enums, the variant name of the rune-local variant is matched:
///
/// ```rust
/// use rune::{FromValue, Vm};
/// use rune::sync::Arc;
///
/// #[derive(Debug, PartialEq, FromValue)]
/// enum Enum {
///     Variant(u32),
///     Variant2 { a: u32, b: u32 },
/// }
///
/// let mut sources = rune::sources! {
///     entry => {
///         enum Enum {
///             Variant(a),
///         }
///
///         pub fn main() {
///             Enum::Variant(42)
///         }
///     }
/// };
///
/// let unit = rune::prepare(&mut sources).build()?;
/// let unit = Arc::try_new(unit)?;
///
/// let mut vm = Vm::without_runtime(unit)?;
/// let foo = vm.call(["main"], ())?;
/// let foo: Enum = rune::from_value(foo)?;
///
/// assert_eq!(foo, Enum::Variant(42));
/// # Ok::<_, rune::support::Error>(())
/// ```
pub use rune_macros::FromValue;

/// Cheap conversion trait to convert something infallibly into a dynamic [`Value`].
pub trait IntoValue {
    /// Convert into a dynamic [`Value`].
    #[doc(hidden)]
    fn into_value(self) -> Value;
}

impl IntoValue for Value {
    #[inline]
    fn into_value(self) -> Value {
        self
    }
}

impl IntoValue for &Value {
    #[inline]
    fn into_value(self) -> Value {
        self.clone()
    }
}

/// Convert something into the dynamic [`Value`].
///
/// # Examples
///
/// ```
/// use rune::sync::Arc;
/// use rune::{ToValue, Vm};
///
/// #[derive(ToValue)]
/// struct Foo {
///     field: u64,
/// }
///
/// let mut sources = rune::sources! {
///     entry => {
///         pub fn main(foo) {
///             foo.field + 1
///         }
///     }
/// };
///
/// let unit = rune::prepare(&mut sources).build()?;
/// let unit = Arc::try_new(unit)?;
/// let mut vm = Vm::without_runtime(unit)?;
///
/// let foo = vm.call(["main"], (Foo { field: 42 },))?;
/// let foo: u64 = rune::from_value(foo)?;
///
/// assert_eq!(foo, 43);
/// # Ok::<_, rune::support::Error>(())
/// ```
pub fn from_value<T>(value: impl IntoValue) -> Result<T, RuntimeError>
where
    T: FromValue,
{
    T::from_value(value.into_value())
}

/// Trait for converting types from the dynamic [Value] container.
///
/// # Examples
///
/// ```
/// use rune::sync::Arc;
/// use rune::{FromValue, Vm};
///
/// #[derive(FromValue)]
/// struct Foo {
///     field: u64,
/// }
///
/// let mut sources = rune::sources! {
///     entry => {
///         pub fn main() {
///             #{field: 42}
///         }
///     }
/// };
///
/// let unit = rune::prepare(&mut sources).build()?;
/// let unit = Arc::try_new(unit)?;
/// let mut vm = Vm::without_runtime(unit)?;
///
/// let foo = vm.call(["main"], ())?;
/// let foo: Foo = rune::from_value(foo)?;
///
/// assert_eq!(foo.field, 42);
/// # Ok::<_, rune::support::Error>(())
/// ```
#[diagnostic::on_unimplemented(
    message = "FromValue is not implemented for `{Self}`",
    label = "FromValue is not implemented for `{Self}`",
    note = "This probably means that `{Self}` hasn't derived rune::Any"
)]
pub trait FromValue: 'static + Sized {
    /// Try to convert to the given type, from the given value.
    fn from_value(value: Value) -> Result<Self, RuntimeError>;
}

/// Unsafe to mut coercion.
pub trait UnsafeToMut {
    /// The raw guard returned.
    ///
    /// Must only be dropped *after* the value returned from this function is no
    /// longer live.
    type Guard: 'static;

    /// # Safety
    ///
    /// Caller must ensure that the returned reference does not outlive the
    /// guard.
    unsafe fn unsafe_to_mut<'a>(value: Value) -> Result<(&'a mut Self, Self::Guard), RuntimeError>;
}

/// Unsafe to ref coercion.
pub trait UnsafeToRef {
    /// The raw guard returned.
    ///
    /// Must only be dropped *after* the value returned from this function is no
    /// longer live.
    type Guard: 'static;

    /// # Safety
    ///
    /// Caller must ensure that the returned reference does not outlive the
    /// guard.
    unsafe fn unsafe_to_ref<'a>(value: Value) -> Result<(&'a Self, Self::Guard), RuntimeError>;
}

impl<T> FromValue for T
where
    T: AnyMarker,
{
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        value.downcast()
    }
}

impl FromValue for Value {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        Ok(value)
    }
}

// Option impls

impl<T> FromValue for Option<T>
where
    T: FromValue,
{
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        Ok(match value.downcast::<Option<Value>>()? {
            Some(some) => Some(T::from_value(some.clone())?),
            None => None,
        })
    }
}

impl FromValue for rust_alloc::string::String {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        let string = String::from_value(value)?;
        let string = rust_alloc::string::String::from(string);
        Ok(string)
    }
}

impl FromValue for alloc::Box<str> {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        let string = value.borrow_string_ref()?;
        let string = alloc::Box::try_from(string.as_ref())?;
        Ok(string)
    }
}

impl FromValue for rust_alloc::boxed::Box<str> {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        let string = value.borrow_string_ref()?;
        let string = rust_alloc::boxed::Box::<str>::from(string.as_ref());
        Ok(string)
    }
}

impl FromValue for Ref<str> {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        Ok(Ref::map(Ref::<String>::from_value(value)?, String::as_str))
    }
}

impl UnsafeToRef for str {
    type Guard = RawAnyGuard;

    #[inline]
    unsafe fn unsafe_to_ref<'a>(value: Value) -> Result<(&'a Self, Self::Guard), RuntimeError> {
        let string = value.into_ref::<String>()?;
        let (string, guard) = Ref::into_raw(string);
        Ok((string.as_ref().as_str(), guard))
    }
}

impl UnsafeToMut for str {
    type Guard = RawAnyGuard;

    #[inline]
    unsafe fn unsafe_to_mut<'a>(value: Value) -> Result<(&'a mut Self, Self::Guard), RuntimeError> {
        let string = value.into_mut::<String>()?;
        let (mut string, guard) = Mut::into_raw(string);
        Ok((string.as_mut().as_mut_str(), guard))
    }
}

impl<T, E> FromValue for Result<T, E>
where
    T: FromValue,
    E: FromValue,
{
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        Ok(match value.downcast::<Result<Value, Value>>()? {
            Ok(ok) => Result::Ok(T::from_value(ok.clone())?),
            Err(err) => Result::Err(E::from_value(err.clone())?),
        })
    }
}

impl FromValue for bool {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        value.as_bool()
    }
}

impl FromValue for char {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        value.as_char()
    }
}

macro_rules! impl_integer {
    ($($ty:ty),* $(,)?) => {
        $(
            impl FromValue for $ty {
                #[inline]
                fn from_value(value: Value) -> Result<Self, RuntimeError> {
                    value.as_integer()
                }
            }
        )*
    };
}

impl_integer!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

impl FromValue for f64 {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        value.as_float()
    }
}

impl FromValue for f32 {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        Ok(value.as_float()? as f32)
    }
}

cfg_std! {
    macro_rules! impl_map {
        ($ty:ty, $key:ty) => {
            impl<T> FromValue for $ty
            where
                T: FromValue,
            {
                fn from_value(value: Value) -> Result<Self, RuntimeError> {
                    let object = value.downcast::<$crate::runtime::Object>()?;

                    let mut output = <$ty>::with_capacity(object.len());

                    for (key, value) in object {
                        let key = <$key>::try_from(key)?;
                        let value = <T>::from_value(value)?;
                        output.insert(key, value);
                    }

                    Ok(output)
                }
            }
        };
    }

    impl_map!(::std::collections::HashMap<String, T>, String);
    impl_map!(::std::collections::HashMap<rust_alloc::string::String, T>, rust_alloc::string::String);
}

macro_rules! impl_try_map {
    ($ty:ty, $key:ty) => {
        impl<T> FromValue for $ty
        where
            T: FromValue,
        {
            fn from_value(value: Value) -> Result<Self, RuntimeError> {
                let object = value.downcast::<$crate::runtime::Object>()?;

                let mut output = <$ty>::try_with_capacity(object.len())?;

                for (key, value) in object {
                    let key = <$key>::try_from(key)?;
                    let value = <T>::from_value(value)?;
                    output.try_insert(key, value)?;
                }

                Ok(output)
            }
        }
    };
}

impl_try_map!(alloc::HashMap<String, T>, String);
impl_try_map!(alloc::HashMap<rust_alloc::string::String, T>, rust_alloc::string::String);

impl FromValue for Ordering {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        value.as_ordering()
    }
}

impl FromValue for Hash {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        value.as_hash()
    }
}
