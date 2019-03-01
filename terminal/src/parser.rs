use std::collections::HashMap;

pub enum ParseError {
    WrongCmdName,
    UnexpectedParams,
}

struct OptionRule<'a> {
    name: &'a str,
    short: &'a str,
    long: &'a str,
    has_value: bool,
    help: &'a str
}

impl<'a> OptionRule<'a> {
    fn with_name(name: &'a str) -> Self {
        OptionRule {
            name: name,
            short: "",
            long: "",
            has_value: false,
            help: ""
        }
    }

    fn short(&self, short_name: &'a str) -> Self {
        OptionRule {
            name: self.name,
            short: short_name,
            long: self.long,
            has_value: self.has_value,
            help: self.help
        }
    }

    fn long(&self, long_name: &'a str) -> Self {
        OptionRule {
            name: self.name,
            short: self.short,
            long: long_name,
            has_value: self.has_value,
            help: self.help
        }
    }

    fn takes_value(&self, has_value: bool) -> Self {
        OptionRule {
            name: self.name,
            short: self.short,
            long: self.long,
            has_value: has_value,
            help: self.help
        }
    }

    fn help(&self, help_info: &'a str) -> Self {
        OptionRule {
            name: self.name,
            short: self.short,
            long: self.long,
            has_value: self.has_value,
            help: help_info
        }
    }
}

pub struct AppOptions<'a> {
    args: HashMap<&'a str, Vec<OptionRule<'a>>>
}

impl<'a> AppOptions<'a> {
    pub fn new(app_name: &'a str) -> Self {
        let mut map = HashMap::new();
        map.insert(app_name, Vec::new());

        AppOptions {
            args: map
        }
    }

    fn arg(&mut self, rule: OptionRule) -> Self {
        
    }

    fn get_app(&self, name: &'a str) -> Option<&Vec<OptionRule>> {
        return self.args.get(name);
    }
}

/// 
pub fn parse(input: &str, opts: &AppOptions) -> Result<(), ParseError> {
    let cmds: Vec<&str> = input.split(' ').collect();

    println!("cmds = {:?}", cmds);

    match opts.get_app(cmds[0]) {
        Some(v) => {
            Ok(())
        },
        None => Err(ParseError::WrongCmdName),
    }
}