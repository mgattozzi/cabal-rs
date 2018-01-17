extern crate walkdir;
use walkdir::WalkDir;
use std::path::Path;
use std::process::Command;
use std::env;
use std::io::Error;
//use std::collections::HashMap;
//const PROG_NOT_SET: &str = "cabal-rs: Program was not set using `with_prog`";
const STATIC_EXTENSION: &str = ".a";

pub struct Cabal<T>
    where T: AsRef<Path>
{
    path: T,
    build_dir: Option<T>,
    verbose: Option<Verbosity>,
    jobs: Option<usize>,
    //w_prog: Option<HashMap<Prog, Vec<&'a str>>>,
    only: bool
}

pub enum Verbosity {
    Zero,
    One,
    Two,
    Three,
}

// #[allow(non_camel_case_types)]
// #[derive(PartialEq, Eq, Hash, Clone, Copy)]
// pub enum Prog {
//     alex,
//     ar,
//     c2hs,
//     cpphs,
//     doctest,
//     gcc,
//     ghc,
//     ghc_pkg,
//     ghcjs,
//     ghcjs_pkg,
//     greencard,
//     haddock,
//     happy,
//     haskell_suite,
//     haskell_suite_pkg,
//     hmake,
//     hpc,
//     hsc2hs,
//     hscolour,
//     jhc,
//     ld,
//     lhc,
//     lhc_pkg,
//     pkg_config,
//     runghc,
//     strip,
//     tar,
//     uhc
// }

impl<T> Cabal<T>
    where T: AsRef<Path>
{
    pub fn src(path: T) -> Self {
        Self {
            path,
            build_dir: None,
            verbose: None,
            jobs: None,
            //w_prog: None,
            only: false,
        }
    }

    pub fn build_dir(mut self, path: T) -> Self {
        self.build_dir = Some(path);
        self
    }

    pub fn jobs(mut self, j: usize) -> Self {
        self.jobs = Some(j);
        self
    }

    // pub fn with_prog(mut self, prog: Prog) -> Self {
    //     self.w_prog = Some(self.w_prog.map_or_else(
    //         || {
    //             let mut map = HashMap::new();
    //             map.insert(prog, Vec::new());
    //             map
    //         },
    //         |mut map| {
    //             map.insert(prog, Vec::new());
    //             map
    //         }
    //     ));
    //     self
    // }

    // /// ## Panic
    // /// This will panic if you have not set the program in the given argument
    // /// with the function `with_prog` prior to this function call.
    // pub fn prog_option(mut self, prog: Prog, option: &'a str) -> Self {
    //     let mut map = self.w_prog.expect(PROG_NOT_SET);
    //     {
    //         let opts = map.get_mut(&prog).expect(PROG_NOT_SET);
    //         opts.push(option);
    //     }
    //     self.w_prog = Some(map);
    //     self
    // }

    // /// ## Panic
    // /// This will panic if you have not set the program in the given argument
    // /// with the function `with_prog` prior to this function call.
    // pub fn prog_options(mut self, prog: Prog, options: Vec<&'a str>) -> Self {
    //     let mut map = self.w_prog.expect(PROG_NOT_SET);
    //     {
    //         let opts = map.get_mut(&prog).expect(PROG_NOT_SET);
    //         opts.extend(&options);
    //     }
    //     self.w_prog = Some(map);
    //     self
    // }

    pub fn verbose(mut self, v: Verbosity) -> Self {
        self.verbose = Some(v);
        self
    }

    pub fn build(self) -> Result<(), Error> {
        let mut options = Vec::new();

        if self.only {
            options.push("--only".into());
        }

        let mut build_dir = None;
        if let Some(dir) = self.build_dir {
            let mut directory = String::from("--builddir=");
            directory.push_str(&dir.as_ref().to_str().unwrap());
            build_dir = Some(dir);
            options.push(directory);
        }

        if let Some(v) = self.verbose {
            let mut verbose = String::from("--verbose=");
            verbose.push({
                match v {
                    Verbosity::Zero  => '0',
                    Verbosity::One   => '1',
                    Verbosity::Two   => '2',
                    Verbosity::Three => '3',
                }
            });
            options.push(verbose);
        }

        if let Some(j) = self.jobs {
            let mut verbose = String::from("--jobs=");
            verbose.push_str(&j.to_string());
            options.push(verbose);

        }

        // if let Some(map) = self.w_prog {
        //     for (prog, opts) in map {
        //     }
        // }

        let previous_dir = env::current_dir()?;
        env::set_current_dir(self.path.as_ref())?;
        let output =
            Command::new("cabal")
                .arg("build")
                .args(&options)
                .output()
                .expect("cabal-rs: Failed to execute cabal build");
        if output.status.success() {
            let mut output_dir = env::current_dir()?;
            match build_dir {
                Some(dir) => {
                    output_dir.push(dir);
                    output_dir.push("build");
                },
                None      => {
                    output_dir.push("dist");
                    output_dir.push("build");
                },
            }

            for entry in WalkDir::new(output_dir) {
                let entry = entry?;
                let mut path = entry.path().to_path_buf();
                let file_name = entry.file_name().to_str().unwrap();
                path.pop();
                if file_name.contains(STATIC_EXTENSION) {
                    println!("cargo:rustc-link-search=native={}", path.display());
                    // Get rid of lib from the file name
                    let temp = file_name.split_at(3).1;
                    // Get rid of the .a from the file name
                    let trimmed = temp.split_at(temp.len() - STATIC_EXTENSION.len()).0;
                    println!("cargo:rustc-link-lib=static={}", trimmed);
                }
            }
        } else {
            panic!("Failed to compile program with cabal. stderr output: {}",
                    String::from_utf8_lossy(&output.stderr));
        }
            env::set_current_dir(previous_dir)?;

        Ok(())
    }
}
