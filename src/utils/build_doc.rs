//! Crate documentation builder
//!
//! This module is extremely similar to cargo install operation, except it's building
//! documentation of a crate and not installing anything.

use std::path::{Path, PathBuf};
use std::env;

use cargo::core::{SourceId, Dependency, Registry, Source, Package, Workspace};
use cargo::util::{CargoResult, Config, human, Filesystem};
use cargo::sources::SourceConfigMap;
use cargo::ops;


/// Builds documentation of a crate and version.
///
/// Crate will be built into current working directory/crate-version.
///
/// It will build latest version, if no version is given.
// idea is to make cargo to download
// and build a crate and its documentation
// instead of doing it manually like in the previous version of cratesfyi
pub fn build_doc(name: &str, vers: Option<&str>, target: Option<&str>) -> CargoResult<Package> {
    let config = try!(Config::default());
    let source_id = try!(SourceId::crates_io(&config));

    let source_map = try!(SourceConfigMap::new(&config));
    let mut source = try!(source_map.load(&source_id));

    // update crates.io-index registry
    try!(source.update());

    let dep = try!(Dependency::parse(name, vers, &source_id));
    let deps = try!(source.query(&dep));
    let pkg = try!(deps.iter().map(|p| p.package_id()).max()
                   // FIXME: This is probably not a rusty way to handle options and results
                   //        or maybe it is who knows...
                   .map(|pkgid| source.download(pkgid))
                   .unwrap_or(Err(human("PKG download error"))));

    let current_dir = try!(env::current_dir());
    let target_dir = PathBuf::from(current_dir)
        .join(format!("{}-{}", pkg.manifest().name(), pkg.manifest().version()));

    let opts = ops::CompileOptions {
        config: &config,
        jobs: None,
        target: target,
        features: &[],
        no_default_features: false,
        spec: &[],
        exec_engine: None,
        mode: ops::CompileMode::Doc { deps: false },
        release: false,
        filter: ops::CompileFilter::new(true, &[], &[], &[], &[]),
        target_rustc_args: None,
        target_rustdoc_args: None,
    };

    let ws = try!(Workspace::one(pkg, &config, Some(Filesystem::new(target_dir))));

    try!(ops::compile_ws(&ws, Some(source), &opts));

    Ok(try!(ws.current()).clone())
}



/// Downloads a crate and returns Cargo Package.
pub fn get_package(name: &str, vers: Option<&str>) -> CargoResult<Package> {
    debug!("Getting package with cargo");
    let config = try!(Config::default());
    let source_id = try!(SourceId::crates_io(&config));

    let source_map = try!(SourceConfigMap::new(&config));
    let mut source = try!(source_map.load(&source_id));

    let dep = try!(Dependency::parse(name, vers, &source_id));
    let deps = try!(source.query(&dep));
    let pkg = try!(deps.iter().map(|p| p.package_id()).max()
                   // FIXME: This is probably not a rusty way to handle options and results
                   //        or maybe it is who knows...
                   .map(|pkgid| source.download(pkgid))
                   .unwrap_or(Err(human("PKG download error"))));

    Ok(pkg)
}


/// Updates central crates-io.index repository
pub fn update_sources() -> CargoResult<()> {
    let config = try!(Config::default());
    let source_id = try!(SourceId::crates_io(&config));

    let source_map = try!(SourceConfigMap::new(&config));
    let mut source = try!(source_map.load(&source_id));

    source.update()
}


/// Gets source path of a downloaded package.
pub fn source_path(pkg: &Package) -> Option<&Path> {
    // parent of the manifest file is where source codes are stored
    pkg.manifest_path().parent()
}




#[cfg(test)]
mod test {
    use std::path::Path;
    use std::fs::remove_dir_all;
    use super::*;

    #[test]
    fn test_build_doc() {
        let doc = build_doc("rand", None, None);
        assert!(doc.is_ok());

        let doc = doc.unwrap();
        remove_dir_all(format!("{}-{}", doc.manifest().name(), doc.manifest().version())).unwrap();

        let doc = build_doc("SOMECRATEWICHWILLBENVEREXISTS", None, None);
        assert!(doc.is_err());
    }

    #[test]
    fn test_get_package() {
        let pkg = get_package("rand", None);
        assert!(pkg.is_ok());

        let pkg = pkg.unwrap();

        let manifest = pkg.manifest();
        assert_eq!(manifest.name(), "rand");
    }


    #[test]
    fn test_source_path() {
        let pkg = get_package("rand", None).unwrap();
        let source_path = source_path(&pkg).unwrap();
        assert!(source_path.is_dir());

        let cargo_toml_path = Path::new(source_path).join("Cargo.toml");
        assert!(cargo_toml_path.exists());
        assert!(cargo_toml_path.is_file());

        let src_path = Path::new(source_path).join("src");
        assert!(src_path.exists());
        assert!(src_path.is_dir());
    }
}
