extern crate getopts;
extern crate toml;

use std::ffi::OsStr;
use std::str::FromStr;
use std::fmt::Result as FmtResult;
use std::fmt::{Formatter,Display};
use std::error::Error;
use std::any::{Any,TypeId};
use std::result;

use getopts::{HasArg,Occur,ParsingStyle};
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

#[derive(Clone, PartialEq)]
pub struct ConfigOption {
    /// TOML option name. If `None`, not allowed in TOML
    toml_name : String,
    /// The clap::Arg instance, for use with CLI.
    /// If `None`, not allowed in CLI options
    
    // Fields copied fro getopts::OptGroup
    /// Short name of the option, e.g. `h` for a `-h` option
    short_name: String,
    /// Long name of the option, e.g. `help` for a `--help` option
    long_name: String,
    /// Hint for argument, e.g. `FILE` for a `-o FILE` option
    hint: String,
    /// Description for usage help text
    desc: String,
    /// Whether option has an argument
    hasarg: HasArg,
    /// How often it can occur
    occur: Occur,
    
    /// The type we expect to extract
    typ : TypeId,
    /// Default value, if present
    default : Option<Value>
}

pub struct Match {
    /// Name of the option. Will be long_name or toml_name from ConfigOption, defaulting to
    /// long_name if both are present
    name : String,    
    /// Value found so far
    value : Option<Value>,
    /// Current value's "precedence". 0 for compiled-in-default.
    precedence : i32
}

pub struct Matches {
    main : Vec<Match>,
    groups : std::collections::HashMap<String, Vec<Match>>
}

pub struct ConfiguratorGroup {
    pub name : String,
    /// The arguments allowed
    pub args : Vec<ConfigOption>,
    pub in_toml : bool,
    pub in_cli : bool
}

pub struct Configurator {
    main_group : ConfiguratorGroup,
    groups : Vec<ConfiguratorGroup>,
}

impl ConfiguratorGroup {
    // /// Create a group
    // pub fn new() -> ConfiguratorGroup<'c> {
    //     return ConfiguratorGroup {
    //         
    //     }
    // }
}

impl Configurator {
    fn getopts(&self) -> getopts::Options{
        unimplemented!();
    }
    
    pub fn parse_cli<C: IntoIterator>(&self, args: C, p : ParsingStyle) -> Result<Matches, ConfigError>
        where C::Item: AsRef<OsStr> {
        let mut getopt_struct = self.getopts();
        getopt_struct.parsing_style(p);
        let parsed = getopt_struct.parse(args);
        
        unimplemented!();
    }
    
    pub fn parse_toml(&self, ) -> Result<Matches, ConfigError> {
        unimplemented!();
    }
}
