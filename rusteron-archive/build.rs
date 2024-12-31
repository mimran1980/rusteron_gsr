use bindgen::EnumVariation;
use cmake::Config;
use dunce::canonicalize;
use log::info;
use proc_macro2::TokenStream;
use regex::Regex;
use rusteron_code_gen::{append_to_file, format_with_rustfmt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{env, fs};

#[derive(Eq, PartialEq)]
pub enum LinkType {
    Dynamic,
    Static,
}

impl LinkType {
    fn detect() -> LinkType {
        if cfg!(feature = "static") {
            LinkType::Static
        } else {
            LinkType::Dynamic
        }
    }

    fn link_lib(&self) -> &'static str {
        match self {
            LinkType::Dynamic => "dylib=",
            LinkType::Static => "static=",
        }
    }

    fn target_name(&self) -> &'static str {
        match self {
            LinkType::Dynamic => "aeron_archive_c_client",
            LinkType::Static => "aeron_archive_c_client_static",
        }
    }
    fn target_name_base(&self) -> &'static str {
        match self {
            LinkType::Dynamic => "aeron",
            LinkType::Static => "aeron_static",
        }
    }
}

pub fn main() {
    update_gradle_if_git_is_missing();

    let aeron_path = canonicalize(Path::new("./aeron")).unwrap();
    let header_path = aeron_path.join("aeron-archive/src/main/c");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    run_gradle_build_if_missing(&aeron_path);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=bindings.h");

    if pkg_config::probe_library("uuid").is_err() {
        eprintln!("uuid lib not found in path");
    }

    let link_type = LinkType::detect();
    println!(
        "cargo:rustc-link-lib={}{}",
        link_type.link_lib(),
        link_type.target_name()
    );
    println!(
        "cargo:rustc-link-lib={}{}",
        link_type.link_lib(),
        link_type.target_name_base()
    );

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-arg=-Wl,--allow-multiple-definition");
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-arg=/FORCE:MULTIPLE");
    }

    if let LinkType::Static = link_type {
        // On Windows, there are some extra libraries needed for static link
        // that aren't included by Aeron.
        if cfg!(target_os = "windows") {
            println!("cargo:rustc-link-lib=shell32");
            println!("cargo:rustc-link-lib=iphlpapi");
        }
        if cfg!(target_os = "linux") {
            println!("cargo:rustc-link-lib=uuid");
        }
    }

    let mut config = Config::new(&aeron_path);
    if std::env::var("PROFILE").unwrap() == "release" {
        config.profile("Release");
        config.define(
            "CMAKE_CXX_FLAGS_RELEASE",
            "-O3 -DNDEBUG -march=native -funroll-loops -flto",
        );
        config.define(
            "CMAKE_C_FLAGS_RELEASE",
            "-O3 -DNDEBUG -march=native -funroll-loops -flto",
        );
    } else {
        config.profile("Debug");
    }
    let cmake_output = config
        .define("CMAKE_C_FLAGS", "-fcommon")
        .define("BUILD_AERON_DRIVER", "OFF")
        .define("BUILD_AERON_ARCHIVE_API", "ON")
        // needed for mac os
        .define("CMAKE_OSX_DEPLOYMENT_TARGET", "14.0")
        .define("AERON_TESTS", "OFF")
        .define("AERON_BUILD_SAMPLES", "OFF")
        .define("AERON_BUILD_DOCUMENTATION", "OFF")
        .define("BUILD_SHARED_LIBS", "ON")
        .build_target(link_type.target_name())
        .build();

    // Trying to figure out the final path is a bit weird;
    // For Linux/OSX, it's just build/lib
    // For Windows, the .lib file is in build/lib/{profile}, but the DLL
    // is shipped in build/binaries/{profile}
    let base_lib_dir = cmake_output.join("build");
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("lib").display()
    );
    // Because the `cmake_output` path is different for debug/release, we're not worried
    // about accidentally linking the Debug library when this is a release build or vice-versa
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("lib/Debug").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("binaries/Debug").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("lib/Release").display()
    );
    println!(
        "cargo:rustc-link-search=native={}",
        base_lib_dir.join("binaries/Release").display()
    );

    println!("cargo:include={}", header_path.display());
    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}", header_path.display()))
        // We need to include some of the headers from `aeron c client`, so update the include path here
        .clang_arg(format!(
            "-I{}",
            aeron_path.join("aeron-client/src/main/c").display()
        ))
        .header("bindings.h")
        .allowlist_function("aeron_.*")
        .allowlist_type("aeron_.*")
        .allowlist_var("AERON_.*")
        .rustified_enum("aeron_.*_enum")
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .derive_debug(true)
        .generate()
        .expect("Unable to generate aeron_archive bindings");

    let out = out_path.join("bindings.rs");
    bindings
        .write_to_file(out.clone())
        .expect("Couldn't write bindings!");

    let bindings = rusteron_code_gen::parse_bindings(&out);
    let aeron = out_path.join("aeron.rs");
    let _ = fs::remove_file(aeron.clone());

    // include custom aeron code
    let aeron_custom = out_path.join("aeron_custom.rs");
    // let rb_custom = out_path.join("rb_custom.rs");
    let _ = fs::remove_file(aeron_custom.clone());
    // let _ = fs::remove_file(rb_custom.clone());
    append_to_file(
        aeron_custom.to_str().unwrap(),
        rusteron_code_gen::CUSTOM_AERON_CODE,
    )
    .unwrap();
    // append_to_file(
    //     rb_custom.to_str().unwrap(),
    //     rusteron_code_gen::CUSTOM_RB_CODE,
    // )
    // .unwrap();

    let mut stream = TokenStream::new();
    for (p, w) in bindings.wrappers.values().enumerate() {
        let code =
            rusteron_code_gen::generate_rust_code(w, &bindings.wrappers, p == 0, false, true);
        stream.extend(code);
    }
    for handler in &bindings.handlers {
        let code = rusteron_code_gen::generate_handlers(handler, &bindings);
        stream.extend(code);
    }
    append_to_file(
        aeron.to_str().unwrap(),
        &format_with_rustfmt(&stream.to_string()).unwrap(),
    )
    .unwrap();

    if std::env::var("COPY_BINDINGS").is_ok() {
        copy_binds(out);
    }
}

fn run_gradle_build_if_missing(aeron_path: &PathBuf) {
    if !aeron_path
        .join("aeron-all")
        .join("build")
        .join("libs")
        .exists()
    {
        let path = std::path::MAIN_SEPARATOR;
        let gradle = if cfg!(target_os = "windows") {
            &format!("{}{path}aeron{path}gradlew.bat", env!("CARGO_MANIFEST_DIR"),)
        } else {
            "./gradlew"
        };
        let dir = format!("{}{path}aeron", env!("CARGO_MANIFEST_DIR"),);
        info!("running {} in {}", gradle, dir);

        Command::new(gradle)
            .current_dir(dir)
            .args([
                ":aeron-agent:jar",
                ":aeron-samples:jar",
                ":aeron-archive:jar",
                ":aeron-all:jar",
                ":buildSrc:jar",
            ])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn().expect("failed to run gradle, which is required to build aeron-archive c lib. Please refer to wiki page regarding build setup")
            .wait().expect("gradle returned an error");
    }
    println!("cargo:rerun-if-changed=aeron/aeron-all/build/libs");
}

/// crates.io will exclude .git directory when publishing but aeron gradle build will fail as it
/// uses the .git directory to set version/hash for project
fn update_gradle_if_git_is_missing() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let aeron_git_dir = Path::new(manifest_dir).join("aeron/.git");
    let aeron_build_gradle = Path::new(manifest_dir).join("aeron/build.gradle");

    if !aeron_git_dir.exists() {
        println!("Aeron .git directory not found. Updating build.gradle with a dummy hash.");

        let gradle_content =
            fs::read_to_string(&aeron_build_gradle).expect("Failed to read aeron/build.gradle");

        // Replace `gitCommitHash` with a dummy hash as function fails
        let mut updated_gradle_content = gradle_content.replace(
            r#"def gitCommitHash = io.aeron.build.GithubUtil.currentGitHash("${projectDir}")"#,
            r#"def gitCommitHash = "dummy-hash""#,
        );

        // for some reason io.aeron plugins don't work, we don't actually need them so they get removed
        // ALL of this effort just because crates.io removes .git directory !!!!!
        let patterns = vec![
            // Remove dedupJar task block
            r"(?s)tasks\.register\('dedupJar',\s*io\.aeron\.build\.DeduplicateTask\)\s*\{.*?\}",
            // Remove shadowJar.finalizedBy dedupJar
            r"shadowJar\.finalizedBy\s+dedupJar",
            // Remove asciidoctorGithub task block
            r"(?s)tasks\.register\('asciidoctorGithub',\s*io\.aeron\.build\.AsciidoctorPreprocessTask\)\s*\{.*?\}",
            // Remove tutorialPublish task block
            r"(?s)tasks\.register\('tutorialPublish',\s*io\.aeron\.build\.TutorialPublishTask\)\s*\{.*?\}",
        ];

        for pattern in patterns {
            let re = Regex::new(pattern).expect("Invalid regex pattern");
            updated_gradle_content = re.replace_all(&updated_gradle_content, "").to_string();
        }

        fs::write(&aeron_build_gradle, updated_gradle_content)
            .expect("Failed to write updated aeron/build.gradle");
    }

    println!("cargo:rerun-if-changed=aeron/.git");
    println!("cargo:rerun-if-changed=aeron/build.gradle");
}

fn copy_binds(out: PathBuf) {
    let cargo_base_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let custom_bindings_path = cargo_base_dir.join("../rusteron-code-gen/bindings/archive.rs");

    if custom_bindings_path.exists() {
        fs::copy(out.clone(), custom_bindings_path.clone())
            .expect("Failed to override bindings.rs with custom bindings from archive.rs");
    } else {
        eprintln!(
            "Warning: Custom bindings not found at: {}",
            custom_bindings_path.display()
        );
    }
}
