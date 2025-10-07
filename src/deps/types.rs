use crate::deps::utils::search_library;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub type LibraryRef<'a> = Rc<RefCell<Library<'a>>>;

#[derive(Debug)]
pub struct DefinedDependency<'a> {
    pub name: &'a str,
    pub path: &'a str,
}

impl<'a> DefinedDependency<'a> {
    pub fn new(name: &'a str, path: &'a str) -> Self {
        Self { name, path }
    }
}

pub struct Library<'a> {
    pub name: &'a str,
    pub path: &'a str,

    pub is_32_bit: bool,

    pub defined_deps: Box<[DefinedDependency<'a>]>,
    pub found_deps: Vec<LibraryRef<'a>>,
}

impl<'a> std::fmt::Debug for Library<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let found_deps = self
            .found_deps
            .iter()
            .map(|dep| dep.borrow().name)
            .collect::<Vec<_>>();

        f.debug_struct("Library")
            .field("name", &self.name)
            .field("path", &self.path)
            .field("is_32_bit", &self.is_32_bit)
            .field("defined_deps", &self.defined_deps)
            .field("found_deps", &found_deps)
            .finish()
    }
}

impl<'a> PartialEq<Self> for Library<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl<'a> Eq for Library<'a> {}

impl<'a> PartialOrd<Self> for Library<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Library<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.path == other.path {
            Ordering::Equal
        } else if self.path > other.path {
            Ordering::Greater
        } else {
            Ordering::Less
        }
    }
}

impl<'a> Library<'a> {
    pub fn new(path: &'a str, deps: Box<[DefinedDependency<'a>]>) -> Self {
        let name = path.rsplit_once('/').unwrap().1;
        let is_32_bit = !path.rsplit_once('/').unwrap().0.contains("lib64");

        Self {
            name,
            path,
            is_32_bit,
            defined_deps: deps,
            found_deps: Vec::new(),
        }
    }
}

pub struct LibraryHolder<'a> {
    pub libs: Box<[LibraryRef<'a>]>,
}

impl<'a> LibraryHolder<'a> {
    pub fn new(data: &'a str) -> Self {
        let libs: Box<[LibraryRef<'a>]> = {
            let prepared_list: Box<[_]> = data
                .split('\n')
                .filter(|x| !x.is_empty())
                .map(|x| {
                    let (path, deps_str) = x.split_once(";").unwrap();
                    let deps: Box<[_]> = if !deps_str.is_empty() {
                        deps_str
                            .split(',')
                            .map(|x| {
                                let (name, path) = x.split_once('>').unwrap_or((x, x));
                                DefinedDependency::new(name, path)
                            })
                            .filter(|dep| {
                                ![
                                    "libhidlbase.so",
                                    "liblog.so",
                                    "libutils.so",
                                    "libcutils.so",
                                    "libc++.so",
                                    "libc.so",
                                    "libm.so",
                                    "libdl.so",
                                ]
                                .contains(&dep.name)
                            })
                            .collect()
                    } else {
                        Box::new([])
                    };

                    (path, deps)
                })
                .collect();

            prepared_list
                .into_iter()
                .map(|(path, deps)| Rc::new(RefCell::new(Library::new(path, deps))))
                .collect()
        };

        for lib_ref in &libs {
            let mut lib = lib_ref.borrow_mut();

            let mut found_deps = Vec::new();

            for dep in &lib.defined_deps {
                let Some(dep_lib) = search_library(&libs, dep.path, Some(lib.is_32_bit)) else {
                    log::warn!("Failed to get {} dependency of {}!", dep.name, lib.name);
                    continue;
                };

                found_deps.push(dep_lib.clone());
            }

            lib.found_deps.append(&mut found_deps);
        }

        Self { libs }
    }

    pub fn search(&self, search: &[impl AsRef<str>]) -> Box<[LibraryRef<'a>]> {
        fn recurse_deps<'a>(map: &mut HashMap<&'a str, LibraryRef<'a>>, dep: LibraryRef<'a>) {
            let path = dep.borrow().path;

            for dep in &dep.borrow().found_deps {
                recurse_deps(map, dep.clone());
            }

            map.insert(path, dep);
        }

        let mut required_list: HashMap<&'a str, LibraryRef<'a>> = HashMap::new();

        for lib_name in search {
            let Some(lib) = search_library(&self.libs, lib_name.as_ref(), None) else {
                continue;
            };

            recurse_deps(&mut required_list, lib.clone());
        }

        required_list.values().map(Rc::clone).collect()
    }
}

impl<'a> Deref for LibraryHolder<'a> {
    type Target = Box<[LibraryRef<'a>]>;

    fn deref(&self) -> &Self::Target {
        &self.libs
    }
}

impl DerefMut for LibraryHolder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.libs
    }
}
