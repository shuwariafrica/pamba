use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
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

/// Every entry under `dir` by its path from `dir`, a file with its bytes and a directory with
/// none.
fn tree(dir: &Path) -> BTreeMap<PathBuf, Option<Vec<u8>>> {
    let mut entries = BTreeMap::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in fs::read_dir(&next).unwrap() {
            let path = entry.unwrap().path();
            let relative = path.strip_prefix(dir).unwrap().to_path_buf();
            if path.is_dir() {
                entries.insert(relative, None);
                pending.push(path);
            } else {
                entries.insert(relative, Some(fs::read(&path).unwrap()));
            }
        }
    }
    entries
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

    fn unpack(&self, archive: &Path, into: &Path) -> Result<(), Unstaged> {
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
const NEEDED: &str = "tools/MSIX/win10-x64/Microsoft.WindowsAppRuntime.2.msix";
const TEXT: &str = "license.txt";
/// A stand-in package as its download unpacks: the file the staging needs and the licence text
/// at its root.
const PACKAGE: [&str; 2] = [NEEDED, TEXT];

fn held(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn package(sha256: &str) -> Package<'_> {
    Package {
        name: "Test.Package",
        version: "1.0.0",
        sha256,
        texts: &[TEXT],
    }
}

/// The package's directory in the cache, as the web view's staging takes it.
fn found(cache: &Path, package: &Package<'_>, tools: &Fake) -> Result<PathBuf, Unstaged> {
    cached(cache, package, Path::new(NEEDED), tools, |dir| {
        Ok(dir.to_path_buf())
    })
}

#[test]
fn a_package_whose_bytes_are_not_the_ones_held_is_refused_and_nothing_of_it_is_unpacked() {
    let cache = TempDir::new();
    let digest = held(b"other bytes");
    let tools = Fake::new(BYTES, PACKAGE.to_vec());

    let refused = found(&cache.0, &package(&digest), &tools);

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
    let tools = Fake::new(BYTES, PACKAGE.to_vec());

    let first = found(&cache.0, &package(&digest), &tools).unwrap();
    assert_eq!(tools.downloads.get(), 1);
    let second = found(&cache.0, &package(&digest), &tools);

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
    let tools = Fake::new(BYTES, PACKAGE.to_vec());

    let dir = found(&cache.0, &package(&digest), &tools).unwrap();

    assert_eq!(dir, cache.0.join(&digest));
    assert_eq!(names(&cache.0), BTreeSet::from([digest]));
}

#[test]
fn a_download_that_did_not_arrive_leaves_nothing_behind() {
    let cache = TempDir::new();
    let digest = held(BYTES);
    let mut tools = Fake::new(BYTES, PACKAGE.to_vec());
    tools.arrives = false;

    let refused = found(&cache.0, &package(&digest), &tools);

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
    let mut tools = Fake::new(BYTES, PACKAGE.to_vec());
    tools.unpacks = false;

    let refused = found(&cache.0, &package(&digest), &tools);

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

    let refused = msix_unpacked(&RUNTIME, &runtime.0, msix, "x64", &tools);

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

/// A build of `counter` for x64 with MSVC, with its `LOCALAPPDATA` and its target folder in a
/// directory of its own.
struct Build {
    temp: TempDir,
    local: PathBuf,
    out: PathBuf,
}

impl Build {
    fn new() -> Self {
        let temp = TempDir::new();
        let local = temp.0.join("local");
        let out = temp.0.join("target/debug/build/counter-0123/out");
        fs::create_dir_all(&local).unwrap();
        fs::create_dir_all(&out).unwrap();
        Self { temp, local, out }
    }

    fn vars(&self, target: &'static str) -> impl Fn(&str) -> Option<OsString> {
        let (local, out) = (self.local.clone(), self.out.clone());
        move |name: &str| match name {
            "CARGO_CFG_TARGET_OS" => Some(OsString::from(target)),
            "CARGO_CFG_TARGET_ENV" => Some(OsString::from("msvc")),
            "CARGO_CFG_TARGET_ARCH" => Some(OsString::from("x86_64")),
            "OUT_DIR" => Some(out.clone().into_os_string()),
            "LOCALAPPDATA" => Some(local.clone().into_os_string()),
            _ => None,
        }
    }

    /// The profile's folder, where the executable is written.
    fn beside(&self) -> PathBuf {
        self.temp.0.join("target/debug")
    }

    fn packages(&self) -> PathBuf {
        self.local.join("pamba/packages")
    }

    /// Lays the two packages in the cache as their downloads unpack for x64, each file holding
    /// its package's name and its own path, so the staging finds them and downloads nothing.
    fn lay_packages(&self) {
        let laid = [
            (
                &RUNTIME,
                [
                    "tools/MSIX/win10-x64/Microsoft.WindowsAppRuntime.2.msix",
                    "tools/MSIX/win10-x64/Microsoft.WindowsAppRuntime.Main.2.msix",
                    "license.txt",
                    "NOTICE.txt",
                    "Microsoft.WindowsAppSDK.Runtime.nuspec",
                    "[Content_Types].xml",
                ],
            ),
            (
                &WEBVIEW2,
                [
                    "runtimes/win-x64/native_uap/Microsoft.Web.WebView2.Core.dll",
                    "runtimes/win-x64/native/WebView2Loader.dll",
                    "LICENSE.txt",
                    "NOTICE.txt",
                    "Microsoft.Web.WebView2.nuspec",
                    "[Content_Types].xml",
                ],
            ),
        ];
        for (package, files) in laid {
            for file in files {
                write(
                    &self.packages().join(package.sha256).join(file),
                    &format!("{} {file}", package.name),
                );
            }
        }
    }
}

/// The runtime's inner package as its unpacking writes it.
const MSIX: [&str; 5] = [
    "Microsoft.WindowsAppRuntime.dll",
    "Microsoft.ui.xaml.dll",
    "Microsoft.UI.Xaml/Assets/NoiseAsset_256x256_PNG.png",
    "AppxManifest.xml",
    "[Content_Types].xml",
];

#[test]
fn it_stands_aside_where_the_target_or_the_building_machine_is_not_windows() {
    let build = Build::new();
    let tools = Fake::new(BYTES, PACKAGE.to_vec());

    let off_windows = stage(Host::Other, &build.vars("windows"), &tools);
    let for_linux = stage(Host::Windows, &build.vars("linux"), &tools);

    assert_eq!(tools.downloads.get(), 0, "it fetched");
    assert_eq!(
        names(&build.local),
        BTreeSet::new(),
        "it wrote into the cache's parent"
    );
    assert_eq!(names(&build.out), BTreeSet::new(), "it wrote into OUT_DIR");
    assert_eq!(names(&build.beside()), BTreeSet::from(["build".to_owned()]));
    assert_eq!(off_windows.unwrap(), Vec::<String>::new());
    assert_eq!(for_linux.unwrap(), Vec::<String>::new());
}

#[test]
fn the_three_licence_texts_stand_in_licenses_beside_the_executable_as_their_packages_hold_them() {
    let build = Build::new();
    build.lay_packages();
    let tools = Fake::new(BYTES, MSIX.to_vec());

    stage(Host::Windows, &build.vars("windows"), &tools).unwrap();

    let licenses = build.beside().join("licenses");
    assert_eq!(
        names(&licenses),
        BTreeSet::from(
            [
                "Microsoft.WindowsAppSDK.Runtime.license.txt",
                "Microsoft.WindowsAppSDK.Runtime.NOTICE.txt",
                "Microsoft.Web.WebView2.LICENSE.txt",
            ]
            .map(str::to_owned)
        )
    );
    for (package, text) in [
        (&RUNTIME, "license.txt"),
        (&RUNTIME, "NOTICE.txt"),
        (&WEBVIEW2, "LICENSE.txt"),
    ] {
        assert_eq!(
            fs::read(licenses.join(format!("{}.{text}", package.name))).unwrap(),
            fs::read(build.packages().join(package.sha256).join(text)).unwrap(),
            "{} {text}",
            package.name
        );
    }
}

#[test]
fn beside_the_executable_stands_nothing_of_either_package_but_the_files_it_stages() {
    let build = Build::new();
    build.lay_packages();
    let tools = Fake::new(BYTES, MSIX.to_vec());

    stage(Host::Windows, &build.vars("windows"), &tools).unwrap();

    assert_eq!(
        names(&build.beside()),
        BTreeSet::from(
            [
                ".pamba-windows-build.lock",
                "Microsoft.UI.Xaml",
                "Microsoft.Web.WebView2.Core.dll",
                "Microsoft.WindowsAppRuntime.dll",
                "Microsoft.ui.xaml.dll",
                "build",
                "licenses",
            ]
            .map(str::to_owned)
        )
    );
}

#[test]
fn the_cache_is_asked_for_under_packages_and_an_earlier_pambas_folder_is_left_as_it_was() {
    let build = Build::new();
    build.lay_packages();
    let earlier = build.local.join("pamba/windows-app-runtime");
    for file in [
        format!(
            "{}/MSIX/win10-x64/Microsoft.WindowsAppRuntime.2.msix",
            RUNTIME.sha256
        ),
        format!(
            "{}/.msix-x64/Microsoft.WindowsAppRuntime.dll",
            RUNTIME.sha256
        ),
        format!("{}.lock", RUNTIME.sha256),
        format!(
            "{}/win-x64/native_uap/Microsoft.Web.WebView2.Core.dll",
            WEBVIEW2.sha256
        ),
    ] {
        write(&earlier.join(&file), &file);
    }
    let before = tree(&earlier);
    let tools = Fake::new(BYTES, MSIX.to_vec());

    let staged = stage(Host::Windows, &build.vars("windows"), &tools);

    assert_eq!(tree(&earlier), before, "the earlier folder changed");
    assert_eq!(tools.downloads.get(), 0, "it fetched");
    assert!(staged.is_ok(), "{staged:?}");
}

#[test]
fn a_package_without_one_of_its_licence_texts_is_refused_and_nothing_is_left_under_its_digest() {
    let digest = held(BYTES);
    for (real, needed, missing) in [
        (RUNTIME, NEEDED, "license.txt"),
        (RUNTIME, NEEDED, "NOTICE.txt"),
        (
            WEBVIEW2,
            "runtimes/win-x64/native_uap/Microsoft.Web.WebView2.Core.dll",
            "LICENSE.txt",
        ),
    ] {
        let cache = TempDir::new();
        let package = Package {
            sha256: &digest,
            ..real
        };
        let entries = std::iter::once(needed)
            .chain(real.texts.iter().copied())
            .filter(|entry| *entry != missing)
            .collect();
        let tools = Fake::new(BYTES, entries);

        let refused = cached(&cache.0, &package, Path::new(needed), &tools, |dir| {
            Ok(dir.to_path_buf())
        });

        let Err(Unstaged::MissingFile { package, file }) = refused else {
            panic!("without {missing}, not refused as missing a file: {refused:?}");
        };
        assert_eq!(package, real.to_string());
        assert_eq!(file, Path::new(missing));
        assert!(
            !cache.0.join(&digest).exists(),
            "without {missing}, a directory named by the digest is left"
        );
    }
}

#[test]
fn a_package_directory_lacking_a_file_is_removed_and_filled_again_once() {
    let digest = held(BYTES);
    let damaged = || {
        let cache = TempDir::new();
        write(&cache.0.join(&digest).join(NEEDED), "the inner package");
        write(&cache.0.join(&digest).join("stale"), "left by the damage");
        cache
    };

    let healed_cache = damaged();
    let whole = Fake::new(BYTES, PACKAGE.to_vec());
    let healed = found(&healed_cache.0, &package(&digest), &whole);
    assert!(healed.is_ok(), "not filled again: {healed:?}");
    assert_eq!(whole.downloads.get(), 1);
    assert_eq!(
        names(&healed_cache.0.join(&digest)),
        BTreeSet::from([TEXT, "tools"].map(str::to_owned)),
        "not removed whole"
    );

    let refused_cache = damaged();
    let lacking = Fake::new(BYTES, vec![NEEDED]);
    let refused = found(&refused_cache.0, &package(&digest), &lacking);
    assert!(
        matches!(&refused, Err(Unstaged::MissingFile { file, .. }) if file == Path::new(TEXT)),
        "{refused:?}"
    );
    assert_eq!(
        lacking.downloads.get(),
        1,
        "not downloaded exactly once more"
    );
}

/// The runtime's inner package in the cache, as the runtime's staging takes it.
fn found_inner(cache: &Path, package: &Package<'_>, tools: &Fake) -> Result<PathBuf, Unstaged> {
    cached(cache, package, Path::new(NEEDED), tools, |dir| {
        msix_unpacked(package, dir, Path::new(NEEDED), "x64", tools)
    })
}

#[test]
fn a_package_whose_inner_package_lacks_a_file_is_removed_and_filled_again_once() {
    let digest = held(BYTES);
    let damaged = || {
        let cache = TempDir::new();
        for file in PACKAGE {
            write(&cache.0.join(&digest).join(file), file);
        }
        write(
            &cache
                .0
                .join(&digest)
                .join(".msix-x64/Microsoft.ui.xaml.dll"),
            "left by the damage",
        );
        cache
    };

    let healed_cache = damaged();
    let whole = Fake::new(
        BYTES,
        vec![
            NEEDED,
            TEXT,
            "Microsoft.WindowsAppRuntime.dll",
            "Microsoft.ui.xaml.dll",
        ],
    );
    let healed = found_inner(&healed_cache.0, &package(&digest), &whole);
    assert!(healed.is_ok(), "not filled again: {healed:?}");
    assert_eq!(whole.downloads.get(), 1);
    assert!(
        healed_cache
            .0
            .join(&digest)
            .join(".msix-x64/Microsoft.WindowsAppRuntime.dll")
            .is_file()
    );

    let refused_cache = damaged();
    let lacking = Fake::new(BYTES, vec![NEEDED, TEXT, "Microsoft.ui.xaml.dll"]);
    let refused = found_inner(&refused_cache.0, &package(&digest), &lacking);
    assert!(
        matches!(
            &refused,
            Err(Unstaged::MissingFile { file, .. })
                if file == Path::new("Microsoft.WindowsAppRuntime.dll")
        ),
        "{refused:?}"
    );
    assert_eq!(
        lacking.downloads.get(),
        1,
        "not downloaded exactly once more"
    );
}

#[test]
fn each_package_is_asked_for_at_nugets_content_address_in_lower_case() {
    assert_eq!(
        RUNTIME.address(),
        "https://api.nuget.org/v3-flatcontainer/microsoft.windowsappsdk.runtime/2.6.4/\
         microsoft.windowsappsdk.runtime.2.6.4.nupkg"
    );
    assert_eq!(
        WEBVIEW2.address(),
        "https://api.nuget.org/v3-flatcontainer/microsoft.web.webview2/1.0.4078.44/\
         microsoft.web.webview2.1.0.4078.44.nupkg"
    );
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
