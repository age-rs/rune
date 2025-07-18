use rune::sync::Arc;
use rune::termcolor::{ColorChoice, StandardStream};
use rune::{ContextError, Diagnostics, Module, Vm};

#[rune::function(instance)]
fn divide_by_three(value: i64) -> i64 {
    value / 3
}

#[tokio::main]
async fn main() -> rune::support::Result<()> {
    let m = module()?;

    let mut context = rune_modules::default_context()?;
    context.install(m)?;
    let runtime = Arc::try_new(context.runtime()?)?;

    let mut sources = rune::sources!(entry => {
        pub fn main(number) {
            number.divide_by_three()
        }
    });

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

    let output = vm.execute(["main"], (33i64,))?.complete()?;
    let output: i64 = rune::from_value(output)?;

    println!("output: {output}");
    Ok(())
}

fn module() -> Result<Module, ContextError> {
    let mut m = Module::with_item(["mymodule"])?;
    m.function_meta(divide_by_three)?;
    Ok(m)
}
