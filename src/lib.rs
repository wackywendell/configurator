extern crate getopts;
extern crate toml;

use std::ffi::OsStr;
// use std::str::FromStr;
use std::fmt::Result as FmtResult;
use std::fmt::{Formatter, Display};
use std::error::Error;
use std::any::TypeId; //Any
// use std::result;
use std::convert::From;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use getopts::{HasArg, Occur, ParsingStyle};
use toml::Value;
// #[derive(Debug)]
// pub struct ConfigError {
//     /// Description
//     description: String,
//     /// what option caused this
//     option: String,
//     /// Which file it came from (TOML or CLI)
//     file: String,
//     linecol: Option<(u64, u64)>
// }
//
// impl Display for ConfigError {
//     fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
//         match self {
//             &ConfigError{description: ref d, option: ref opt, linecol: Some((l,c)), file: ref f, ..} =>
//                 write!(fmtr, "Error in {}:{}:{} with option {}: {}", f, l, c, opt, d),
//             &ConfigError{description: ref d, option: ref opt, linecol: None, file: ref f, ..} =>
//                 write!(fmtr, "Error in {} with option {}: {}", f, opt, d)
//         }
//     }
// }
//
//
// impl Error for ConfigError {
//     fn description(&self) -> &str {
//         return &*self.description
//     }
// }

#[derive(Debug)]
pub struct DuplicateKeyError {
    key: String,
    msg: String,
}

impl Display for DuplicateKeyError {
    fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
        self.msg.fmt(fmtr)
    }
}

impl Error for DuplicateKeyError {
    fn description(&self) -> &str {
        &self.key
    }
}

#[derive(Debug)]
pub struct MissingKeyError {
    key: String,
    msg: String,
}

impl Display for MissingKeyError {
    fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
        self.msg.fmt(fmtr)
    }
}

impl Error for MissingKeyError {
    fn description(&self) -> &str {
        &self.key
    }
}

#[derive(Debug)]
pub enum ConfigError {
    CliError(getopts::Fail),
}

impl Display for ConfigError {
    fn fmt(&self, fmtr: &mut Formatter) -> FmtResult {
        match self {
            &ConfigError::CliError(ref e) => e.fmt(fmtr),
        }
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        match self {
            &ConfigError::CliError(ref e) => e.description(),
        }
    }
}

impl From<getopts::Fail> for ConfigError {
    fn from(e: getopts::Fail) -> Self {
        ConfigError::CliError(e)
    }
}

#[derive(Clone, PartialEq)]
pub struct ConfigOption {
    /// TOML option name. If `""`, not allowed in TOML
    toml_name: String,
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
    typ: TypeId,
    /// Default value, if present
    default: Option<Value>,
}

pub struct Match {
    /// Value found so far
    pub value: Option<Value>,
    /// Current value's "precedence". 0 for CLI, std::i32::MIN for compiled-in-default.
    pub precedence: i32,
}

/// The configuration values after parsing.
pub struct Matches {
    // Group name -> match name -> match
    /// The name of the option will be long_name or toml_name from ConfigOption, defaulting to
    /// long_name if both are present
    groups: HashMap<String, HashMap<String, Match>>,
}

impl Matches {
    pub fn from_configs<C: IntoIterator>(config: C) -> Result<Matches, DuplicateKeyError>
        where C::Item: AsRef<ConfiguratorGroup>
    {
        let mut matches = Matches { groups: HashMap::new() };
        for group_ref in config {
            let group: &ConfiguratorGroup = group_ref.as_ref();
            let mut map: HashMap<String, Match> = HashMap::new();

            for opt in &group.args {
                let name: String = match (opt.toml_name.as_ref(), opt.long_name.as_ref()) {
                    (ref tname, "") => {
                        let tname_ref: &str = tname;
                        tname_ref.to_string()
                    } // String::clone(tname),
                    (_, ref long_name) => long_name.to_string(),
                };
                let mtch = Match {
                    value: opt.default.clone(),
                    precedence: std::i32::MIN,
                };
                map.insert(name, mtch);
            }


            match matches.groups.insert(group.name.clone(), map) {
                None => {}
                Some(_) => {
                    return Err(DuplicateKeyError {
                        key: group.name.clone(),
                        msg: format!("Found two groups with the same key {}", group.name),
                    });
                }
            }
        }

        Ok(matches)
    }

    pub fn update(&mut self,
                  group: &str,
                  name: &str,
                  value: Value,
                  precedence: i32)
                  -> Result<(), MissingKeyError> {
        match self.groups
                  .entry(group.to_owned()) {
            Entry::Vacant(_) => {
                return Err(MissingKeyError {
                    key: group.to_owned(),
                    msg: format!("Group {} not found", group),
                })
            }
            Entry::Occupied(mut loc) => {
                let mut group_map: &mut HashMap<String, Match> = loc.get_mut();
                match group_map.entry(name.to_owned()) {
                    Entry::Vacant(_) => {
                        return Err(MissingKeyError {
                            key: name.to_owned(),
                            msg: format!("Key {} not found in group {}", name, group),
                        })
                    }
                    Entry::Occupied(mut loc) => {
                        let ref mut cur_match = loc.get_mut();
                        if cur_match.precedence > precedence {
                            **cur_match = Match {
                                value: Some(value),
                                precedence: precedence,
                            };
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct ConfiguratorGroup {
    pub name: String,
    /// The arguments allowed
    pub args: Vec<ConfigOption>,
    pub in_toml: bool,
    pub in_cli: bool,
}

pub struct Configurator {
    groups: Vec<ConfiguratorGroup>,
    matches: Matches,
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
    pub fn new(groups: Vec<ConfiguratorGroup>) -> Configurator {
        unimplemented!()
    }

    fn getopts(&self) -> getopts::Options {
        let mut opts = getopts::Options::new();
        for group in &self.groups {
            if !group.in_cli {
                continue;
            }
            for opt in &group.args {
                opts.opt(&opt.short_name,
                         &opt.long_name,
                         &opt.desc,
                         &opt.hint,
                         opt.hasarg,
                         opt.occur);
            }
        }

        opts
    }

    pub fn parse_cli<C: IntoIterator>(&self,
                                      args: C,
                                      p: ParsingStyle,
                                      precedence: i32)
                                      -> Result<Matches, ConfigError>
        where C::Item: AsRef<OsStr>
    {
        let mut getopt_struct = self.getopts();
        getopt_struct.parsing_style(p);
        let parsed = try!(getopt_struct.parse(args));
        for group in &self.groups {
            if !group.in_cli {
                continue;
            }
            for opt in &group.args {
                unimplemented!();
            }
        }
        unimplemented!();
    }

    pub fn parse_toml(&self) -> Result<Matches, ConfigError> {
        unimplemented!();
    }
}
