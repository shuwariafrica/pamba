use std::cell::Cell;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

/// A directory of its own under the system's temporary folder, removed when the case ends.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = env::temp_dir().join(format!(
            "pamba-windows-build-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn names(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect()
}

/// Stands in for `curl.exe` and `tar.exe`: a download writes `bytes`, and an unpacking writes
/// `entries` into the directory it is given.
struct Fake {
    bytes: &'static [u8],
    arrives: bool,
    entries: Vec<&'static str>,
    unpacks: bool,
    downloads: Cell<usize>,
    unpackings: Cell<usize>,
}

impl Fake {
    fn new(bytes: &'static [u8], entries: Vec<&'static str>) -> Self {
        Self {
            bytes,
            arrives: true,
            entries,
            unpacks: true,
            downloads: Cell::new(0),
            unpackings: Cell::new(0),
        }
    }
}

impl Tools for Fake {
    fn download(&self, package: &Package<'_>, to: &Path) -> Result<(), Unstaged> {
        self.downloads.set(self.downloads.get() + 1);
        if self.arrives {
            fs::write(to, self.bytes).unwrap();
            Ok(())
        } else {
            fs::write(to, self.bytes.get(..self.bytes.len() / 2).unwrap()).unwrap();
            Err(Unstaged::NotDownloaded {
                package: package.to_string(),
                address: package.address(),
                code: Some(18),
                words: "curl: (18) transfer closed with bytes remaining to read".to_owned(),
            })
        }
    }

    fn unpack(&self, archive: &Path, into: &Path, _strip: Strip) -> Result<(), Unstaged> {
        self.unpackings.set(self.unpackings.get() + 1);
        assert!(
            archive.is_file(),
            "unpacked {} where no archive stands",
            archive.display()
        );
        for entry in &self.entries {
            write(&into.join(entry), entry);
        }
        if self.unpacks {
            Ok(())
        } else {
            Err(Unstaged::Tool {
                tool: PathBuf::from("tar.exe"),
                failure: ToolFailure::Failed {
                    code: Some(1),
                    words: "tar.exe: Error opening archive: Unrecognized archive format".to_owned(),
                },
            })
        }
    }
}

const BYTES: &[u8] = b"the package's bytes";
const NEEDED: &str = "MSIX/win10-x64/Microsoft.WindowsAppRuntime.2.msix";

fn held(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn package(sha256: &str) -> Package<'_> {
    Package {
        name: "Test.Package",
        version: "1.0.0",
        sha256,
    }
}

#[test]
fn a_package_whose_bytes_are_not_the_ones_held_is_refused_and_nothing_of_it_is_unpacked() {
    let cache = TempDir::new();
    let digest = held(b"other bytes");
    let tools = Fake::new(BYTES, vec![NEEDED]);

    let refused = unpacked(&cache.0, &package(&digest), Path::new(NEEDED), &tools);

    let Err(Unstaged::NotHeld {
        package,
        held,
        found,
        ..
    }) = refused
    else {
        panic!("not refused as bytes not held: {refused:?}");
    };
    assert_eq!(package, "Test.Package 1.0.0");
    assert_eq!(held, digest);
    assert_eq!(found, self::held(BYTES));
    assert_eq!(tools.unpackings.get(), 0, "bytes not held were unpacked");
    assert_eq!(
        names(&cache.0),
        BTreeSet::new(),
        "the cache keeps something"
    );
}

#[test]
fn a_package_whose_bytes_are_the_ones_held_is_unpacked_once_and_found_again() {
    let cache = TempDir::new();
    let digest = held(BYTES);
    let tools = Fake::new(BYTES, vec![NEEDED]);

    let first = unpacked(&cache.0, &package(&digest), Path::new(NEEDED), &tools).unwrap();
    assert_eq!(tools.downloads.get(), 1);
    let second = unpacked(&cache.0, &package(&digest), Path::new(NEEDED), &tools);

    assert_eq!(tools.downloads.get(), 1, "the second call fetched again");
    assert_eq!(second.unwrap(), first);
    assert_eq!(first, cache.0.join(&digest));
    assert!(first.join(NEEDED).is_file());
    assert_eq!(names(&cache.0), BTreeSet::from([digest]));
}

#[test]
fn what_a_stopped_build_left_for_the_package_is_removed_before_it_is_fetched() {
    let cache = TempDir::new();
    let digest = held(BYTES);
    write(
        &cache.0.join(format!("{digest}.nupkg-1")),
        "a download cut short",
    );
    write(
        &cache.0.join(format!("{digest}.partial-1")).join(NEEDED),
        "an unpacking cut short",
    );
    let tools = Fake::new(BYTES, vec![NEEDED]);

    let dir = unpacked(&cache.0, &package(&digest), Path::new(NEEDED), &tools).unwrap();

    assert_eq!(dir, cache.0.join(&digest));
    assert_eq!(names(&cache.0), BTreeSet::from([digest]));
}

#[test]
fn a_download_that_did_not_arrive_leaves_nothing_behind() {
    let cache = TempDir::new();
    let digest = held(BYTES);
    let mut tools = Fake::new(BYTES, vec![NEEDED]);
    tools.arrives = false;

    let refused = unpacked(&cache.0, &package(&digest), Path::new(NEEDED), &tools);

    assert!(
        matches!(refused, Err(Unstaged::NotDownloaded { code: Some(18), .. })),
        "{refused:?}"
    );
    assert_eq!(
        names(&cache.0),
        BTreeSet::new(),
        "the cache keeps something"
    );
}

#[test]
fn an_unpacking_that_fails_leaves_no_directory_named_by_the_digest() {
    let cache = TempDir::new();
    let digest = held(BYTES);
    let mut tools = Fake::new(BYTES, vec![NEEDED]);
    tools.unpacks = false;

    let refused = unpacked(&cache.0, &package(&digest), Path::new(NEEDED), &tools);

    assert!(
        matches!(
            refused,
            Err(Unstaged::Tool {
                failure: ToolFailure::Failed { .. },
                ..
            })
        ),
        "{refused:?}"
    );
    let partial = format!("{digest}.partial-");
    for name in names(&cache.0) {
        assert!(name.starts_with(&partial), "the cache holds {name}");
    }
}

#[test]
fn only_what_the_runtime_list_names_is_staged() {
    let temp = TempDir::new();
    let msix = temp.0.join("msix");
    let beside = temp.0.join("beside");
    for entry in [
        "Microsoft.UI.Xaml.dll",
        "Microsoft.UI.Xaml/Assets/NoiseAsset_256x256_PNG.png",
        "AF-ZA/Microsoft.ui.xaml.dll.mui",
        "AppxManifest.xml",
        "[Content_Types].xml",
        "Unnamed/Microsoft.UI.Xaml.dll",
    ] {
        write(&msix.join(entry), entry);
    }
    fs::create_dir(&beside).unwrap();

    copy_named(&msix, &beside).unwrap();

    assert_eq!(
        names(&beside),
        BTreeSet::from(["AF-ZA", "Microsoft.UI.Xaml", "Microsoft.UI.Xaml.dll"].map(str::to_owned))
    );
    assert_eq!(
        fs::read_to_string(beside.join("Microsoft.UI.Xaml/Assets/NoiseAsset_256x256_PNG.png"))
            .unwrap(),
        "Microsoft.UI.Xaml/Assets/NoiseAsset_256x256_PNG.png"
    );
    assert!(beside.join("AF-ZA/Microsoft.ui.xaml.dll.mui").is_file());
}

#[test]
fn a_package_without_a_file_the_staging_needs_is_refused() {
    let runtime = TempDir::new();
    let msix = Path::new(NEEDED);
    write(&runtime.0.join(msix), "the inner package");
    let tools = Fake::new(BYTES, vec!["Microsoft.ui.xaml.dll"]);

    let refused = msix_unpacked(&runtime.0, msix, "x64", &tools);

    let Err(Unstaged::MissingFile { package, file }) = refused else {
        panic!("not refused as missing a file: {refused:?}");
    };
    assert_eq!(package, "Microsoft.WindowsAppSDK.Runtime 2.6.4");
    assert_eq!(file, Path::new("Microsoft.WindowsAppRuntime.dll"));
    assert!(!runtime.0.join(".msix-x64").exists());
}

#[test]
fn the_manifest_carries_the_mark_and_the_linker_is_told_to_embed_it_for_each_toolchain() {
    let manifest = marked(APP_MANIFEST);
    let mark = format!("<description>{SELF_CONTAINED_MARK}</description>");
    let assembly = manifest.find("<assembly").unwrap();
    let opening = assembly + manifest[assembly..].find('>').unwrap() + 1;
    assert!(
        manifest[opening..].starts_with(&mark),
        "the mark is not the first thing inside <assembly>"
    );
    assert_eq!(manifest.matches(&mark).count(), 1);
    assert_eq!(manifest.replacen(&mark, "", 1), APP_MANIFEST);

    let path = Path::new("out").join("app.manifest");
    assert_eq!(
        link_args(toolchain("msvc", "").unwrap(), &path),
        [
            "cargo:rustc-link-arg-bins=/MANIFEST:EMBED".to_owned(),
            format!(
                "cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}",
                path.display()
            ),
        ]
    );
    assert_eq!(
        link_args(toolchain("gnu", "llvm").unwrap(), &path),
        [
            "cargo:rustc-link-arg-bins=-Wl,/MANIFEST:EMBED".to_owned(),
            format!(
                "cargo:rustc-link-arg-bins=-Wl,/MANIFESTINPUT:{}",
                path.display()
            ),
        ]
    );
    let refused = toolchain("gnu", "");
    assert!(
        matches!(&refused, Err(Unstaged::UnsupportedToolchain { env, abi }) if env == "gnu" && abi.is_empty()),
        "{refused:?}"
    );
}

#[test]
fn the_two_assets_are_the_pinned_commits() {
    assert_eq!(
        held(RUNTIME_FILES.as_bytes()),
        "26b2ca391f260ffcf00c083747f407a34836ae41abce3af9fa707cea107741a4"
    );
    assert_eq!(
        held(APP_MANIFEST.as_bytes()),
        "40c2a9b7df0e255d5a578052d2ef63d6d26c7dead3429775b20cd17eafad7673"
    );
}

#[test]
fn it_stands_aside_where_the_target_or_the_building_machine_is_not_windows() {
    let temp = TempDir::new();
    let local = temp.0.join("local");
    let out = temp.0.join("target/debug/build/counter-0123/out");
    fs::create_dir_all(&local).unwrap();
    fs::create_dir_all(&out).unwrap();
    let tools = Fake::new(BYTES, vec![NEEDED]);
    let build = |target: &'static str| {
        let (local, out) = (local.clone(), out.clone());
        move |name: &str| match name {
            "CARGO_CFG_TARGET_OS" => Some(OsString::from(target)),
            "CARGO_CFG_TARGET_ENV" => Some(OsString::from("msvc")),
            "CARGO_CFG_TARGET_ARCH" => Some(OsString::from("x86_64")),
            "OUT_DIR" => Some(out.clone().into_os_string()),
            "LOCALAPPDATA" => Some(local.clone().into_os_string()),
            _ => None,
        }
    };

    let off_windows = stage(Host::Other, &build("windows"), &tools);
    let for_linux = stage(Host::Windows, &build("linux"), &tools);

    assert_eq!(tools.downloads.get(), 0, "it fetched");
    assert_eq!(
        names(&local),
        BTreeSet::new(),
        "it wrote into the cache's parent"
    );
    assert_eq!(names(&out), BTreeSet::new(), "it wrote into OUT_DIR");
    assert_eq!(
        names(&temp.0.join("target/debug")),
        BTreeSet::from(["build".to_owned()])
    );
    assert_eq!(off_windows.unwrap(), Vec::<String>::new());
    assert_eq!(for_linux.unwrap(), Vec::<String>::new());
}

#[test]
fn the_folder_beside_the_executable_is_found_from_either_form_cargo_gives_out_dir() {
    let root = Path::new("release").join("target");
    for profile in [
        root.join("profiling"),
        root.join("aarch64-pc-windows-msvc").join("debug"),
    ] {
        for package in [
            Path::new("counter-0123"),
            &Path::new("counter").join("0123"),
        ] {
            let out = profile.join("build").join(package).join("out");
            assert_eq!(
                beside_executable(&out),
                Some(profile.clone()),
                "{}",
                out.display()
            );
        }
    }
    let other = root.join("profiling").join("counter").join("out");
    assert_eq!(beside_executable(&other), None);
}
