#[macro_use]
extern crate lazy_static;

mod nowmake;
use std::{env, fs, process};

fn main() {
    let mut requested_targets: Vec<String> = env::args().skip(1).collect();
    if requested_targets.len() == 0 {
        requested_targets.push(String::from(nowmake::DEFAULT_TARGET_NAME));
    }
    
    let targets_text = match fs::read_to_string(nowmake::FILE_NAME) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("Can't open the targets file ({}): {:?}", nowmake::FILE_NAME, error);
            process::exit(1);
        }
    };
    
    let targets = nowmake::Target::read_from(&targets_text);
    
    for target in targets {
        if requested_targets.contains(&target.result) {
            match target.now_make() {
                Ok(_) => println!("Made {} now.", target.result)
                Err(error) => eprintln!("Target {} failed: {}", target.result, error),
            }
        }
    }
}
