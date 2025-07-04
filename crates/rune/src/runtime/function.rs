use core::fmt;
use core::future::Future;

use crate as rune;
use crate::alloc::fmt::TryWrite;
use crate::alloc::prelude::*;
use crate::alloc::{self, Box, Vec};
use crate::function;
use crate::runtime;
use crate::runtime::vm::Isolated;
use crate::shared::AssertSend;
use crate::sync::Arc;
use crate::{Any, Hash};

use super::{
    Address, AnySequence, Args, Call, ConstValue, Formatter, FromValue, FunctionHandler,
    GuardedArgs, Output, OwnedTuple, Rtti, RuntimeContext, RuntimeError, Stack, Unit, Value, Vm,
    VmCall, VmError, VmErrorKind, VmHalt,
};

/// The type of a function in Rune.
///
/// Functions can be called using call expression syntax, such as `<expr>()`.
///
/// There are multiple different kind of things which can be coerced into a
/// function in Rune:
/// * Regular functions.
/// * Closures (which might or might not capture their environment).
/// * Built-in constructors for tuple types (tuple structs, tuple variants).
///
/// # Examples
///
/// ```rune
/// // Captures the constructor for the `Some(<value>)` tuple variant.
/// let build_some = Some;
/// assert_eq!(build_some(42), Some(42));
///
/// fn build(value) {
///     Some(value)
/// }
///
/// // Captures the function previously defined.
/// let build_some = build;
/// assert_eq!(build_some(42), Some(42));
/// ```
#[derive(Any, TryClone)]
#[repr(transparent)]
#[rune(item = ::std::ops)]
pub struct Function(FunctionImpl<Value>);

impl Function {
    /// Construct a [Function] from a Rust closure.
    ///
    /// # Examples
    ///
    /// ```
    /// use rune::{Hash, Vm};
    /// use rune::runtime::Function;
    /// use rune::sync::Arc;
    ///
    /// let mut sources = rune::sources! {
    ///     entry => {
    ///         pub fn main(function) {
    ///             function(41)
    ///         }
    ///     }
    /// };
    ///
    /// let unit = rune::prepare(&mut sources).build()?;
    /// let unit = Arc::try_new(unit)?;
    /// let mut vm = Vm::without_runtime(unit)?;
    ///
    /// let function = Function::new(|value: u32| value + 1)?;
    ///
    /// assert_eq!(function.type_hash(), Hash::EMPTY);
    ///
    /// let value = vm.call(["main"], (function,))?;
    /// let value: u32 = rune::from_value(value)?;
    /// assert_eq!(value, 42);
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    ///
    /// Asynchronous functions:
    ///
    /// ```
    /// use rune::{Hash, Vm};
    /// use rune::runtime::Function;
    /// use rune::sync::Arc;
    ///
    /// # futures_executor::block_on(async move {
    /// let mut sources = rune::sources! {
    ///     entry => {
    ///         pub async fn main(function) {
    ///             function(41).await
    ///         }
    ///     }
    /// };
    ///
    /// let unit = rune::prepare(&mut sources).build()?;
    /// let unit = Arc::try_new(unit)?;
    /// let mut vm = Vm::without_runtime(unit)?;
    ///
    /// let function = Function::new(|value: u32| async move { value + 1 })?;
    ///
    /// assert_eq!(function.type_hash(), Hash::EMPTY);
    ///
    /// let value = vm.async_call(["main"], (function,)).await?;
    /// let value: u32 = rune::from_value(value)?;
    /// assert_eq!(value, 42);
    /// # Ok::<_, rune::support::Error>(())
    /// # })?;
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    pub fn new<F, A, K>(f: F) -> alloc::Result<Self>
    where
        F: function::Function<A, K>,
        K: function::FunctionKind,
    {
        Ok(Self(FunctionImpl {
            inner: Inner::FnHandler(FnHandler {
                handler: FunctionHandler::new(move |stack, addr, args, output| {
                    f.call(stack, addr, args, output)
                })?,
                hash: Hash::EMPTY,
            }),
        }))
    }

    /// Perform an asynchronous call over the function which also implements
    /// [Send].
    pub async fn async_send_call<A, T>(&self, args: A) -> Result<T, VmError>
    where
        A: Send + GuardedArgs,
        T: Send + FromValue,
    {
        self.0.async_send_call(args).await
    }

    /// Perform a call over the function represented by this function pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use rune::{Hash, Vm};
    /// use rune::runtime::Function;
    /// use rune::sync::Arc;
    ///
    /// let mut sources = rune::sources! {
    ///     entry => {
    ///         fn add(a, b) {
    ///             a + b
    ///         }
    ///
    ///         pub fn main() { add }
    ///     }
    /// };
    ///
    /// let unit = rune::prepare(&mut sources).build()?;
    /// let unit = Arc::try_new(unit)?;
    /// let mut vm = Vm::without_runtime(unit)?;
    ///
    /// let value = vm.call(["main"], ())?;
    ///
    /// let value: Function = rune::from_value(value)?;
    /// assert_eq!(value.call::<u32>((1, 2))?, 3);
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    pub fn call<T>(&self, args: impl GuardedArgs) -> Result<T, VmError>
    where
        T: FromValue,
    {
        self.0.call(args)
    }

    /// Call with the given virtual machine. This allows for certain
    /// optimizations, like avoiding the allocation of a new vm state in case
    /// the call is internal.
    ///
    /// A stop reason will be returned in case the function call results in
    /// a need to suspend the execution.
    pub(crate) fn call_with_vm(
        &self,
        vm: &mut Vm,
        addr: Address,
        args: usize,
        out: Output,
    ) -> Result<Option<VmHalt>, VmError> {
        self.0.call_with_vm(vm, addr, args, out)
    }

    /// Create a function pointer from a handler.
    pub(crate) fn from_handler(handler: FunctionHandler, hash: Hash) -> Self {
        Self(FunctionImpl::from_handler(handler, hash))
    }

    /// Create a function pointer from an offset.
    pub(crate) fn from_vm_offset(
        context: Arc<RuntimeContext>,
        unit: Arc<Unit>,
        offset: usize,
        call: Call,
        args: usize,
        hash: Hash,
    ) -> Self {
        Self(FunctionImpl::from_offset(
            context, unit, offset, call, args, hash,
        ))
    }

    /// Create a function pointer from an offset.
    pub(crate) fn from_vm_closure(
        context: Arc<RuntimeContext>,
        unit: Arc<Unit>,
        offset: usize,
        call: Call,
        args: usize,
        environment: Box<[Value]>,
        hash: Hash,
    ) -> Self {
        Self(FunctionImpl::from_closure(
            context,
            unit,
            offset,
            call,
            args,
            environment,
            hash,
        ))
    }

    /// Create a function pointer from an offset.
    pub(crate) fn from_unit_struct(rtti: Arc<Rtti>) -> Self {
        Self(FunctionImpl::from_unit_struct(rtti))
    }

    /// Create a function pointer from an offset.
    pub(crate) fn from_tuple_struct(rtti: Arc<Rtti>, args: usize) -> Self {
        Self(FunctionImpl::from_tuple_struct(rtti, args))
    }

    /// Type [Hash][struct@Hash] of the underlying function.
    ///
    /// # Examples
    ///
    /// The type hash of a top-level function matches what you get out of
    /// [Hash::type_hash].
    ///
    /// ```
    /// use rune::runtime::Function;
    /// use rune::sync::Arc;
    /// use rune::{Hash, Vm};
    ///
    /// let mut sources = rune::sources! {
    ///     entry => {
    ///         fn pony() { }
    ///
    ///         pub fn main() { pony }
    ///     }
    /// };
    ///
    /// let unit = rune::prepare(&mut sources).build()?;
    /// let unit = Arc::try_new(unit)?;
    /// let mut vm = Vm::without_runtime(unit)?;
    ///
    /// let pony = vm.call(["main"], ())?;
    /// let pony: Function = rune::from_value(pony)?;
    ///
    /// assert_eq!(pony.type_hash(), Hash::type_hash(["pony"]));
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    pub fn type_hash(&self) -> Hash {
        self.0.type_hash()
    }

    /// Try to convert into a [SyncFunction]. This might not be possible if this
    /// function is something which is not [Sync], like a closure capturing
    /// context which is not thread-safe.
    ///
    /// # Examples
    ///
    /// ```
    /// use rune::{Hash, Vm};
    /// use rune::runtime::Function;
    /// use rune::sync::Arc;
    ///
    /// let mut sources = rune::sources! {
    ///     entry => {
    ///         fn pony() { }
    ///
    ///         pub fn main() { pony }
    ///     }
    /// };
    ///
    /// let unit = rune::prepare(&mut sources).build()?;
    /// let unit = Arc::try_new(unit)?;
    /// let mut vm = Vm::without_runtime(unit)?;
    ///
    /// let pony = vm.call(["main"], ())?;
    /// let pony: Function = rune::from_value(pony)?;
    ///
    /// // This is fine, since `pony` is a free function.
    /// let pony = pony.into_sync()?;
    ///
    /// assert_eq!(pony.type_hash(), Hash::type_hash(["pony"]));
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    ///
    /// The following *does not* work, because we return a closure which tries
    /// to make use of a [Generator][crate::runtime::Generator] which is not a
    /// constant value.
    ///
    /// ```
    /// use rune::runtime::Function;
    /// use rune::sync::Arc;
    /// use rune::{Hash, Vm};
    ///
    /// let mut sources = rune::sources! {
    ///     entry => {
    ///         fn generator() {
    ///             yield 42;
    ///         }
    ///
    ///         pub fn main() {
    ///             let g = generator();
    ///
    ///             move || {
    ///                 g.next()
    ///             }
    ///         }
    ///     }
    /// };
    ///
    /// let unit = rune::prepare(&mut sources).build()?;
    /// let unit = Arc::try_new(unit)?;
    /// let mut vm = Vm::without_runtime(unit)?;
    ///
    /// let closure = vm.call(["main"], ())?;
    /// let closure: Function = rune::from_value(closure)?;
    ///
    /// // This is *not* fine since the returned closure has captured a
    /// // generator which is not a constant value.
    /// assert!(closure.into_sync().is_err());
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    pub fn into_sync(self) -> Result<SyncFunction, RuntimeError> {
        Ok(SyncFunction(self.0.into_sync()?))
    }

    /// Clone a function.
    ///
    /// # Examples
    ///
    /// ```rune
    /// fn function() {
    ///     42
    /// }
    ///
    /// let a = function;
    /// let b = a.clone();
    /// assert_eq!(a(), b());
    /// ```
    #[rune::function(keep, protocol = CLONE)]
    fn clone(&self) -> Result<Function, VmError> {
        Ok(self.try_clone()?)
    }

    /// Debug format a function.
    ///
    /// # Examples
    ///
    /// ```rune
    /// fn function() {
    ///     42
    /// }
    ///
    /// println!("{function:?}");
    /// ``
    #[rune::function(keep, protocol = DEBUG_FMT)]
    fn debug_fmt(&self, f: &mut Formatter) -> alloc::Result<()> {
        write!(f, "{self:?}")
    }
}

/// A callable sync function. This currently only supports a subset of values
/// that are supported by the Vm.
#[repr(transparent)]
pub struct SyncFunction(FunctionImpl<ConstValue>);

assert_impl!(SyncFunction: Send + Sync);

impl SyncFunction {
    /// Perform an asynchronous call over the function which also implements
    /// [Send].
    ///
    /// # Examples
    ///
    /// ```
    /// use rune::runtime::SyncFunction;
    /// use rune::sync::Arc;
    /// use rune::{Hash, Vm};
    ///
    /// # futures_executor::block_on(async move {
    /// let mut sources = rune::sources! {
    ///     entry => {
    ///         async fn add(a, b) {
    ///             a + b
    ///         }
    ///
    ///         pub fn main() { add }
    ///     }
    /// };
    ///
    /// let unit = rune::prepare(&mut sources).build()?;
    /// let unit = Arc::try_new(unit)?;
    /// let mut vm = Vm::without_runtime(unit)?;
    ///
    /// let add = vm.call(["main"], ())?;
    /// let add: SyncFunction = rune::from_value(add)?;
    ///
    /// let value = add.async_send_call::<u32>((1, 2)).await?;
    /// assert_eq!(value, 3);
    /// # Ok::<_, rune::support::Error>(())
    /// # })?;
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    pub async fn async_send_call<T>(&self, args: impl GuardedArgs + Send) -> Result<T, VmError>
    where
        T: Send + FromValue,
    {
        self.0.async_send_call(args).await
    }

    /// Perform a call over the function represented by this function pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use rune::runtime::SyncFunction;
    /// use rune::sync::Arc;
    /// use rune::{Hash, Vm};
    ///
    /// let mut sources = rune::sources! {
    ///     entry => {
    ///         fn add(a, b) {
    ///             a + b
    ///         }
    ///
    ///         pub fn main() { add }
    ///     }
    /// };
    ///
    /// let unit = rune::prepare(&mut sources).build()?;
    /// let unit = Arc::try_new(unit)?;
    /// let mut vm = Vm::without_runtime(unit)?;
    ///
    /// let add = vm.call(["main"], ())?;
    /// let add: SyncFunction = rune::from_value(add)?;
    ///
    /// assert_eq!(add.call::<u32>((1, 2))?, 3);
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    pub fn call<T>(&self, args: impl GuardedArgs) -> Result<T, VmError>
    where
        T: FromValue,
    {
        self.0.call(args)
    }

    /// Type [Hash][struct@Hash] of the underlying function.
    ///
    /// # Examples
    ///
    /// The type hash of a top-level function matches what you get out of
    /// [Hash::type_hash].
    ///
    /// ```
    /// use rune::runtime::SyncFunction;
    /// use rune::sync::Arc;
    /// use rune::{Hash, Vm};
    ///
    /// let mut sources = rune::sources! {
    ///     entry => {
    ///         fn pony() { }
    ///
    ///         pub fn main() { pony }
    ///     }
    /// };
    ///
    /// let unit = rune::prepare(&mut sources).build()?;
    /// let unit = Arc::try_new(unit)?;
    /// let mut vm = Vm::without_runtime(unit)?;
    ///
    /// let pony = vm.call(["main"], ())?;
    /// let pony: SyncFunction = rune::from_value(pony)?;
    ///
    /// assert_eq!(pony.type_hash(), Hash::type_hash(["pony"]));
    /// # Ok::<_, rune::support::Error>(())
    /// ```
    pub fn type_hash(&self) -> Hash {
        self.0.type_hash()
    }
}

impl TryClone for SyncFunction {
    fn try_clone(&self) -> alloc::Result<Self> {
        Ok(Self(self.0.try_clone()?))
    }
}

/// A stored function, of some specific kind.
struct FunctionImpl<V> {
    inner: Inner<V>,
}

impl<V> TryClone for FunctionImpl<V>
where
    V: TryClone,
{
    #[inline]
    fn try_clone(&self) -> alloc::Result<Self> {
        Ok(Self {
            inner: self.inner.try_clone()?,
        })
    }
}

impl<V> FunctionImpl<V>
where
    V: TryClone,
    OwnedTuple: TryFrom<Box<[V]>>,
    VmErrorKind: From<<OwnedTuple as TryFrom<Box<[V]>>>::Error>,
{
    fn call<T>(&self, args: impl GuardedArgs) -> Result<T, VmError>
    where
        T: FromValue,
    {
        let value = match &self.inner {
            Inner::FnHandler(handler) => {
                let count = args.count();
                let size = count.max(1);
                // Ensure we have space for the return value.
                let mut stack = Stack::with_capacity(size)?;
                let _guard = unsafe { args.guarded_into_stack(&mut stack) }?;
                stack.resize(size)?;
                handler
                    .handler
                    .call(&mut stack, Address::ZERO, count, Address::ZERO.output())?;
                stack.at(Address::ZERO).clone()
            }
            Inner::FnOffset(fn_offset) => fn_offset.call(args, ())?,
            Inner::FnClosureOffset(closure) => {
                let environment = closure.environment.try_clone()?;
                let environment = OwnedTuple::try_from(environment)?;
                closure.fn_offset.call(args, (environment,))?
            }
            Inner::FnUnitStruct(empty) => {
                check_args(args.count(), 0)?;
                Value::empty_struct(empty.rtti.clone())?
            }
            Inner::FnTupleStruct(tuple) => {
                check_args(args.count(), tuple.args)?;
                // SAFETY: We don't let the guard outlive the value.
                let (args, _guard) = unsafe { args.guarded_into_vec()? };
                Value::tuple_struct(tuple.rtti.clone(), args)?
            }
        };

        Ok(T::from_value(value)?)
    }

    fn async_send_call<'a, A, T>(
        &'a self,
        args: A,
    ) -> impl Future<Output = Result<T, VmError>> + Send + 'a
    where
        A: 'a + Send + GuardedArgs,
        T: 'a + Send + FromValue,
    {
        let future = async move {
            let value: Value = self.call(args)?;

            let value = match value.try_borrow_mut::<runtime::Future>()? {
                Some(future) => future.await?,
                None => value,
            };

            Ok(T::from_value(value)?)
        };

        // Safety: Future is send because there is no way to call this
        // function in a manner which allows any values from the future
        // to escape outside of this future, hence it can only be
        // scheduled by one thread at a time.
        unsafe { AssertSend::new(future) }
    }

    /// Call with the given virtual machine. This allows for certain
    /// optimizations, like avoiding the allocation of a new vm state in case
    /// the call is internal.
    ///
    /// A stop reason will be returned in case the function call results in
    /// a need to suspend the execution.
    pub(crate) fn call_with_vm(
        &self,
        vm: &mut Vm,
        addr: Address,
        args: usize,
        out: Output,
    ) -> Result<Option<VmHalt>, VmError> {
        let reason = match &self.inner {
            Inner::FnHandler(handler) => {
                handler.handler.call(vm.stack_mut(), addr, args, out)?;
                None
            }
            Inner::FnOffset(fn_offset) => {
                if let Some(vm_call) = fn_offset.call_with_vm(vm, addr, args, (), out)? {
                    return Ok(Some(VmHalt::VmCall(vm_call)));
                }

                None
            }
            Inner::FnClosureOffset(closure) => {
                let environment = closure.environment.try_clone()?;
                let environment = OwnedTuple::try_from(environment)?;

                if let Some(vm_call) =
                    closure
                        .fn_offset
                        .call_with_vm(vm, addr, args, (environment,), out)?
                {
                    return Ok(Some(VmHalt::VmCall(vm_call)));
                }

                None
            }
            Inner::FnUnitStruct(empty) => {
                check_args(args, 0)?;
                vm.stack_mut()
                    .store(out, || Value::empty_struct(empty.rtti.clone()))?;
                None
            }
            Inner::FnTupleStruct(tuple) => {
                check_args(args, tuple.args)?;

                let seq = vm.stack().slice_at(addr, args)?;
                let data = seq.iter().cloned();
                let value = AnySequence::new(tuple.rtti.clone(), data)?;
                vm.stack_mut().store(out, value)?;
                None
            }
        };

        Ok(reason)
    }

    /// Create a function pointer from a handler.
    pub(crate) fn from_handler(handler: FunctionHandler, hash: Hash) -> Self {
        Self {
            inner: Inner::FnHandler(FnHandler { handler, hash }),
        }
    }

    /// Create a function pointer from an offset.
    pub(crate) fn from_offset(
        context: Arc<RuntimeContext>,
        unit: Arc<Unit>,
        offset: usize,
        call: Call,
        args: usize,
        hash: Hash,
    ) -> Self {
        Self {
            inner: Inner::FnOffset(FnOffset {
                context,
                unit,
                offset,
                call,
                args,
                hash,
            }),
        }
    }

    /// Create a function pointer from an offset.
    pub(crate) fn from_closure(
        context: Arc<RuntimeContext>,
        unit: Arc<Unit>,
        offset: usize,
        call: Call,
        args: usize,
        environment: Box<[V]>,
        hash: Hash,
    ) -> Self {
        Self {
            inner: Inner::FnClosureOffset(FnClosureOffset {
                fn_offset: FnOffset {
                    context,
                    unit,
                    offset,
                    call,
                    args,
                    hash,
                },
                environment,
            }),
        }
    }

    /// Create a function pointer from an offset.
    pub(crate) fn from_unit_struct(rtti: Arc<Rtti>) -> Self {
        Self {
            inner: Inner::FnUnitStruct(FnUnitStruct { rtti }),
        }
    }

    /// Create a function pointer from an offset.
    pub(crate) fn from_tuple_struct(rtti: Arc<Rtti>, args: usize) -> Self {
        Self {
            inner: Inner::FnTupleStruct(FnTupleStruct { rtti, args }),
        }
    }

    #[inline]
    fn type_hash(&self) -> Hash {
        match &self.inner {
            Inner::FnHandler(FnHandler { hash, .. }) | Inner::FnOffset(FnOffset { hash, .. }) => {
                *hash
            }
            Inner::FnClosureOffset(fco) => fco.fn_offset.hash,
            Inner::FnUnitStruct(func) => func.rtti.type_hash(),
            Inner::FnTupleStruct(func) => func.rtti.type_hash(),
        }
    }
}

impl FunctionImpl<Value> {
    /// Try to convert into a [SyncFunction].
    fn into_sync(self) -> Result<FunctionImpl<ConstValue>, RuntimeError> {
        let inner = match self.inner {
            Inner::FnClosureOffset(closure) => {
                let mut env = Vec::try_with_capacity(closure.environment.len())?;

                for value in Vec::from(closure.environment) {
                    env.try_push(FromValue::from_value(value)?)?;
                }

                Inner::FnClosureOffset(FnClosureOffset {
                    fn_offset: closure.fn_offset,
                    environment: env.try_into_boxed_slice()?,
                })
            }
            Inner::FnHandler(inner) => Inner::FnHandler(inner),
            Inner::FnOffset(inner) => Inner::FnOffset(inner),
            Inner::FnUnitStruct(inner) => Inner::FnUnitStruct(inner),
            Inner::FnTupleStruct(inner) => Inner::FnTupleStruct(inner),
        };

        Ok(FunctionImpl { inner })
    }
}

impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0.inner {
            Inner::FnHandler(handler) => {
                write!(f, "native function ({:p})", handler.handler)?;
            }
            Inner::FnOffset(offset) => {
                write!(f, "{} function (at: 0x{:x})", offset.call, offset.offset)?;
            }
            Inner::FnClosureOffset(closure) => {
                write!(
                    f,
                    "closure (at: 0x{:x}, env:{:?})",
                    closure.fn_offset.offset, closure.environment
                )?;
            }
            Inner::FnUnitStruct(empty) => {
                write!(f, "empty {}", empty.rtti.item)?;
            }
            Inner::FnTupleStruct(tuple) => {
                write!(f, "tuple {}", tuple.rtti.item)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Inner<V> {
    /// A native function handler.
    /// This is wrapped as an `Arc<dyn FunctionHandler>`.
    FnHandler(FnHandler),
    /// The offset to a free function.
    ///
    /// This also captures the context and unit it belongs to allow for external
    /// calls.
    FnOffset(FnOffset),
    /// A closure with a captured environment.
    ///
    /// This also captures the context and unit it belongs to allow for external
    /// calls.
    FnClosureOffset(FnClosureOffset<V>),
    /// Constructor for a unit struct.
    FnUnitStruct(FnUnitStruct),
    /// Constructor for a tuple.
    FnTupleStruct(FnTupleStruct),
}

impl<V> TryClone for Inner<V>
where
    V: TryClone,
{
    fn try_clone(&self) -> alloc::Result<Self> {
        Ok(match self {
            Inner::FnHandler(inner) => Inner::FnHandler(inner.clone()),
            Inner::FnOffset(inner) => Inner::FnOffset(inner.clone()),
            Inner::FnClosureOffset(inner) => Inner::FnClosureOffset(inner.try_clone()?),
            Inner::FnUnitStruct(inner) => Inner::FnUnitStruct(inner.clone()),
            Inner::FnTupleStruct(inner) => Inner::FnTupleStruct(inner.clone()),
        })
    }
}

#[derive(Clone, TryClone)]
struct FnHandler {
    /// The function handler.
    handler: FunctionHandler,
    /// Hash for the function type
    hash: Hash,
}

impl fmt::Debug for FnHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FnHandler")
    }
}

#[derive(Clone, TryClone)]
struct FnOffset {
    context: Arc<RuntimeContext>,
    /// The unit where the function resides.
    unit: Arc<Unit>,
    /// The offset of the function.
    offset: usize,
    /// The calling convention.
    call: Call,
    /// The number of arguments the function takes.
    args: usize,
    /// Hash for the function type
    hash: Hash,
}

impl FnOffset {
    /// Perform a call into the specified offset and return the produced value.
    #[tracing::instrument(skip_all, fields(args = args.count(), extra = extra.count(), ?self.offset, ?self.call, ?self.args, ?self.hash))]
    fn call(&self, args: impl GuardedArgs, extra: impl Args) -> Result<Value, VmError> {
        check_args(args.count().wrapping_add(extra.count()), self.args)?;

        let mut vm = Vm::new(self.context.clone(), self.unit.clone());

        vm.set_ip(self.offset);
        let _guard = unsafe { args.guarded_into_stack(vm.stack_mut())? };
        extra.into_stack(vm.stack_mut())?;

        self.call.call_with_vm(vm)
    }

    /// Perform a potentially optimized call into the specified vm.
    ///
    /// This will cause a halt in case the vm being called into isn't the same
    /// as the context and unit of the function.
    #[tracing::instrument(skip_all, fields(args, extra = extra.count(), keep, ?self.offset, ?self.call, ?self.args, ?self.hash))]
    fn call_with_vm(
        &self,
        vm: &mut Vm,
        addr: Address,
        args: usize,
        extra: impl Args,
        out: Output,
    ) -> Result<Option<VmCall>, VmError> {
        check_args(args.wrapping_add(extra.count()), self.args)?;

        let same_unit = matches!(self.call, Call::Immediate if vm.is_same_unit(&self.unit));
        let same_context =
            matches!(self.call, Call::Immediate if vm.is_same_context(&self.context));

        vm.push_call_frame(self.offset, addr, args, Isolated::new(!same_context), out)?;
        extra.into_stack(vm.stack_mut())?;

        // Fast path, just allocate a call frame and keep running.
        if same_context && same_unit {
            tracing::trace!("same context and unit");
            return Ok(None);
        }

        let call = VmCall::new(
            self.call,
            (!same_context).then(|| self.context.clone()),
            (!same_unit).then(|| self.unit.clone()),
            out,
        );

        Ok(Some(call))
    }
}

impl fmt::Debug for FnOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FnOffset")
            .field("context", &(&self.context as *const _))
            .field("unit", &(&self.unit as *const _))
            .field("offset", &self.offset)
            .field("call", &self.call)
            .field("args", &self.args)
            .finish()
    }
}

#[derive(Debug)]
struct FnClosureOffset<V> {
    /// The offset in the associated unit that the function lives.
    fn_offset: FnOffset,
    /// Captured environment.
    environment: Box<[V]>,
}

impl<V> TryClone for FnClosureOffset<V>
where
    V: TryClone,
{
    #[inline]
    fn try_clone(&self) -> alloc::Result<Self> {
        Ok(Self {
            fn_offset: self.fn_offset.clone(),
            environment: self.environment.try_clone()?,
        })
    }
}

#[derive(Debug, Clone, TryClone)]
struct FnUnitStruct {
    /// The type of the empty.
    rtti: Arc<Rtti>,
}

#[derive(Debug, Clone, TryClone)]
struct FnTupleStruct {
    /// The type of the tuple.
    rtti: Arc<Rtti>,
    /// The number of arguments the tuple takes.
    args: usize,
}

impl FromValue for SyncFunction {
    #[inline]
    fn from_value(value: Value) -> Result<Self, RuntimeError> {
        value.downcast::<Function>()?.into_sync()
    }
}

#[inline]
fn check_args(actual: usize, expected: usize) -> Result<(), VmError> {
    if actual != expected {
        return Err(VmError::new(VmErrorKind::BadArgumentCount {
            expected,
            actual,
        }));
    }

    Ok(())
}
