# Field functions

Field functions are special operations which operate on fields. These are
distinct from associated functions, because they are invoked by using the
operation associated with the kind of the field function.

The most common forms of fields functions are *getters* and *setters*, which are
defined through the [`Protocol::GET`] and [`Protocol::SET`] protocols.

The `Any` derive can also generate default implementations of these through
various `#[rune(...)]` attributes:

```rust,noplaypen
#[derive(Any)]
struct External {
    #[rune(get, set, add_assign, copy)]
    number: i64,
    #[rune(get, set)]
    string: String,
}
```

Once registered, this allows `External` to be used like this in Rune:

```rune
pub fn main(external) {
    external.number = external.number + 1;
    external.number += 1;
    external.string = `${external.string} World`;
}
```

The full list of available field functions and their corresponding attributes
are:

| Protocol | Attribute | |
|-|-|-|
| [`Protocol::GET`] | `#[rune(get)]` | For getters, like `external.field`. |
| [`Protocol::SET`] | `#[rune(set)]` | For setters, like `external.field = 42`. |
| [`Protocol::ADD_ASSIGN`] | `#[rune(add_assign)]` | The `+=` operation. |
| [`Protocol::SUB_ASSIGN`] | `#[rune(sub_assign)]` | The `-=` operation. |
| [`Protocol::MUL_ASSIGN`] | `#[rune(mul_assign)]` | The `*=` operation. |
| [`Protocol::DIV_ASSIGN`] | `#[rune(div_assign)]` | The `/=` operation. |
| [`Protocol::BIT_AND_ASSIGN`] | `#[rune(bit_and_assign)]` | The `&=` operation. |
| [`Protocol::BIT_OR_ASSIGN`] | `#[rune(bit_or_assign)]` | The bitwise or operation. |
| [`Protocol::BIT_XOR_ASSIGN`] | `#[rune(bit_xor_assign)]` | The `^=` operation. |
| [`Protocol::SHL_ASSIGN`] | `#[rune(shl_assign)]` | The `<<=` operation. |
| [`Protocol::SHR_ASSIGN`] | `#[rune(shr_assign)]` | The `>>=` operation. |
| [`Protocol::REM_ASSIGN`] | `#[rune(rem_assign)]` | The `%=` operation. |

The manual way to register these functions is to use the new `Module::field_function`
function. This clearly showcases that there's no relationship between the field
used and the function registered:

```rust,noplaypen
use rune::{Any, Module};
use rune::runtime::Protocol;

#[derive(Any)]
struct External {
}

impl External {
    fn field_get(&self) -> String {
        String::from("Hello World")
    }
}

let mut module = Module::new();
module.field_function(&Protocol::GET, "field", External::field_get)?;
```

Would allow for this in Rune:

```rune
pub fn main(external) {
    println!("{}", external.field);
}
```

## Customizing how fields are cloned with `#[rune(get)]`

In order to return a value through `#[rune(get)]`, the value has to be cloned.

By default, this is done through the [`TryClone` trait], but its behavior can be
customized through the following attributes:

#### `#[rune(copy)]`

This indicates that the field is `Copy`.

#### `#[rune(clone)]`

This indicates that the field should use `std::clone::Clone` to clone the value.
Note that this effecitvely means that the memory the value uses during cloning
is *not* tracked and should be avoided in favor of using [`rune::alloc`] and the
[`TryClone` trait] without good reason.

#### `#[rune(clone_with = <path>)]`

This specified a custom method that should be used to clone the value.

```rust,noplaypen
use rune::Any;
use rune::sync::Arc;

#[derive(Any)]
struct External {
    #[rune(get, clone_with = Thing::clone)]
    field: Thing,
}

#[derive(Any, Clone)]
struct Thing {
    name: Arc<String>,
}
```

#### `#[rune(try_clone_with = <path>)]`

This specified a custom method that should be used to clone the value.

```rust,noplaypen
use rune::Any;
use rune::prelude::*;

#[derive(Any)]
struct External {
    #[rune(get, try_clone_with = String::try_clone)]
    field: String,
}
```

## Custom field function

Using the `Any` derive, you can specify a custom field function by using an
argument to the corresponding attribute pointing to the function to use instead.

The following uses an implementation of `add_assign` which performs checked
addition:

```rust,noplaypen
{{#include ../../examples/examples/checked_add_assign.rs}}
```

```text
$> cargo run --example checked_add_assign
Error: numerical overflow (at inst 2)
```

[`Protocol::ADD_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.ADD_ASSIGN
[`Protocol::BIT_AND_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.BIT_AND_ASSIGN
[`Protocol::BIT_OR_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.BIT_OR_ASSIGN
[`Protocol::BIT_XOR_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.BIT_XOR_ASSIGN
[`Protocol::DIV_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.DIV_ASSIGN
[`Protocol::GET`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.GET
[`Protocol::MUL_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.MUL_ASSIGN
[`Protocol::REM_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.REM_ASSIGN
[`Protocol::SET`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.SET
[`Protocol::SHL_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.SHL_ASSIGN
[`Protocol::SHR_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.SHR_ASSIGN
[`Protocol::SUB_ASSIGN`]: https://docs.rs/rune/0/rune/runtime/struct.Protocol.html#associatedconstant.SUB_ASSIGN
[`rune::alloc`]: https://docs.rs/rune/0/rune/alloc/
[`TryClone` trait]: https://docs.rs/rune/0/rune/alloc/clone/trait.TryClone.html
