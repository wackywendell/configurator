extern crate getopts;
extern crate toml;

use std::ffi::OsStr;
use std::str::FromStr;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TomlType {
    String,
    Integer,
    Float,
}

impl TomlType {
    pub fn to_value(&self, s: &str) -> Result<Value, Vec<toml::ParserError>> {
        match *self {
            TomlType::String => Ok(Value::String(s.to_owned())),
            TomlType::Integer => {
                let v = Value::from_str(s);
                match v {
                    Ok(Value::Integer(n)) => Ok(Value::Integer(n)),
                    Ok(wrong_type) => unimplemented!(),
                    Err(e) => Err(e),
                }
            }
            TomlType::Float => {
                let v = Value::from_str(s);
                match v {
                    Ok(Value::Float(n)) => Ok(Value::Float(n)),
                    Ok(wrong_type) => unimplemented!(),
                    Err(e) => Err(e),
                }
            }
        }
    }
}

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

/// A single option, for use in either a config file or as a command-line option
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
    typ: TomlType,
    /// Default value, if present
    default: Option<Value>,
}

impl ConfigOption {
    pub fn get_name(&self) -> &str {
        match (self.toml_name.as_ref(), self.long_name.as_ref()) {
            (ref tname, "") => tname,
            (_, ref long_name) => long_name,
        }
    }
}

/// The best "match" for an option, kept as a `toml::Value`
pub struct Match {
    /// Value found so far
    pub value: Option<Value>,
    /// Current value's "precedence". 0 for CLI, std::i32::MIN for compiled-in-default.
    pub precedence: i32,
}

/// The configuration values after parsing.
/// Kept as a map of (Group name) -> (match name) -> Match
pub struct Matches {
    // Group name -> match name -> match
    /// The name of the option will be long_name or toml_name from ConfigOption, defaulting to
    /// long_name if both are present
    groups: HashMap<String, HashMap<String, Match>>,
}

impl Matches {
    /// Convert a list of ConfiguratorGroups into matches, with just the defaults so far.
    /// Later, we can call `update` to include information from other sources, e.g. a config
    /// file or from the command-line
    pub fn from_configs<C: IntoIterator>(config: C) -> Result<Matches, DuplicateKeyError>
        where C::Item: AsRef<ConfiguratorGroup>
    {
        let mut matches = Matches { groups: HashMap::new() };
        for group_ref in config {
            let group: &ConfiguratorGroup = group_ref.as_ref();
            let mut map: HashMap<String, Match> = HashMap::new();

            for opt in &group.args {
                let name: String = opt.get_name().to_owned();
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

/// A set of options that form a configuration group, as would be found in a toml file.
pub struct ConfiguratorGroup {
    pub name: String,
    /// The arguments allowed
    pub args: Vec<ConfigOption>,
    pub in_toml: bool,
    pub in_cli: bool,
}

/// A class for 1) managing what options are available and what their defaults are, and
/// 2) which options have been found so far.
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

    pub fn parse_cli<C: IntoIterator>(&mut self,
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
                let getopt_match_str = match parsed.opt_str(&*opt.short_name) {
                    None => continue,
                    Some(s) => s,
                };
                let value = try!(opt.typ.to_value(&*getopt_match_str));
                self.matches.update(&*group.name, opt.get_name(), value, precedence);
            }
        }
        Ok(self.matches)
    }

    pub fn parse_toml_partial(&self,
                              parsed_toml: toml::Value,
                              precedence: i32)
                              -> Result<Matches, ConfigError> {
        for group in &self.groups {
            if !group.in_toml {
                continue;
            }
            for opt in &group.args {
                let lookup_str = format!("{}.{}", group.name, opt.get_name());
                let value = try!(parsed_toml.lookup(&*lookup_str));
                self.matches.update(&*group.name, opt.get_name(), value, precedence);
            }
        }
        Ok(self.matches)
    }

    pub fn parse_toml(&self,
                      parsed_toml: toml::Value,
                      precedence: i32)
                      -> Result<Matches, ConfigError> {
        unimplemented!();
    }
}
