extern crate cfg_if;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(windows, feature = "neon-sys"))] {
        use std::env::var;
        use std::path::Path;

        const NODE_ROOT_DIR: &'static str = include_str!(concat!(env!("OUT_DIR"), "\\node_root_dir"));
        const NODE_ARCH:     &'static str = include_str!(concat!(env!("OUT_DIR"), "\\node_arch"));
        const NODE_LIB_FILE: &'static str = include_str!(concat!(env!("OUT_DIR"), "\\node_lib_file"));

        /// Set up the build environment by setting Cargo configuration variables.
        pub fn setup() {
            let debug = var("DEBUG").ok().map_or(false, |s| s == "true");
            let configuration = if debug { "Debug" } else { "Release" };
            let node_lib_file_path = Path::new(NODE_LIB_FILE);
            let mut node_lib_path = Path::new(NODE_ROOT_DIR).to_path_buf();
            node_lib_path.push(NODE_ARCH);
            println!("cargo:rustc-link-search={}\\{}", NODE_ROOT_DIR, configuration);
            println!("cargo:rustc-link-search=native={}", &node_lib_path.display());
            println!("cargo:rustc-link-lib={}", &node_lib_file_path.file_stem().unwrap().to_str().unwrap());

            // Link `win_delay_load_hook.obj` for windows electron
            let node_runtime_env = "npm_config_runtime";
            println!("cargo:rerun-if-env-changed={}", node_runtime_env);
            if var(node_runtime_env).map(|s| s == "electron") == Ok(true) {
                println!("cargo:rustc-cdylib-link-arg=win_delay_load_hook.obj");
                println!("cargo:rustc-cdylib-link-arg=delayimp.lib");
                println!("cargo:rustc-cdylib-link-arg=/DELAYLOAD:node.exe");
            }
        }
    } else if #[cfg(windows)] {
        // ^ automatically not neon-sys
        use std::io::{Error, ErrorKind, Write, Result};
        use std::process::Command;
        use std::path::Path;

        fn node_version() -> Result<String> {
            let output = Command::new("node").arg("-v").output()?;
            if !output.status.success() {
                let _ = std::io::stderr().write_all(&output.stderr);
                panic!("Could not download node.lib. There is likely more information from stderr above.");
            }
            let stdout = String::from_utf8(output.stdout).map_err(|error| {
                Error::new(ErrorKind::InvalidData, error)
            })?;
            // npm_config_target should not contain a leading "v"
            Ok(stdout.trim().trim_start_matches('v').to_string())
        }

        fn download_node_lib(version: &str, arch: &str) -> Result<Vec<u8>> {
            // Assume we're building for node if a disturl is not specified.
            let dist_url = std::env::var("NPM_CONFIG_DISTURL").unwrap_or("https://nodejs.org/dist".into());
            let script = r#"
                const https = require("https");
                var url = process.argv[1];
                function download(url) {
                    https.get(url, function (res) {
                        if(res.statusCode === 302) {
                            return download(res.headers.location);
                        } else if(res.statusCode === 200) {
                            res.pipe(process.stdout);
                        } else {
                            console.error("Invalid HTTP Status: " + res.statusCode);
                        }
                    });
                }
                download(url);
            "#;
            let url = format!("{dist_url}/v{version}/win-{arch}/node.lib", dist_url = dist_url, version = version, arch = arch);

            let output = Command::new("node")
                .arg("-e").arg(script)
                .arg(url)
                .output()?;
                
            // If the stderr was used, assume an error occured
            if !output.stderr.is_empty() {
                // Pack unknown error string into an io error lossily and return
                return Err(Error::new(std::io::ErrorKind::Other, String::from_utf8_lossy(&output.stderr)));
            }

            Ok(output.stdout)
        }

        /// Set up the build environment by setting Cargo configuration variables.
        pub fn setup() {
            // If the user specified a node.lib path, we do not need to download
            if let Some(node_lib_path) = std::env::var_os("NEON_NODE_LIB") {
                let node_lib_path = Path::new(&node_lib_path);
                // Clearing the file name returns the root+directory name
                let dir = node_lib_path.with_file_name("");
                let basename = node_lib_path.file_stem().expect("Could not parse lib name from NEON_NODE_LIB. Does the path include the full file name?");

                println!("cargo:rustc-link-search=native={}", dir.display());
                // `basename` is an OsStr, we can output it anyway by re-wrapping it in a Path
                // Both `dir` and `basename` will be mangled (contain replacement characters) if
                // they are not UTF-8 paths. If we don't mangle them though, Cargo will: so
                // non-UTF-8 paths are simply not supported.
                println!("cargo:rustc-link-lib={}", Path::new(basename).display());
                return;
            }

            let version = std::env::var("npm_config_target")
                .or_else(|_| node_version())
                .expect("Could not determine Node.js version");
            let arch = if std::env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "x86" {
                "x86"
            } else {
                "x64"
            };

            let node_lib_store_path = format!(r"{}/node-{}.lib", env!("OUT_DIR"), arch);

            // Download node.lib if it does not exist
            if let Err(_) = std::fs::metadata(&node_lib_store_path) {
                let node_lib = download_node_lib(&version, arch).expect("Could not download `node.lib`");
                std::fs::write(&node_lib_store_path, &node_lib).expect("Could not save `node.lib`");
            }

            println!("cargo:rustc-link-search=native={}", env!("OUT_DIR"));
            println!("cargo:rustc-link-lib=node-{}", arch);
            println!("cargo:rustc-cdylib-link-arg=delayimp.lib");
            println!("cargo:rustc-cdylib-link-arg=/DELAYLOAD:node.exe");
        }
    } else if #[cfg(target_os = "macos")] {
        /// Set up the build environment by setting Cargo configuration variables.
        pub fn setup() {
            println!("cargo:rustc-cdylib-link-arg=-undefined");
            println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
        }
    } else {
        pub fn setup() { }
    }
}
