// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Creation and destruction of sandboxes.

use profile::Profile;

use std::collections::HashMap;
use std::env;
use std::ffi::CString;
use std::old_io::IoResult;
use std::old_io::process::{self, Process};
use std::old_path::BytesContainer;

pub use platform::{ChildSandbox, Sandbox};

/// All platform-specific sandboxes implement this trait.
///
/// A new sandbox can be created with `Sandbox::new()`, which all platform-specific sandboxes
/// implement.
pub trait SandboxMethods {
    /// Returns this sandbox profile.
    fn profile(&self) -> &Profile;

    /// Spawns a child process eligible for sandboxing.
    fn start(&self, command: &mut Command) -> IoResult<Process>;
}

/// All platform-specific sandboxes in the child process implement this trait.
pub trait ChildSandboxMethods {
    /// Activates the restrictions in this child process from here on out. Be sure to check the
    /// return value!
    fn activate(&self) -> Result<(),()>;
}

pub struct Command {
    module_path: CString,
    args: Vec<CString>,
    env: HashMap<CString,CString>,
}

impl Command {
    /// Constructs a new `Command` for launching the executable at path `module_path` with no
    /// arguments and no environment by default. Builder methods are provided to change these
    /// defaults and otherwise configure the process.
    pub fn new<T>(module_path: T) -> Command where T: BytesContainer {
        Command {
            module_path: CString::from_slice(module_path.container_as_bytes()),
            args: Vec::new(),
            env: HashMap::new(),
        }
    }

    /// Constructs a new `Command` for launching the current executable.
    pub fn me() -> IoResult<Command> {
        Ok(Command::new(try!(env::current_exe())))
    }

    /// Adds an argument to pass to the program.
    pub fn arg<'a,T>(&'a mut self, arg: T) -> &'a mut Command where T: BytesContainer {
        self.args.push(CString::from_slice(arg.container_as_bytes()));
        self
    }

    /// Adds multiple arguments to pass to the program.
    pub fn args<'a,T>(&'a mut self, args: &[T]) -> &'a mut Command where T: BytesContainer {
        self.args.extend(args.iter().map(|arg| CString::from_slice(arg.container_as_bytes())));
        self
    }

    /// Inserts or updates an environment variable mapping.
    pub fn env<'a,T,U>(&'a mut self, key: T, val: U) -> &'a mut Command
                       where T: BytesContainer, U: BytesContainer {
        self.env.insert(CString::from_slice(key.container_as_bytes()),
                        CString::from_slice(val.container_as_bytes()));
        self
    }

    /// Executes the command as a child process, which is returned.
    pub fn spawn(&self) -> IoResult<Process> {
        let env: Vec<_> = self.env.iter().collect();
        process::Command::new(&self.module_path).args(self.args.as_slice())
                                                .env_set_all(env.as_slice())
                                                .spawn()
    }
}
