//! Writes the bindings Pamba's Windows layer holds for what Microsoft's reactor does not reach.
//!
//! Each set of bindings is written by Microsoft's own generator from metadata held here by its
//! SHA-256, through a filter file beside this crate's manifest that names the members bound, into
//! one file of `crates/pamba-windows/src`, which is committed. Whoever moves what a set's metadata
//! comes from, or changes what is called, runs this from anywhere in the workspace with the set's
//! name and the path of each of its metadata files:
//!
//! ```text
//! cargo run -p bind -- xaml <dir>/Microsoft.UI.Xaml.winmd <dir>/Microsoft.UI.winmd <dir>/Microsoft.Foundation.winmd
//! cargo run -p bind -- document <path of Microsoft.Web.WebView2.Core.winmd>
//! ```
//!
//! The sets:
//!
//! - `xaml`: a few members of XAML itself, for what arrives in answer to be brought into view and
//!   said. Its metadata is `crates/tools/reactor-metadata/winmd/` of Microsoft's repository at the
//!   commit of `windows-reactor` that `Cargo.lock` names, which is the metadata the reactor's own
//!   bindings are written from. Cargo's checkout of that commit holds it. The set moves with the
//!   lock: a move of the reactor's commit changes these digests, and this set is written again.
//! - `document`: the web view's core, for a document shown inert and printed. Its metadata is
//!   `lib/Microsoft.Web.WebView2.Core.winmd` in the NuGet package `Microsoft.Web.WebView2` at the
//!   version `pamba-windows-build` stages, and the set moves with that package.
//!
//! A file whose SHA-256 is not the one held is refused, so a set is written from the metadata
//! that was read and from no other.
//!
//! Exits 0 once the set is written; 1 where a file cannot be read or is not the one held, or where
//! the module the set is written into is not on this line of the tree; and 2 on a refused
//! invocation. The generator panics on a filter or metadata it cannot take, which fails the run.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::{env, fs};

use sha2::{Digest as _, Sha256};

const REFUSED_INVOCATION: u8 = 2;

/// One set of bindings.
struct Set {
    /// The name it is asked for by.
    name: &'static str,
    /// Each metadata file it is written from, by its file name, with the SHA-256 held for it.
    metadata: &'static [(&'static str, &'static str)],
    /// The filter file beside this crate's manifest.
    filter: &'static str,
    /// The file written, from the workspace's root.
    output: &'static str,
}

const SETS: &[Set] = &[
    Set {
        name: "xaml",
        // Microsoft's repository at 63a5e3d9b9afbd57c382d0a4f5a367f134e9dc25, the lock's commit.
        metadata: &[
            (
                "Microsoft.UI.Xaml.winmd",
                "a1ad041e172cfd82492964200d1361dd0652b17ba0a81209cab6f9953f958145",
            ),
            (
                "Microsoft.UI.winmd",
                "f70326853a3244cbdc647e690e538d1769752136df8548dacb2e0f8c804327b1",
            ),
            (
                "Microsoft.Foundation.winmd",
                "df2b61da1aa3b078e5c3c5ead57f08a6bcd941e16a8a5a19dbf7e89c715211d8",
            ),
        ],
        filter: "xaml.txt",
        output: "crates/pamba-windows/src/xaml/bindings.rs",
    },
    Set {
        name: "document",
        // `Microsoft.Web.WebView2` 1.0.4078.44, the package `pamba-windows-build` holds to its
        // own digest.
        metadata: &[(
            "Microsoft.Web.WebView2.Core.winmd",
            "f78ad0cc0b72fd111d680cc0bce215d6bd2a332c9f37a5f9b3726289d92241d6",
        )],
        filter: "document.txt",
        output: "crates/pamba-windows/src/document/bindings.rs",
    },
];

fn usage() -> ExitCode {
    eprintln!("usage: bind <set> <path of each of its metadata files>");
    for set in SETS {
        let files: Vec<&str> = set.metadata.iter().map(|(file, _)| *file).collect();
        eprintln!("  bind {} {}", set.name, files.join(" "));
    }
    ExitCode::from(REFUSED_INVOCATION)
}

fn main() -> ExitCode {
    let mut arguments = env::args_os().skip(1);
    let Some(named) = arguments.next() else {
        return usage();
    };
    let Some(set) = SETS.iter().find(|set| named == set.name) else {
        eprintln!("no set is named `{}`", named.display());
        return usage();
    };
    let given: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
    if given.len() != set.metadata.len() {
        return usage();
    }
    let mut inputs = Vec::with_capacity(given.len());
    for (file, held) in set.metadata {
        let Some(path) = given
            .iter()
            .find(|path| path.file_name().is_some_and(|name| name == *file))
        else {
            eprintln!(
                "the set `{}` is written from `{file}`, which was not given",
                set.name
            );
            return usage();
        };
        if let Err(refused) = held_file(path, held) {
            eprintln!("{refused}");
            return ExitCode::FAILURE;
        }
        inputs.push(path);
    }
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = here.join("../..").join(set.output);
    if !output.parent().is_some_and(Path::is_dir) {
        eprintln!(
            "`{}` is not on this line of the tree, so the set `{}` is not written here",
            set.output, set.name
        );
        return ExitCode::FAILURE;
    }
    windows_bindgen::builder()
        .inputs(&inputs)
        .input_default()
        .output(&output)
        .flat()
        .filter_file(here.join(set.filter))
        .write();
    println!("wrote {}", output.display());
    ExitCode::SUCCESS
}

/// Refuses `path` where it cannot be read or its SHA-256 is not `held`.
fn held_file(path: &Path, held: &str) -> Result<(), String> {
    let bytes = fs::read(path)
        .map_err(|error| format!("`{}` could not be read: {error}", path.display()))?;
    let found = hex(&Sha256::digest(&bytes));
    if found == held {
        return Ok(());
    }
    Err(format!(
        "`{}` is not the metadata these bindings are written from: its SHA-256 is {found}, and \
         the one held is {held}. Metadata of another version is taken by a change to this tool \
         that holds its digest.",
        path.display()
    ))
}

/// `bytes` in lower-case hexadecimal.
fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut text, byte| {
            // Writing to a `String` cannot fail.
            let _ = write!(text, "{byte:02x}");
            text
        })
}
