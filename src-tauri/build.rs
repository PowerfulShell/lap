/**
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    write_build_info();
    build_libraw();

    // build tauri
    tauri_build::build();
}

/// writes the build information to a file
fn write_build_info() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_info.rs");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = now.as_secs();

    let mut formatted = String::new();
    write!(
        &mut formatted,
        "pub const BUILD_UNIX_TIME: u64 = {};",
        timestamp
    )
    .unwrap();

    fs::write(dest_path, formatted).unwrap();
}

fn build_libraw() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        build_libraw_windows();
        return;
    }

    println!("cargo:rerun-if-changed=src/libraw_shim.cpp");
    println!("cargo:rerun-if-changed=third_party/LibRaw");
    println!("cargo:rerun-if-changed=third_party/libjpeg-turbo");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = out_dir_path();

    // Build libjpeg-turbo from submodule
    let jpeg_build = build_libjpeg(&manifest_dir, &out_dir);

    // Build LibRaw from submodule
    let libraw_source = manifest_dir.join("third_party/LibRaw");
    if !libraw_source.exists() {
        panic!(
            "LibRaw source not found at {}. Run: git submodule update --init --recursive",
            libraw_source.display()
        );
    }

    let build_root = out_dir.join("libraw-build");
    let build_dir = build_root.join("build");
    let static_lib = build_dir.join("lib/.libs/libraw_r.a");
    let config_log = build_dir.join("config.log");

    fs::create_dir_all(&build_dir).unwrap();

    // Copy source to build dir to avoid polluting submodule (configure generates files in-tree)
    let build_source = build_root.join("src");
    if !build_source.join("configure").exists() {
        if build_source.exists() {
            let _ = fs::remove_dir_all(&build_source);
        }
        copy_dir(&libraw_source, &build_source);
    }

    let needs_reconfigure = !static_lib.exists()
        || (jpeg_build.is_some()
            && fs::read_to_string(&config_log)
                .map(|contents| contents.contains("WARNING: libjpeg not found"))
                .unwrap_or(false));

    if needs_reconfigure {
        if build_source.join("Makefile").exists() {
            let _ = Command::new("make")
                .arg("distclean")
                .current_dir(&build_source)
                .status();
        }

        // Git source doesn't have pre-generated configure; generate it
        if !build_source.join("configure").exists() {
            run_command(
                Command::new("autoreconf")
                    .arg("-i")
                    .current_dir(&build_source),
                "generate LibRaw configure script (autoreconf -i)",
            );
        }

        let mut configure = Command::new("sh");
        configure
            .arg("./configure")
            .arg("--disable-shared")
            .arg("--enable-static")
            .arg("--disable-openmp")
            .arg("--disable-examples")
            .arg("--disable-lcms")
            .current_dir(&build_source);
        if let Some(jpeg) = &jpeg_build {
            let include_flags = jpeg
                .include_dirs
                .iter()
                .map(|dir| format!("-I{}", dir.display()))
                .collect::<Vec<_>>()
                .join(" ");
            let ld_flags = format!("-L{}", jpeg.lib_dir.display());
            configure.env("CPPFLAGS", include_flags);
            configure.env("LDFLAGS", ld_flags);
        }
        run_command(&mut configure, "configure LibRaw");

        let jobs = env::var("NUM_JOBS").unwrap_or_else(|_| "1".to_string());
        run_command(
            Command::new("make")
                .arg(format!("-j{jobs}"))
                .current_dir(&build_source),
            "build LibRaw",
        );
    }

    if let Ok(contents) = fs::read_to_string(&config_log) {
        if contents.contains("WARNING: libjpeg not found") {
            println!(
                "cargo:warning=LibRaw built without libjpeg; lossy DNG and some old Kodak RAW variants may have reduced support"
            );
        }
    }

    println!(
        "cargo:rustc-link-search=native={}",
        build_source.join("lib/.libs").display()
    );
    println!("cargo:rustc-link-lib=static=raw_r");
    if let Some(jpeg) = &jpeg_build {
        println!("cargo:rustc-link-search=native={}", jpeg.lib_dir.display());
        println!("cargo:rustc-link-lib=static=jpeg");
    }
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=m");
    match target_os.as_str() {
        "macos" => println!("cargo:rustc-link-lib=c++"),
        "linux" => println!("cargo:rustc-link-lib=stdc++"),
        _ => {}
    }

    cc::Build::new()
        .cpp(true)
        .flag_if_supported("-std=c++17")
        .include(&build_source)
        .include(build_source.join("libraw"))
        .file("src/libraw_shim.cpp")
        .compile("lap_libraw_shim");
}

fn build_libraw_windows() {
    println!("cargo:rerun-if-changed=src/libraw_shim.cpp");
    println!("cargo:rerun-if-changed=resources/libraw/LibRaw-0.22.0-Win64.zip");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let libraw_root = manifest_dir.join("resources/libraw/windows/LibRaw-0.22.0");
    let include_root = libraw_root.join("libraw");
    let lib_dir = libraw_root.join("lib");

    if !include_root.exists() || !lib_dir.join("libraw.lib").exists() {
        panic!(
            "LibRaw Windows package is not extracted. Expected {} and {}. Extract resources/libraw/LibRaw-0.22.0-Win64.zip to resources/libraw/windows before building on Windows.",
            include_root.display(),
            lib_dir.join("libraw.lib").display()
        );
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=libraw");

    cc::Build::new()
        .cpp(true)
        .flag_if_supported("/std:c++17")
        .include(&libraw_root)
        .include(&include_root)
        .file("src/libraw_shim.cpp")
        .compile("lap_libraw_shim");
}

struct JpegBuild {
    include_dirs: Vec<PathBuf>,
    lib_dir: PathBuf,
}

fn build_libjpeg(manifest_dir: &Path, out_dir: &Path) -> Option<JpegBuild> {
    let source_dir = manifest_dir.join("third_party/libjpeg-turbo");
    if !source_dir.exists() {
        println!(
            "cargo:warning=libjpeg-turbo submodule not found at {}. Run: git submodule update --init --recursive",
            source_dir.display()
        );
        return None;
    }

    let build_root = out_dir.join("libjpeg-build");
    let binary_dir = build_root.join("build");
    let static_lib = binary_dir.join("libjpeg.a");

    fs::create_dir_all(&binary_dir).unwrap();

    if !static_lib.exists() {
        run_command(
            Command::new("cmake")
                .arg("-G")
                .arg("Unix Makefiles")
                .arg("-DCMAKE_BUILD_TYPE=Release")
                .arg("-DENABLE_SHARED=FALSE")
                .arg("-DENABLE_STATIC=TRUE")
                .arg(source_dir.as_os_str())
                .current_dir(&binary_dir),
            "configure libjpeg-turbo",
        );

        let jobs = env::var("NUM_JOBS").unwrap_or_else(|_| "1".to_string());
        run_command(
            Command::new("cmake")
                .arg("--build")
                .arg(".")
                .arg("--target")
                .arg("jpeg-static")
                .arg("--parallel")
                .arg(jobs)
                .current_dir(&binary_dir),
            "build libjpeg-turbo",
        );
    }

    Some(JpegBuild {
        include_dirs: vec![binary_dir.clone(), source_dir],
        lib_dir: binary_dir,
    })
}

fn out_dir_path() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

fn run_command(command: &mut Command, description: &str) {
    let status = command
        .status()
        .unwrap_or_else(|e| panic!("Failed to {}: {}", description, e));
    if !status.success() {
        panic!("Failed to {}: exit status {}", description, status);
    }
}

/// Recursively copy a directory tree
fn copy_dir(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.file_name().map(|n| n == ".git").unwrap_or(false) {
            continue; // skip .git
        }

        if file_type.is_dir() {
            copy_dir(&src_path, &dst_path);
        } else if file_type.is_symlink() {
            let target = fs::read_link(&src_path).unwrap();
            #[cfg(unix)]
            std::os::unix::fs::symlink(target, &dst_path).ok();
        } else {
            fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}
