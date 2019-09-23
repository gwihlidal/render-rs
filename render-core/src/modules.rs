use crate::backend::RenderBackendModule;
use crate::error::{Error, Result};
use failure::Fail;
use glob::glob;
use libloading::{Library, Symbol};
use std::borrow::Borrow;
use std::env;
use std::path::Path;

pub(crate) type FactoryFunc = extern "C" fn() -> Box<dyn RenderBackendModule>;

type LibResult<T> = ::std::io::Result<T>;

#[cfg(target_os = "windows")]
const LIBRARY_EXT: &'static str = "dll";

#[cfg(target_os = "macos")]
const LIBRARY_EXT: &'static str = "dylib";

#[cfg(all(unix, not(target_os = "macos")))]
const LIBRARY_EXT: &'static str = "so";

pub(crate) fn is_backend_module(module_path: &Path) -> bool {
    if let Ok(library) = Library::new(&module_path) {
        let factory_func: LibResult<Symbol<FactoryFunc>> =
            unsafe { library.get(b"render_backend_factory") };
        factory_func.is_ok()
    } else {
        false
    }
}

pub fn create_backend_module(library: &Box<Library>) -> Result<Box<dyn RenderBackendModule>> {
    let factory_func: LibResult<Symbol<FactoryFunc>> =
        unsafe { library.get(b"render_backend_factory") };
    if let Ok(factory_func) = factory_func {
        let backend_module = factory_func();
        Ok(backend_module)
    } else {
        Err(Error::backend("create backend module failed"))
    }
}

pub fn load_backend_modules(module_path: &Path) -> Result<Vec<Box<Library>>> {
    info!("Loading render backend modules from {:?}", module_path);
    let path_str = module_path
        .as_os_str()
        .to_str()
        .expect("failed to parse module path");
    //let path_glob = format!("{}/**/*.{}", path_str, LIBRARY_EXT);
    let path_glob = format!("{}/*.{}", path_str, LIBRARY_EXT);
    //info!("path glob: {}", path_glob);

    let mut modules: Vec<Box<Library>> = vec![];

    for entry in glob(&path_glob).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                let valid_module = is_backend_module(&path);
                if valid_module {
                    modules.push(Box::new(Library::new(&path).unwrap()));
                }
            }
            Err(e) => error!("{:?}", e),
        }
    }

    Ok(modules)
}
