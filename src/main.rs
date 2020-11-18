use std::env;

mod nowmake {
    use std::{io, fs, time, process};
    use regex::Regex;
    
    pub const FILE_NAME: &str = "build.nowmake";
    pub const DEFAULT_TARGET_NAME: &str = "default";
    const TARGET_SYNTAX: &str = r"(.+):(.+)*\n\s*(.+)";

    struct Prerequisite {
        filename: String,
    }
        
    impl Prerequisite {
        fn read_from(string: String) -> Vec<Prerequisite> {
            string.split(" ").filter(|it| it.len() > 0).map(|it| Prerequisite { filename: it.to_string() }).collect()
        }
    
        fn was_updated(&self, after: &time::SystemTime) -> io::Result<bool> {
            Ok(fs::metadata(&self.filename)?.modified()? > *after)
        }
    }

    pub struct Target {
        pub result: String,
        command: String,
        prerequisites: Vec<Prerequisite>,
    }
    
    impl Target { 
        pub fn read_from(data: &str) -> Vec<Target> {
            let regex = Regex::new(TARGET_SYNTAX).unwrap();
            regex.captures_iter(&data)
                .map(|it| 
                    Target { 
                        result: it[1].to_string(), 
                        command: it[3].to_string(), 
                        prerequisites: Prerequisite::read_from(it[2].to_string()) 
                    }
                )
                .collect()
        }
        
        pub fn now_make(&self) -> io::Result<process::ExitStatus> {
            let result_metadata = fs::metadata(&self.result);
                   
            if (&self.prerequisites)
                .into_iter()
                .any(|it| {
                    match &result_metadata {
                        Err(_) => true,
                        Ok(value) => it.was_updated(&value.modified().unwrap()).unwrap_or_else(|error| {
                            println!("Can't fetch last modified date of {}. {:?}", it.filename, error);
                            false
                        })
                    }
                })
            {
                process::Command::new("sh").arg("-c").arg(&self.command).status()
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Nothing changed"))
            }
        }
    }
}

use std::{fs, process};

fn main() {
    let mut requested_targets: Vec<String> = env::args().skip(1).collect();
    if requested_targets.len() == 0 {
        requested_targets.push(String::from(nowmake::DEFAULT_TARGET_NAME));
    }
    
    let targets_text = match fs::read_to_string(nowmake::FILE_NAME) {
        Ok(content) => content,
        Err(error) => {
            println!("Can't open the targets file ({}): {:?}", nowmake::FILE_NAME, error);
            process::exit(1);
        }
    };
    
    let targets = nowmake::Target::read_from(&targets_text);
    
    for target in targets {
        if requested_targets.contains(&target.result) {
            match target.now_make() {
                Err(error) => println!("Failed: {}", error),
                Ok(_) => println!("Made {} now.", target.result)
            }
        }
    }
}
