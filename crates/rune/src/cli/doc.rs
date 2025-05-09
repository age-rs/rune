use std::io::Write;
use std::path::PathBuf;

use crate::doc::Artifacts;

use anyhow::{Context, Result};

use crate::alloc::prelude::*;
use crate::cli::naming::Naming;
use crate::cli::{AssetKind, CommandBase, Config, Entry, EntryPoint, ExitCode, Io, SharedFlags};
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
        /// Output directory to write documentation to.
        #[arg(long)]
        pub(super) output: Option<PathBuf>,
        /// Open the generated documentation in a browser.
        #[arg(long)]
        pub(super) open: bool,
        /// Explicit paths to format.
        pub(super) doc_path: Vec<PathBuf>,
    }
}

pub(super) use cli::Flags;

impl CommandBase for Flags {
    #[inline]
    fn is_workspace(&self, _: AssetKind) -> bool {
        true
    }

    #[inline]
    fn describe(&self) -> &str {
        "Documenting"
    }

    #[inline]
    fn paths(&self) -> &[PathBuf] {
        &self.doc_path
    }
}

pub(super) fn run<'p, I>(
    io: &mut Io<'_>,
    entry: &mut Entry<'_>,
    c: &Config,
    flags: &Flags,
    shared: &SharedFlags,
    options: &Options,
    entries: I,
) -> Result<ExitCode>
where
    I: IntoIterator<Item = EntryPoint<'p>>,
{
    let root = match &flags.output {
        Some(root) => root.clone(),
        None => match &c.manifest_root {
            Some(path) => path.join("target").join("rune-doc"),
            None => match std::env::var_os("CARGO_TARGET_DIR") {
                Some(target) => {
                    let mut target = PathBuf::from(target);
                    target.push("rune-doc");
                    target
                }
                None => {
                    let mut target = PathBuf::new();
                    target.push("target");
                    target.push("rune-doc");
                    target
                }
            },
        },
    };

    writeln!(io.stdout, "Building documentation: {}", root.display())?;

    let context = shared.context(entry, c, None)?;

    let mut visitors = Vec::new();

    let mut naming = Naming::default();

    for e in entries {
        let mut options = options.clone();

        if e.is_argument() {
            options.script = true;
        }

        let item = naming.item(&e)?;

        let mut visitor = crate::doc::Visitor::new(&item)?;
        let mut sources = Sources::new();

        let source = match Source::from_path(e.path()) {
            Ok(source) => source,
            Err(error) => return Err(error).context(e.path().display().try_to_string()?),
        };

        sources.insert(source)?;

        let mut diagnostics = if shared.warnings || flags.warnings_are_errors {
            Diagnostics::new()
        } else {
            Diagnostics::without_warnings()
        };

        let mut source_loader = FileSourceLoader::new();

        let _ = crate::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .with_options(&options)
            .with_visitor(&mut visitor)?
            .with_source_loader(&mut source_loader)
            .build();

        diagnostics.emit(&mut io.stdout.lock(), &sources)?;

        if diagnostics.has_error() || flags.warnings_are_errors && diagnostics.has_warning() {
            return Ok(ExitCode::Failure);
        }

        visitors.try_push(visitor)?;
    }

    let mut artifacts = Artifacts::new();

    crate::doc::build("root", &mut artifacts, Some(&context), &visitors)?;

    for asset in artifacts.assets() {
        asset.build(&root)?;
    }

    if flags.open {
        let path = root.join("index.html");
        let _ = webbrowser::open(&path.display().try_to_string()?);
    }

    Ok(ExitCode::Success)
}
