use rune::alloc;
use rune::runtime::{Mut, Ref};
use rune::sync::Arc;
use rune::termcolor::{ColorChoice, StandardStream};
use rune::{Any, Context, Diagnostics, FromValue, Vm};

#[derive(Any, Debug, Default)]
struct MyBytes {
    #[allow(unused)]
    bytes: Vec<u8>,
}

#[derive(FromValue)]
struct Proxy {
    field: Mut<alloc::String>,
    my_bytes: Ref<MyBytes>,
}

fn main() -> rune::support::Result<()> {
    let context = Context::with_default_modules()?;
    let runtime = Arc::try_new(context.runtime()?)?;

    let mut sources = rune::sources! {
        entry => {
            pub fn passthrough(my_bytes) {
                #{field: String::from("hello world"), my_bytes}
            }
        }
    };

    let mut diagnostics = Diagnostics::new();

    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.emit(&mut writer, &sources)?;
    }

    let unit = result?;
    let unit = Arc::try_new(unit)?;
    let mut vm = Vm::new(runtime, unit);

    let input = MyBytes {
        bytes: vec![77, 77, 77, 77],
    };
    let output = vm.execute(["passthrough"], (input,))?.complete()?;
    let mut output: Proxy = rune::from_value(output)?;

    println!("field: {:?}", output.field);
    println!("my_bytes: {:?}", output.my_bytes);
    output.field.clear();
    Ok(())
}
