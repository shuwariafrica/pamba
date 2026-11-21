//! What the build script of a program on Pamba's Windows layer calls.
//!
//! ```no_run
//! // build.rs
//! pamba_windows_build::self_contained();
//! ```

#![forbid(unsafe_code)]

use std::env;
use std::ffi::{OsStr, OsString};
use std::fmt::{self, Write as _};
use std::fs;
use std::io::{self, Read as _};
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use sha2::{Digest as _, Sha256};

// The two packages, the two assets and the mark are those of `windows-reactor-setup` at the
// windows-rs commit `Cargo.lock` holds for `windows-reactor`, and all of them move together with
// that commit. The assets are Microsoft's, offered under MIT or Apache-2.0, and Pamba takes them
// under Apache-2.0; both notices are carried beside them in `assets/`.
const RUNTIME: Package<'static> = Package {
    name: "Microsoft.WindowsAppSDK.Runtime",
    version: "2.6.4",
    sha256: "3e6d1230df782973d09174cf874c947ee5fbbbe77198c52750771f71a085aa0c",
};
const WEBVIEW2: Package<'static> = Package {
    name: "Microsoft.Web.WebView2",
    version: "1.0.4078.44",
    sha256: "dc4d1d9168df26b830398303e50210b6e1729f6ce5a7ac69d2c766852f489962",
};
const RUNTIME_FILES: &str = include_str!("../assets/runtime.txt");
const APP_MANIFEST: &str = include_str!("../assets/app.manifest");
// `windows-reactor` looks for this text in the executable's manifest to load the runtime from
// beside the executable.
const SELF_CONTAINED_MARK: &str = "windows-reactor-self-contained";

const WEBVIEW2_CORE_DLL: &str = "Microsoft.Web.WebView2.Core.dll";
const MSIX_FILES: [&str; 2] = ["Microsoft.WindowsAppRuntime.dll", "Microsoft.ui.xaml.dll"];

/// Stages the Windows App Runtime beside the program's executable and marks the executable to
/// load it from there, so the program starts on a Windows computer that has no Windows App Runtime
/// installed.
///
/// Each of the runtime's two packages is downloaded from nuget.org and unpacked only where the
/// SHA-256 of the whole download is the one this version of Pamba holds, which defends the build
/// against the network and against the publisher replacing a package. What has been unpacked is
/// kept in `%LOCALAPPDATA%\pamba\windows-app-runtime\`, one directory named by each digest, and
/// a later build takes it from there without downloading: that directory is trusted as the
/// building person's own.
///
/// The folder beside the executable is found from `OUT_DIR`, the one place Cargo names to a build
/// script. A build whose `build.build-dir` is set apart from its target directory therefore has
/// the runtime staged in the build directory's profile folder, where the executable is not: leave
/// `build.build-dir` unset for a program built this way.
///
/// It does nothing where the target is not Windows, so a program's build script calls it on
/// every platform its workspace is checked on. It does nothing either where the machine that
/// builds is not Windows, which cannot stage the runtime: there the Windows target is checked,
/// and a program built there starts only where the runtime is installed.
///
/// # Panics
///
/// Where the runtime cannot be staged, which fails the build with the reason: Cargo's or
/// Windows' environment lacks a variable, the target's toolchain is neither MSVC nor LLVM's GNU,
/// a download does not arrive or its bytes are not the ones held, a package lacks a file the
/// staging needs, or a tool or an act on the file system fails.
#[expect(
    clippy::panic,
    reason = "a build script fails its build by panicking, and Cargo shows the message"
)]
pub fn self_contained() {
    let host = if cfg!(windows) {
        Host::Windows
    } else {
        Host::Other
    };
    match stage(host, &|name| env::var_os(name), &System) {
        Ok(directives) => directives
            .iter()
            .for_each(|directive| println!("{directive}")),
        Err(unstaged) => panic!("{unstaged}"),
    }
}

/// The machine the build script runs on, which is not the target's.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Host {
    Windows,
    Other,
}

/// A package on nuget.org, held to the SHA-256 of its `.nupkg`.
#[derive(Clone, Copy, Debug)]
struct Package<'a> {
    name: &'a str,
    version: &'a str,
    /// In lower-case hexadecimal.
    sha256: &'a str,
}

impl Package<'_> {
    fn address(&self) -> String {
        format!(
            "https://www.nuget.org/api/v2/package/{}/{}",
            self.name, self.version
        )
    }
}

impl fmt::Display for Package<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

/// How many leading components of each entry an unpacking drops.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Strip {
    None,
    /// A `.nupkg`'s first, so its `tools\MSIX\...` stands as `MSIX\...`.
    First,
}

/// The two acts of the staging that reach outside the build: Windows' own `curl.exe` and
/// `tar.exe` in a build, and a stand-in in the cases.
trait Tools {
    /// Writes the bytes at the package's address to the file `to`.
    fn download(&self, package: &Package<'_>, to: &Path) -> Result<(), Unstaged>;
    /// Unpacks the zip archive `archive` into the existing directory `into`.
    fn unpack(&self, archive: &Path, into: &Path, strip: Strip) -> Result<(), Unstaged>;
}

/// Why the runtime was not staged.
#[derive(Debug)]
enum Unstaged {
    /// Cargo or Windows sets no such variable for the build.
    NoVariable(&'static str),
    /// The variable is set, but not as text.
    VariableNotText(&'static str),
    /// The target's toolchain is neither MSVC nor LLVM's GNU, whose linkers embed the manifest.
    UnsupportedToolchain {
        env: String,
        abi: String,
    },
    /// `OUT_DIR` is in neither form Cargo gives it, so the folder beside the executable is not
    /// known.
    OutDirForm(PathBuf),
    Tool {
        tool: PathBuf,
        failure: ToolFailure,
    },
    NotDownloaded {
        package: String,
        address: String,
        code: Option<i32>,
        words: String,
    },
    NotHeld {
        package: String,
        address: String,
        held: String,
        found: String,
    },
    MissingFile {
        package: String,
        file: PathBuf,
    },
    File {
        act: &'static str,
        path: PathBuf,
        error: io::Error,
    },
    Copy {
        from: PathBuf,
        to: PathBuf,
        error: io::Error,
    },
}

#[derive(Debug)]
enum ToolFailure {
    NotRun(io::Error),
    Failed { code: Option<i32>, words: String },
}

impl fmt::Display for Unstaged {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoVariable(name) => write!(f, "The build's environment does not set `{name}`."),
            Self::VariableNotText(name) => {
                write!(f, "The build's environment sets `{name}`, but not as text.")
            }
            Self::UnsupportedToolchain { env, abi } => write!(
                f,
                "The target's toolchain (environment `{env}`, ABI `{abi}`) is neither MSVC nor \
                 LLVM's GNU, so the runtime's manifest cannot be embedded in the executable."
            ),
            Self::OutDirForm(out) => write!(
                f,
                "`OUT_DIR` is `{}`, which is in neither form Cargo gives it, so the folder \
                 beside the executable is not known.",
                out.display()
            ),
            Self::Tool {
                tool,
                failure: ToolFailure::NotRun(error),
            } => write!(f, "`{}` could not be run: {error}", tool.display()),
            Self::Tool {
                tool,
                failure: ToolFailure::Failed { code, words },
            } => write!(
                f,
                "`{}` failed with {}: {words}",
                tool.display(),
                Status(*code)
            ),
            Self::NotDownloaded {
                package,
                address,
                code,
                words,
            } => write!(
                f,
                "`{package}` did not arrive from `{address}`: `curl.exe` failed with {}: {words}",
                Status(*code)
            ),
            Self::NotHeld {
                package,
                address,
                held,
                found,
            } => write!(
                f,
                "`{package}`, downloaded from `{address}`, is not the package this version of \
                 Pamba holds. Its SHA-256 is `{found}`; the one held is `{held}`. Nothing of it \
                 was unpacked and the download was removed. A download cut short or changed on \
                 the way gives this, and so does a package its publisher has replaced. A newer \
                 runtime is taken by a change to Pamba that fixes its digest."
            ),
            Self::MissingFile { package, file } => write!(
                f,
                "`{package}` holds no `{}`, which the staging needs.",
                file.display()
            ),
            Self::File { act, path, error } => {
                write!(f, "Could not {act} `{}`: {error}", path.display())
            }
            Self::Copy { from, to, error } => write!(
                f,
                "Could not copy `{}` to `{}`: {error}",
                from.display(),
                to.display()
            ),
        }
    }
}

/// A tool's exit status, as its exit code where it has one.
struct Status(Option<i32>);

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(code) => write!(f, "exit code {code}"),
            None => f.write_str("no exit code"),
        }
    }
}

/// The toolchains whose linkers take `/MANIFEST:EMBED`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Toolchain {
    Msvc,
    LlvmGnu,
}

/// Stages the runtime for the build `vars` describes, and gives the directives for Cargo.
fn stage(
    host: Host,
    vars: &impl Fn(&str) -> Option<OsString>,
    tools: &impl Tools,
) -> Result<Vec<String>, Unstaged> {
    if text(vars, "CARGO_CFG_TARGET_OS")? != "windows" || host != Host::Windows {
        return Ok(Vec::new());
    }
    let abi = match vars("CARGO_CFG_TARGET_ABI") {
        None => String::new(),
        Some(abi) => abi
            .into_string()
            .map_err(|_| Unstaged::VariableNotText("CARGO_CFG_TARGET_ABI"))?,
    };
    let toolchain = toolchain(&text(vars, "CARGO_CFG_TARGET_ENV")?, &abi)?;
    let arch = match text(vars, "CARGO_CFG_TARGET_ARCH")?.as_str() {
        "aarch64" => "arm64",
        "x86" => "x86",
        _ => "x64",
    };
    let out_dir = PathBuf::from(vars("OUT_DIR").ok_or(Unstaged::NoVariable("OUT_DIR"))?);
    let beside =
        beside_executable(&out_dir).ok_or_else(|| Unstaged::OutDirForm(out_dir.clone()))?;
    let cache = PathBuf::from(vars("LOCALAPPDATA").ok_or(Unstaged::NoVariable("LOCALAPPDATA"))?)
        .join("pamba")
        .join("windows-app-runtime");
    create_dir_all(&cache)?;

    let runtime_msix = Path::new("MSIX")
        .join(format!("win10-{arch}"))
        .join("Microsoft.WindowsAppRuntime.2.msix");
    let msix = {
        let _lock = lock(&cache, &RUNTIME)?;
        let runtime = unpacked(&cache, &RUNTIME, &runtime_msix, tools)?;
        msix_unpacked(&runtime, &runtime_msix, arch, tools)?
    };
    let webview2_core = Path::new(&format!("win-{arch}"))
        .join("native_uap")
        .join(WEBVIEW2_CORE_DLL);
    let webview2 = {
        let _lock = lock(&cache, &WEBVIEW2)?;
        unpacked(&cache, &WEBVIEW2, &webview2_core, tools)?
    };

    create_dir_all(&beside)?;
    {
        let _lock = lock_file(&beside.join(".pamba-windows-build.lock"))?;
        copy_named(&msix, &beside)?;
        copy_entry(
            &webview2.join(&webview2_core),
            &beside.join(WEBVIEW2_CORE_DLL),
        )?;
    }

    let manifest = out_dir.join("app.manifest");
    fs::write(&manifest, marked(APP_MANIFEST)).map_err(|error| Unstaged::File {
        act: "write",
        path: manifest.clone(),
        error,
    })?;
    Ok(link_args(toolchain, &manifest).into())
}

fn text(vars: &impl Fn(&str) -> Option<OsString>, name: &'static str) -> Result<String, Unstaged> {
    vars(name)
        .ok_or(Unstaged::NoVariable(name))?
        .into_string()
        .map_err(|_| Unstaged::VariableNotText(name))
}

fn toolchain(env: &str, abi: &str) -> Result<Toolchain, Unstaged> {
    match (env, abi) {
        ("msvc", _) => Ok(Toolchain::Msvc),
        ("gnu", "llvm") => Ok(Toolchain::LlvmGnu),
        _ => Err(Unstaged::UnsupportedToolchain {
            env: env.to_owned(),
            abi: abi.to_owned(),
        }),
    }
}

/// The profile's folder, where the executable is written, from `OUT_DIR` in either of Cargo's
/// forms, `<profile>/build/<package>-<hash>/out` and `<profile>/build/<package>/<hash>/out`.
/// `PROFILE` is not read, since it says `debug` or `release` for a custom profile too.
fn beside_executable(out_dir: &Path) -> Option<PathBuf> {
    if out_dir.file_name() != Some(OsStr::new("out")) {
        return None;
    }
    [3, 2]
        .into_iter()
        .filter_map(|depth| out_dir.ancestors().nth(depth))
        .find(|path| path.file_name() == Some(OsStr::new("build")))
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

/// The directory named by the package's digest, holding the whole of its `.nupkg` unpacked: found
/// where it stands, otherwise downloaded, verified and unpacked. The caller holds the package's
/// lock.
fn unpacked(
    cache: &Path,
    package: &Package<'_>,
    needed: &Path,
    tools: &impl Tools,
) -> Result<PathBuf, Unstaged> {
    let dir = cache.join(package.sha256);
    if !is_dir(&dir)? {
        let downloads = format!("{}.nupkg-", package.sha256);
        remove_left(cache, &downloads)?;
        let download = cache.join(format!("{downloads}{}", process::id()));
        verified(package, &download, tools)?;
        let whole = whole(cache, package.sha256, |partial| {
            tools.unpack(&download, partial, Strip::First)?;
            require(package, partial, needed)
        });
        discard(&download)?;
        whole?;
    }
    require(package, &dir, needed)?;
    Ok(dir)
}

/// Downloads the package to the file `to`, and leaves the file there only where its bytes are
/// the ones held.
fn verified(package: &Package<'_>, to: &Path, tools: &impl Tools) -> Result<(), Unstaged> {
    if let Err(unstaged) = tools.download(package, to) {
        discard(to)?;
        return Err(unstaged);
    }
    let found = match sha256_of(to) {
        Ok(found) => found,
        Err(unstaged) => {
            discard(to)?;
            return Err(unstaged);
        }
    };
    if found != package.sha256 {
        discard(to)?;
        return Err(Unstaged::NotHeld {
            package: package.to_string(),
            address: package.address(),
            held: package.sha256.to_owned(),
            found,
        });
    }
    Ok(())
}

/// The runtime's inner `.msix` for the architecture, unpacked into `<sha256>\.msix-<arch>\`. The
/// caller holds the runtime's lock.
fn msix_unpacked(
    runtime: &Path,
    msix: &Path,
    arch: &str,
    tools: &impl Tools,
) -> Result<PathBuf, Unstaged> {
    let name = format!(".msix-{arch}");
    let dir = runtime.join(&name);
    let require_all = |dir: &Path| {
        MSIX_FILES
            .iter()
            .try_for_each(|file| require(&RUNTIME, dir, Path::new(file)))
    };
    if !is_dir(&dir)? {
        whole(runtime, &name, |partial| {
            tools.unpack(&runtime.join(msix), partial, Strip::None)?;
            require_all(partial)
        })?;
    }
    require_all(&dir)?;
    Ok(dir)
}

/// Fills `parent\<name>.partial-<process id>` and renames it to `parent\<name>` once it is whole,
/// so a directory of that name is never part of a filling. A filling an earlier build left is
/// removed first; the caller holds the lock that keeps any other build from filling.
fn whole(
    parent: &Path,
    name: &str,
    fill: impl FnOnce(&Path) -> Result<(), Unstaged>,
) -> Result<PathBuf, Unstaged> {
    let prefix = format!("{name}.partial-");
    remove_left(parent, &prefix)?;
    let partial = parent.join(format!("{prefix}{}", process::id()));
    fs::create_dir(&partial).map_err(|error| file_error("create", &partial, error))?;
    if let Err(unstaged) = fill(&partial) {
        // The next build removes what is left where this fails, so its error would add nothing
        // to the one that stopped the filling.
        let _ = fs::remove_dir_all(&partial);
        return Err(unstaged);
    }
    let dir = parent.join(name);
    fs::rename(&partial, &dir).map_err(|error| file_error("rename into place", &dir, error))?;
    Ok(dir)
}

/// Removes every entry of `parent` whose name starts with `prefix`: what a build that was stopped
/// left there, under a name carrying its process id. The caller holds the lock that keeps any
/// other build from writing such an entry.
fn remove_left(parent: &Path, prefix: &str) -> Result<(), Unstaged> {
    for entry in fs::read_dir(parent).map_err(|error| file_error("read", parent, error))? {
        let entry = entry.map_err(|error| file_error("read", parent, error))?;
        if entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with(prefix))
        {
            let path = entry.path();
            let is_dir = entry
                .file_type()
                .map_err(|error| file_error("read", &path, error))?
                .is_dir();
            let removed = if is_dir {
                fs::remove_dir_all(&path)
            } else {
                fs::remove_file(&path)
            };
            removed.map_err(|error| file_error("remove", &path, error))?;
        }
    }
    Ok(())
}

fn require(package: &Package<'_>, dir: &Path, file: &Path) -> Result<(), Unstaged> {
    let missing = || Unstaged::MissingFile {
        package: package.to_string(),
        file: file.to_path_buf(),
    };
    match fs::metadata(dir.join(file)) {
        Ok(metadata) if metadata.is_file() => Ok(()),
        Ok(_) => Err(missing()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Err(missing()),
        Err(error) => Err(file_error("read", &dir.join(file), error)),
    }
}

fn sha256_of(path: &Path) -> Result<String, Unstaged> {
    let mut file = fs::File::open(path).map_err(|error| file_error("open", path, error))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0; 1 << 16];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| file_error("read", path, error))?;
        match buffer.get(..read) {
            Some(bytes) if !bytes.is_empty() => hasher.update(bytes),
            _ => break,
        }
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut hex, byte| {
            // Writing to a `String` does not fail.
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

/// Copies beside the executable every entry of the unpacked `.msix` that `runtime.txt` names,
/// without regard to case.
fn copy_named(msix: &Path, beside: &Path) -> Result<(), Unstaged> {
    for entry in fs::read_dir(msix).map_err(|error| file_error("read", msix, error))? {
        let entry = entry.map_err(|error| file_error("read", msix, error))?;
        let name = entry.file_name();
        if name.to_str().is_some_and(|name| {
            RUNTIME_FILES
                .lines()
                .any(|line| line.trim().eq_ignore_ascii_case(name))
        }) {
            copy_entry(&entry.path(), &beside.join(name))?;
        }
    }
    Ok(())
}

fn copy_entry(from: &Path, to: &Path) -> Result<(), Unstaged> {
    if is_dir(from)? {
        create_dir_all(to)?;
        for entry in fs::read_dir(from).map_err(|error| file_error("read", from, error))? {
            let entry = entry.map_err(|error| file_error("read", from, error))?;
            copy_entry(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        fs::copy(from, to)
            .map(|_| ())
            .map_err(|error| Unstaged::Copy {
                from: from.to_path_buf(),
                to: to.to_path_buf(),
                error,
            })
    }
}

/// The carried manifest with the self-contained mark as the first child of its `<assembly>`.
fn marked(manifest: &str) -> String {
    let opening = manifest
        .find("<assembly")
        .and_then(|start| Some(start + manifest.get(start..)?.find('>')? + 1));
    let Some(opening) = opening else {
        unreachable!("the carried app.manifest opens an <assembly> element")
    };
    let (head, tail) = manifest.split_at(opening);
    format!("{head}<description>{SELF_CONTAINED_MARK}</description>{tail}")
}

/// Cargo's directives telling the executable's linker to embed the manifest at `manifest`.
fn link_args(toolchain: Toolchain, manifest: &Path) -> [String; 2] {
    let prefix = match toolchain {
        Toolchain::Msvc => "",
        Toolchain::LlvmGnu => "-Wl,",
    };
    [
        format!("cargo:rustc-link-arg-bins={prefix}/MANIFEST:EMBED"),
        format!(
            "cargo:rustc-link-arg-bins={prefix}/MANIFESTINPUT:{}",
            manifest.display()
        ),
    ]
}

/// Holds the package's lock in the cache, so that of two builds at once one fetches and unpacks
/// the package and the other finds it.
fn lock(cache: &Path, package: &Package<'_>) -> Result<fs::File, Unstaged> {
    lock_file(&cache.join(format!("{}.lock", package.sha256)))
}

fn lock_file(path: &Path) -> Result<fs::File, Unstaged> {
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .map_err(|error| file_error("open", path, error))?;
    file.lock()
        .map_err(|error| file_error("lock", path, error))?;
    Ok(file)
}

fn is_dir(path: &Path) -> Result<bool, Unstaged> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(file_error("read", path, error)),
    }
}

fn create_dir_all(path: &Path) -> Result<(), Unstaged> {
    fs::create_dir_all(path).map_err(|error| file_error("create", path, error))
}

/// Removes the file where it stands.
fn discard(path: &Path) -> Result<(), Unstaged> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(file_error("remove", path, error)),
    }
}

fn file_error(act: &'static str, path: &Path, error: io::Error) -> Unstaged {
    Unstaged::File {
        act,
        path: path.to_path_buf(),
        error,
    }
}

/// Windows' own `curl.exe` and `tar.exe`, from `%SystemRoot%\System32`.
struct System;

impl System {
    fn tool(name: &str) -> Result<PathBuf, Unstaged> {
        env::var_os("SystemRoot")
            .map(|root| PathBuf::from(root).join("System32").join(name))
            .ok_or(Unstaged::NoVariable("SystemRoot"))
    }

    fn run(tool: &Path, command: &mut Command) -> Result<process::Output, Unstaged> {
        command.output().map_err(|error| Unstaged::Tool {
            tool: tool.to_path_buf(),
            failure: ToolFailure::NotRun(error),
        })
    }
}

impl Tools for System {
    fn download(&self, package: &Package<'_>, to: &Path) -> Result<(), Unstaged> {
        let curl = Self::tool("curl.exe")?;
        let address = package.address();
        println!("Downloading {package} from {address}");
        let output = Self::run(
            &curl,
            Command::new(&curl)
                .args([
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--location",
                    "--output",
                ])
                .arg(to)
                .arg(&address),
        )?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Unstaged::NotDownloaded {
                package: package.to_string(),
                address,
                code: output.status.code(),
                words: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            })
        }
    }

    fn unpack(&self, archive: &Path, into: &Path, strip: Strip) -> Result<(), Unstaged> {
        let tar = Self::tool("tar.exe")?;
        let mut command = Command::new(&tar);
        command.arg("-xf").arg(archive).arg("-C").arg(into);
        if strip == Strip::First {
            command.arg("--strip-components=1");
        }
        let output = Self::run(&tar, &mut command)?;
        if output.status.success() {
            Ok(())
        } else {
            Err(Unstaged::Tool {
                tool: tar,
                failure: ToolFailure::Failed {
                    code: output.status.code(),
                    words: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
                },
            })
        }
    }
}

#[cfg(test)]
mod tests;
