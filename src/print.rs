use clap::ArgMatches;
use rayon::prelude::*;
use spongy::{parse_with, Item, Wrapper};
use std::io::{self, Write};

use crate::context::Context;
use crate::module::ALL_MODULES;
use crate::modules::{self, utils::query_parser::parse_query};

pub fn prompt(args: ArgMatches) {
    let context = Context::new(args);
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    write!(handle, "{}", get_prompt(context)).unwrap();
}

pub fn get_prompt(context: Context) -> String {
    let config = context.config.get_root_config();

    let prompt = parse_with(config.prompt_order, |item: &Item| -> Option<String> {
        match item.wrapper {
            Wrapper::DollarCurly => {
                // Parse query string from the item
                let (module_name, _query) = parse_query(&item.text);

                if ALL_MODULES.contains(&module_name) {
                    if !context.is_module_disabled_in_config(&module_name) {
                        if let Some(module) = modules::handle(&module_name, &context) {
                            return Some(format!("{}", module));
                        }
                    }
                    Some(String::new())
                } else {
                    log::debug!(
                        "Expected prompt_order to contain value from {:?}. Instead received {}",
                        ALL_MODULES,
                        module_name,
                    );
                    None
                }
            }
            _ => None,
        }
    })
    .unwrap();

    // Write a new line before the prompt
    if config.add_newline {
        format!("\n{}", prompt)
    } else {
        prompt
    }
}

pub fn module(module_name: &str, args: ArgMatches) {
    let context = Context::new(args);

    // If the module returns `None`, print an empty string
    let module = modules::handle(module_name, &context)
        .map(|m| m.to_string())
        .unwrap_or_default();

    print!("{}", module);
}
