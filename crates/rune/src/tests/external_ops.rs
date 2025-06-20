prelude!();

use std::cmp::Ordering;

#[test]
fn struct_assign() -> Result<()> {
    macro_rules! test_case {
        ([$($op:tt)*], $protocol:ident, $derived:tt, $initial:literal, $arg:literal, $expected:literal) => {{
            #[derive(Debug, Default, Any)]
            struct External {
                value: i64,
                field: i64,
                #[rune($derived)]
                derived: i64,
                #[rune($derived = External::custom)]
                custom: i64,
            }

            impl External {
                fn value(&mut self, value: i64) {
                    self.value $($op)* value;
                }

                fn field(&mut self, value: i64) {
                    self.field $($op)* value;
                }

                fn custom(&mut self, value: i64) {
                    self.custom $($op)* value;
                }
            }

            let mut module = Module::new();
            module.ty::<External>()?;

            module.associated_function(&Protocol::$protocol, External::value)?;
            module.field_function(&Protocol::$protocol, "field", External::field)?;

            let mut context = Context::default();
            context.install(module)?;
            let runtime = Arc::try_new(context.runtime()?)?;

            let mut sources = Sources::new();
            sources.insert(Source::new(
                "test",
                format!(r#"
                pub fn type(number) {{
                    number {op} {arg};
                    number.field {op} {arg};
                    number.derived {op} {arg};
                    number.custom {op} {arg};
                }}
                "#, op = stringify!($($op)*), arg = stringify!($arg)),
            )?)?;

            let unit = prepare(&mut sources)
                .with_context(&context)
                .build()?;

            let unit = Arc::try_new(unit)?;
            let vm = Vm::new(runtime, unit);

            {
                let mut foo = External::default();
                foo.value = $initial;
                foo.field = $initial;
                foo.derived = $initial;
                foo.custom = $initial;

                let output = vm.try_clone()?.call(["type"], (&mut foo,))?;

                assert_eq!(foo.value, $expected, "{} != {} (value)", foo.value, $expected);
                assert_eq!(foo.field, $expected, "{} != {} (field)", foo.field, $expected);
                assert_eq!(foo.derived, $expected, "{} != {} (derived)", foo.derived, $expected);
                assert_eq!(foo.custom, $expected, "{} != {} (custom)", foo.custom, $expected);
                output.into_unit().unwrap();
            }
        }};
    }

    test_case!([+=], ADD_ASSIGN, add_assign, 0, 3, 3);
    test_case!([-=], SUB_ASSIGN, sub_assign, 4, 3, 1);
    test_case!([*=], MUL_ASSIGN, mul_assign, 8, 2, 16);
    test_case!([/=], DIV_ASSIGN, div_assign, 8, 3, 2);
    test_case!([%=], REM_ASSIGN, rem_assign, 25, 10, 5);
    test_case!([&=], BIT_AND_ASSIGN, bit_and_assign, 0b1001, 0b0011, 0b0001);
    test_case!([|=], BIT_OR_ASSIGN, bit_or_assign, 0b1001, 0b0011, 0b1011);
    test_case!([^=], BIT_XOR_ASSIGN, bit_xor_assign, 0b1001, 0b0011, 0b1010);
    test_case!([<<=], SHL_ASSIGN, shl_assign, 0b1001, 0b0001, 0b10010);
    test_case!([>>=], SHR_ASSIGN, shr_assign, 0b1001, 0b0001, 0b100);
    Ok(())
}

#[test]
fn tuple_assign() -> Result<()> {
    macro_rules! test_case {
        ([$($op:tt)*], $protocol:ident, $derived:tt, $initial:literal, $arg:literal, $expected:literal) => {{
            #[derive(Debug, Default, Any)]
            struct External(i64, i64, #[rune($derived)] i64, #[rune($derived = External::custom)] i64);

            impl External {
                fn value(&mut self, value: i64) {
                    self.0 $($op)* value;
                }

                fn field(&mut self, value: i64) {
                    self.1 $($op)* value;
                }

                fn custom(&mut self, value: i64) {
                    self.3 $($op)* value;
                }
            }

            let mut module = Module::new();
            module.ty::<External>()?;

            module.associated_function(&Protocol::$protocol, External::value)?;
            module.index_function(&Protocol::$protocol, 1, External::field)?;

            let mut context = Context::default();
            context.install(module)?;
            let runtime = Arc::try_new(context.runtime()?)?;

            let mut sources = Sources::new();
            sources.insert(Source::new(
                "test",
                format!(r#"
                pub fn type(number) {{
                    number {op} {arg};
                    number.1 {op} {arg};
                    number.2 {op} {arg};
                    number.3 {op} {arg};
                }}
                "#, op = stringify!($($op)*), arg = stringify!($arg)),
            )?)?;

            let unit = prepare(&mut sources)
                .with_context(&context)
                .build()?;

            let unit = Arc::try_new(unit)?;
            let vm = Vm::new(runtime, unit);

            {
                let mut foo = External::default();
                foo.0 = $initial;
                foo.1 = $initial;
                foo.2 = $initial;
                foo.3 = $initial;

                let output = vm.try_clone()?.call(["type"], (&mut foo,))?;

                assert_eq!(foo.0, $expected, "{} != {} (value .0)", foo.0, $expected);
                assert_eq!(foo.1, $expected, "{} != {} (field .1)", foo.1, $expected);
                assert_eq!(foo.2, $expected, "{} != {} (derived .2)", foo.2, $expected);
                assert_eq!(foo.3, $expected, "{} != {} (custom .3)", foo.3, $expected);
                output.into_unit().unwrap();
            }
        }};
    }

    test_case!([+=], ADD_ASSIGN, add_assign, 0, 3, 3);
    test_case!([-=], SUB_ASSIGN, sub_assign, 4, 3, 1);
    test_case!([*=], MUL_ASSIGN, mul_assign, 8, 2, 16);
    test_case!([/=], DIV_ASSIGN, div_assign, 8, 3, 2);
    test_case!([%=], REM_ASSIGN, rem_assign, 25, 10, 5);
    test_case!([&=], BIT_AND_ASSIGN, bit_and_assign, 0b1001, 0b0011, 0b0001);
    test_case!([|=], BIT_OR_ASSIGN, bit_or_assign, 0b1001, 0b0011, 0b1011);
    test_case!([^=], BIT_XOR_ASSIGN, bit_xor_assign, 0b1001, 0b0011, 0b1010);
    test_case!([<<=], SHL_ASSIGN, shl_assign, 0b1001, 0b0001, 0b10010);
    test_case!([>>=], SHR_ASSIGN, shr_assign, 0b1001, 0b0001, 0b100);
    Ok(())
}

#[test]
fn struct_binary() -> Result<()> {
    macro_rules! test_case {
        ([$($op:tt)*], $protocol:ident, $derived:tt, $initial:literal, $arg:literal, $expected:literal) => {{
            #[derive(Debug, Any)]
            struct External {
                value: i64,
            }

            impl External {
                fn value(&self, value: i64) -> i64 {
                    self.value $($op)* value
                }
            }

            let mut module = Module::new();
            module.ty::<External>()?;

            module.associated_function(&Protocol::$protocol, External::value)?;

            let mut context = Context::default();
            context.install(module)?;
            let runtime = Arc::try_new(context.runtime()?)?;

            let source = format!("pub fn type(number) {{ number {op} {arg} }}", op = stringify!($($op)*), arg = stringify!($arg));

            let mut sources = Sources::new();
            sources.insert(Source::memory(source)?)?;

            let unit = prepare(&mut sources)
                .with_context(&context)
                .build()?;

            let unit = Arc::try_new(unit)?;
            let mut vm = Vm::new(runtime, unit);

            let foo = External { value: $initial };
            let output = vm.call(["type"], (foo,))?;
            let value = crate::from_value::<i64>(output)?;

            let expected: i64 = $expected;
            assert_eq!(value, expected, "{value} != {expected} (value)");
        }};
    }

    test_case!([+], ADD, add, 0, 3, 3);
    test_case!([-], SUB, sub, 4, 3, 1);
    test_case!([*], MUL, mul, 8, 2, 16);
    test_case!([/], DIV, div, 8, 3, 2);
    test_case!([%], REM, rem, 25, 10, 5);
    test_case!([&], BIT_AND, bit_and, 0b1001, 0b0011, 0b0001);
    test_case!([|], BIT_OR, bit_or, 0b1001, 0b0011, 0b1011);
    test_case!([^], BIT_XOR, bit_xor, 0b1001, 0b0011, 0b1010);
    test_case!([<<], SHL, shl, 0b1001, 0b0001, 0b10010);
    test_case!([>>], SHR, shr, 0b1001, 0b0001, 0b100);
    Ok(())
}

#[test]
fn struct_unary() -> Result<()> {
    macro_rules! test_case {
        ([$($op:tt)*], $protocol:ident, $derived:tt, $ty:ty, $initial:literal, $expected:expr) => {{
            #[derive(Debug, Any)]
            struct External {
                value: $ty,
            }

            impl External {
                fn value(&self) -> External {
                    External {
                        value: $($op)*self.value,
                    }
                }
            }

            let mut module = Module::new();
            module.ty::<External>()?;
            module.associated_function(&Protocol::$protocol, External::value)?;

            let mut context = Context::default();
            context.install(module)?;
            let runtime = Arc::try_new(context.runtime()?)?;

            let source = format!("pub fn type(value) {{ {op} value }}", op = stringify!($($op)*));

            let mut sources = Sources::new();
            sources.insert(Source::memory(source)?)?;

            let unit = prepare(&mut sources)
                .with_context(&context)
                .build()?;

            let unit = Arc::try_new(unit)?;
            let mut vm = Vm::new(runtime, unit);

            let external = External { value: $initial };

            let output = vm.call(["type"], (external,))?;
            let External { value: actual } = crate::from_value(output)?;

            let expected: $ty = $expected;
            assert_eq!(actual, expected);
        }};
    }

    test_case!([-], NEG, neg, i64, 100, -100);
    test_case!([-], NEG, neg, f64, 100.0, -100.0);
    test_case!([!], NOT, not, i64, 100, !100);
    test_case!([!], NOT, not, u64, 100, !100);
    test_case!([!], NOT, not, bool, true, false);
    Ok(())
}

#[test]
fn ordering_struct() -> Result<()> {
    macro_rules! test_case {
        ([$($op:tt)*], $protocol:ident, $initial:literal, $arg:literal, $expected:literal) => {{
            #[derive(Debug, Default, Any)]
            struct External {
                value: i64,
            }

            impl External {
                fn value(&self, value: i64) -> Option<Ordering> {
                    PartialOrd::partial_cmp(&self.value, &value)
                }
            }

            let mut module = Module::new();
            module.ty::<External>()?;

            module.associated_function(&Protocol::$protocol, External::value)?;

            let mut context = Context::default();
            context.install(module)?;

            let runtime = Arc::try_new(context.runtime()?)?;

            let mut sources = Sources::new();
            sources.insert(Source::new(
                "test",
                format!(r#"
                pub fn type(number) {{
                    number {op} {arg}
                }}
                "#, op = stringify!($($op)*), arg = stringify!($arg)),
            )?)?;

            let unit = prepare(&mut sources)
                .with_context(&context)
                .build()?;

            let unit = Arc::try_new(unit)?;
            let vm = Vm::new(runtime, unit);

            {
                let mut foo = External::default();
                foo.value = $initial;

                let output = vm.try_clone()?.call(["type"], (&mut foo,))?;
                let a: bool = rune::from_value(output)?;

                assert_eq!(a, $expected, "{} != {} (value)", foo.value, $expected);
            }
        }};
    }

    test_case!([<], PARTIAL_CMP, 1, 2, true);
    test_case!([<], PARTIAL_CMP, 2, 1, false);

    test_case!([>], PARTIAL_CMP, 2, 1, true);
    test_case!([>], PARTIAL_CMP, 1, 2, false);

    test_case!([>=], PARTIAL_CMP, 3, 2, true);
    test_case!([>=], PARTIAL_CMP, 2, 2, true);
    test_case!([>=], PARTIAL_CMP, 1, 2, false);

    test_case!([<=], PARTIAL_CMP, 2, 3, true);
    test_case!([<=], PARTIAL_CMP, 2, 2, true);
    test_case!([<=], PARTIAL_CMP, 2, 1, false);
    Ok(())
}

#[test]
fn eq_struct() -> Result<()> {
    macro_rules! test_case {
        ([$($op:tt)*], $protocol:ident, $initial:literal, $arg:literal, $expected:literal) => {{
            #[derive(Debug, Default, Any)]
            struct External {
                value: i64,
            }

            impl External {
                fn value(&self, value: i64) -> bool {
                    self.value $($op)* value
                }
            }

            let mut module = Module::new();
            module.ty::<External>()?;

            module.associated_function(&Protocol::$protocol, External::value)?;

            let mut context = Context::default();
            context.install(module)?;

            let runtime = Arc::try_new(context.runtime()?)?;

            let mut sources = Sources::new();
            sources.insert(Source::new(
                "test",
                format!(r#"
                pub fn type(number) {{ number {op} {arg} }}
                "#, op = stringify!($($op)*), arg = stringify!($arg)),
            )?)?;

            let unit = prepare(&mut sources)
                .with_context(&context)
                .build()?;

            let unit = Arc::try_new(unit)?;
            let vm = Vm::new(runtime, unit);

            {
                let mut foo = External::default();
                foo.value = $initial;

                let output = vm.try_clone()?.call(["type"], (&mut foo,))?;
                let a: bool = rune::from_value(output)?;

                assert_eq!(a, $expected, "{} != {} (value)", foo.value, $expected);
            }
        }};
    }

    test_case!([==], PARTIAL_EQ, 2, 2, true);
    test_case!([==], PARTIAL_EQ, 2, 1, false);
    Ok(())
}
