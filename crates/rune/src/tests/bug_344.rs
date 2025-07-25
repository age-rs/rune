//! Test a bug where the contract for `UnsafeFromValue` was not being properly
//! upheld by internal module helpers when registering a native function.
//!
//! This ensures that the contract "works" by checking that a value which is
//! being used hasn't had its guard dropped through a shared reference-counted
//! cell.
//!
//! See: https://github.com/rune-rs/rune/issues/344

prelude!();

use std::cell::Cell;
use std::rc::Rc;

use crate::compile::meta;
use crate::runtime::{AnyTypeInfo, RuntimeError};

#[test]
fn bug_344_function() -> Result<()> {
    let mut context = Context::new();
    let mut module = Module::new();

    module.function("function", function).build()?;

    context.install(module)?;
    let runtime = context.runtime()?;

    let function = runtime.function(&hash!(function)).expect("expect function");

    let mut stack = Stack::new();
    stack.push(rune::to_value(GuardCheck::new())?)?;
    function.call(&mut stack, Address::new(0), 1, Output::keep(0))?;
    assert_eq!(stack.at(Address::new(0)).as_signed()?, 42);
    return Ok(());

    fn function(check: &GuardCheck) -> i64 {
        check.ensure_not_dropped("immediate argument");
        42
    }
}

#[test]
fn bug_344_inst_fn() -> Result<()> {
    #[rune::function(instance)]
    fn function(s: &GuardCheck, check: &GuardCheck) -> i64 {
        s.ensure_not_dropped("async self argument");
        check.ensure_not_dropped("async instance argument");
        42
    }

    let mut context = Context::new();
    let mut module = Module::new();

    module.ty::<GuardCheck>()?;
    module.function_meta(function)?;

    context.install(module)?;
    let runtime = context.runtime()?;

    let hash = Hash::associated_function(GuardCheck::HASH, "function");
    let function = runtime.function(&hash).expect("expect function");

    let mut stack = Stack::new();
    stack.push(rune::to_value(GuardCheck::new())?)?;
    stack.push(rune::to_value(GuardCheck::new())?)?;
    function.call(&mut stack, Address::ZERO, 2, Output::keep(0))?;

    assert_eq!(stack.at(Address::ZERO).as_signed()?, 42);
    Ok(())
}

#[test]
fn bug_344_async_function() -> Result<()> {
    let mut context = Context::new();
    let mut module = Module::new();

    module.function("function", function).build()?;

    context.install(module)?;
    let runtime = context.runtime()?;

    let function = runtime.function(&hash!(function)).expect("expect function");

    let mut stack = Stack::new();
    stack.push(rune::to_value(GuardCheck::new())?)?;
    function.call(&mut stack, Address::ZERO, 1, Output::keep(0))?;
    let future = stack.at(Address::ZERO).clone().into_future()?;
    assert_eq!(block_on(future)?.as_signed()?, 42);
    return Ok(());

    async fn function(check: Ref<GuardCheck>) -> i64 {
        check.ensure_not_dropped("async argument");
        42
    }
}

#[test]
fn bug_344_async_inst_fn() -> Result<()> {
    #[rune::function(instance)]
    async fn function(s: Ref<GuardCheck>, check: Ref<GuardCheck>) -> i64 {
        s.ensure_not_dropped("self argument");
        check.ensure_not_dropped("instance argument");
        42
    }

    let mut context = Context::new();
    let mut module = Module::new();

    module.ty::<GuardCheck>()?;
    module.function_meta(function)?;

    context.install(module)?;
    let runtime = context.runtime()?;

    let hash = Hash::associated_function(GuardCheck::HASH, "function");
    let function = runtime.function(&hash).expect("expect function");

    let mut stack = Stack::new();
    stack.push(rune::to_value(GuardCheck::new())?)?;
    stack.push(rune::to_value(GuardCheck::new())?)?;
    function.call(&mut stack, Address::new(0), 2, Output::keep(0))?;

    let future = stack.at(Address::new(0)).clone().into_future()?;
    assert_eq!(block_on(future)?.as_signed()?, 42);

    Ok(())
}

struct Guard {
    #[allow(unused)]
    guard: RawAnyGuard,
    dropped: Rc<Cell<bool>>,
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.dropped.set(true);
    }
}

struct GuardCheck {
    dropped: Rc<Cell<bool>>,
}

impl GuardCheck {
    fn new() -> Self {
        Self {
            dropped: Rc::new(Cell::new(false)),
        }
    }

    fn ensure_not_dropped(&self, what: &str) {
        assert!(
            !self.dropped.get(),
            "value has was previously dropped: {what}",
        );
    }
}

impl Any for GuardCheck {}
impl rune::__priv::AnyMarker for GuardCheck {}

impl Named for GuardCheck {
    const ITEM: &'static Item = rune_macros::item!(GuardCheck);
}

impl TypeHash for GuardCheck {
    const HASH: Hash = rune_macros::hash!(GuardCheck);
}

impl TypeOf for GuardCheck {
    const STATIC_TYPE_INFO: AnyTypeInfo = GuardCheck::ANY_TYPE_INFO;
}

impl MaybeTypeOf for GuardCheck {
    #[inline]
    fn maybe_type_of() -> alloc::Result<meta::DocType> {
        Ok(meta::DocType::new(Self::HASH))
    }
}

impl InstallWith for GuardCheck {}

impl UnsafeToRef for GuardCheck {
    type Guard = Guard;

    #[inline]
    unsafe fn unsafe_to_ref<'a>(value: Value) -> Result<(&'a Self, Self::Guard), RuntimeError> {
        let (output, guard) = Ref::into_raw(value.into_ref::<GuardCheck>()?);

        let guard = Guard {
            guard,
            // Regardless of what happens, the value is available here and the
            // refcounted value will be available even if the underlying value
            // *is* dropped prematurely because it's been cloned.
            dropped: output.as_ref().dropped.clone(),
        };

        Ok((output.as_ref(), guard))
    }
}
