extern crate clap;
extern crate toml;

use std::str::FromStr;
use std::fmt::Result as FmtResult;
use std::fmt::{Formatter,Display};
use std::error::Error;
use std::any::{Any,TypeId};

use clap::{App,Arg};
use toml::Value;

#[derive(Debug)]
pub struct ConfigError {
    /// Description
    description: String,
    /// what option caused this
    option: String,
    /// Which file it came from (TOML or CLI)
    file: String,
    linecol: Option<(u64, u64)>
}

impl Display for ConfigError {
    fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
        match self {
            &ConfigError{description: ref d, option: ref opt, linecol: Some((l,c)), file: ref f, ..} =>
                write!(fmtr, "Error in {}:{}:{} with option {}: {}", f, l, c, opt, d),
            &ConfigError{description: ref d, option: ref opt, linecol: None, file: ref f, ..} =>
                write!(fmtr, "Error in {} with option {}: {}", f, opt, d)
        }
    }
}


impl Error for ConfigError {
    fn description(&self) -> &str {
        return &*self.description
    }
}

pub struct ConfigOption<'c> {
    /// TOML option name. If `None`, not allowed in TOML
    toml_name : Option<&'c str>,
    /// The clap::Arg instance, for use with CLI.
    /// If `None`, not allowed in CLI options
    arg : Option<Arg<'c, 'c, 'c, 'c, 'c, 'c>>,
    /// The type we expect to extract
    typ : TypeId
}

impl<'c> ConfigOption<'c> {
    pub fn new<T : Any + FromStr>(arg : Arg<'c, 'c, 'c, 'c, 'c, 'c>) -> ConfigOption<'c> {
        ConfigOption {
            toml_name : Some(arg.name),
            arg : Some(arg),
            typ : TypeId::of::<T>()
        }
    }
    
    pub fn new_cli<T : Any + FromStr>(arg : Arg<'c, 'c, 'c, 'c, 'c, 'c>) -> ConfigOption<'c> {
        ConfigOption {
            toml_name : None,
            arg : Some(arg),
            typ : TypeId::of::<T>()
        }
    }
    
    pub fn new_toml<T : Any + FromStr>(name : &'c str) -> ConfigOption<'c> {
        ConfigOption {
            toml_name : Some(name),
            arg : None,
            typ : TypeId::of::<T>()
        }
    }
    
    pub fn extract_from_string<T : Any + FromStr>(&self, from : &str) -> Option<T> {
        let out_type = TypeId::of::<T>();
        if out_type != self.typ {return None;}
        return T::from_str(from).ok();
    }
}

pub struct ConfiguratorGroup<'c> {
    /// The arguments allowed
    pub args : Vec<ConfigOption<'c>>,
    pub in_toml : bool,
    pub in_cli : bool
}

pub struct Configurator<'c> {
    main_group : ConfiguratorGroup<'c>,
    groups : Vec<ConfiguratorGroup<'c>>,
}

impl<'c> ConfiguratorGroup<'c> {
    // /// Create a group
    // pub fn new() -> ConfiguratorGroup<'c> {
    //     return ConfiguratorGroup {
    //         
    //     }
    // }
}
