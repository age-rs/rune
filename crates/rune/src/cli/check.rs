use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::cli::{visitor, AssetKind, CommandBase, Config, Entry, ExitCode, Io, SharedFlags};
use crate::compile::FileSourceLoader;
use crate::{Diagnostics, Options, Source, Sources};

mod cli {
    use std::path::PathBuf;
    use std::vec::Vec;

    use clap::Parser;

    #[derive(Parser, Debug)]
    #[command(rename_all = "kebab-case")]
    pub(crate) struct Flags {
        /// Exit with a non-zero exit-code even for warnings
        #[arg(long)]
        pub(super) warnings_are_errors: bool,
        /// Explicit paths to check.
        pub(super) check_path: Vec<PathBuf>,
    }
}

pub(super) use cli::Flags;

impl CommandBase for Flags {
    #[inline]
    fn is_debug(&self) -> bool {
        true
    }

    #[inline]
    fn is_workspace(&self, _: AssetKind) -> bool {
        true
    }

    #[inline]
    fn describe(&self) -> &str {
        "Checking"
    }

    #[inline]
    fn paths(&self) -> &[PathBuf] {
        &self.check_path
    }
}

pub(super) fn run(
    io: &mut Io<'_>,
    entry: &mut Entry<'_>,
    c: &Config,
    flags: &Flags,
    shared: &SharedFlags,
    options: &Options,
    path: &Path,
) -> Result<ExitCode> {
    writeln!(io.stdout, "Checking: {}", path.display())?;

    let context = shared.context(entry, c, None)?;

    let source =
        Source::from_path(path).with_context(|| format!("reading file: {}", path.display()))?;

    let mut sources = Sources::new();

    sources.insert(source)?;

    let mut diagnostics = if shared.warnings || flags.warnings_are_errors {
        Diagnostics::new()
    } else {
        Diagnostics::without_warnings()
    };

    let mut test_finder = visitor::FunctionVisitor::new(visitor::Attribute::None);
    let mut source_loader = FileSourceLoader::new();

    let _ = crate::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .with_options(options)
        .with_visitor(&mut test_finder)?
        .with_source_loader(&mut source_loader)
        .build();

    diagnostics.emit(&mut io.stdout.lock(), &sources)?;

    if diagnostics.has_error() || flags.warnings_are_errors && diagnostics.has_warning() {
        Ok(ExitCode::Failure)
    } else {
        Ok(ExitCode::Success)
    }
}
