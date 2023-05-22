use colored::Colorize;
use std::env::current_dir;
use std::error::Error;
use std::path::PathBuf;
use std::{env, fmt};
use std::{fs, process};
// enum ArgumentTypes {

// }
const HELP_MESSSAGE: &str = "\
USAGE:
  grust PATTERN [OPTIONS]
  -gitignore - Doesn't consider gitignore file when searching
  -limit-size [NUMBER] - Overwrite file size limit (DEFAULT = 50000)
  -path [PATH] - Overwrite path to search (dir | file) (DEFAULT = [CURRENT_DIR])
";

fn read_gitignore_file(dir: &PathBuf) -> Vec<PathBuf> {
    let mut file = dir.clone();
    file.push(".gitignore");
    if !file.exists() {
        return vec![];
    }
    let mut return_vec: Vec<PathBuf> = Vec::new();
    let data = if let Ok(result) = fs::read_to_string(file) {
        result
    } else {
        return vec![];
    };
    for line in data.lines() {
        let mut new_path = PathBuf::new();
        new_path.push(dir.as_os_str());
        new_path.push(line.replace("./", "").replace("/", ""));
        return_vec.push(new_path);
    }
    return_vec
}

fn read_dir(dir: fs::ReadDir, pattern: &String, ignored_files: &Vec<PathBuf>, limit_size: u32) {
    for d in dir {
        if let Ok(result) = d {
            if ignored_files.contains(&result.path()) || result.file_name().eq(".git") {
                continue;
            }
            if result.file_type().unwrap().is_dir() {
                if let Ok(next_path) = fs::read_dir(result.path().as_path()) {
                    read_dir(next_path, pattern, &ignored_files, limit_size)
                } else {
                    eprintln!("Could not read directory")
                };
            } else {
                search_on_file(
                    result.path().as_os_str().to_str().unwrap().clone(),
                    pattern,
                    limit_size,
                );
            }
        } else {
            eprintln!("Failed to read dir")
        }
    }
}

fn search_on_file(path: &str, pattern: &String, limit_size: u32) {
    match fs::metadata(path) {
        Ok(result) => {
            if result.len() as u32 > limit_size {
                return;
            }
        }
        Err(_) => eprintln!("Error reading metadata"),
    }
    let data = if let Ok(data_file) = fs::read_to_string(path) {
        data_file
    } else {
        String::new()
    };
    let data: Vec<&str> = data.split("\n").collect();
    for (idx_line, line) in data.iter().enumerate() {
        match line.find(pattern) {
            Some(idx) => {
                println!(
                    "{}:{}:{}",
                    path.green(),
                    (idx_line + 1).to_string().green(),
                    (idx + 1).to_string().green()
                );
                println!("{} ", line.replace("  ", ""));
                println!("");
            }
            None => (),
        };
    }
}

// struct ConfigBuildError<'a> {
//     cause: &'a dyn Error,
//     descriptiom: String,
// }

// impl Error for ConfigBuildError<'_> {
//     fn cause(&self) -> Option<&dyn Error> {
//         Some(self.cause)
//     }
//     fn description(&self) -> &str {
//         "Something went wrong with the parameters passed"
//     }
// }
// impl fmt::Display for ConfigBuildError<'_> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         Ok(())
//     }
// }

enum ConfigBuildError {
    Test,
}
struct Config {
    path: PathBuf,
    pattern: String,
    github: bool,
    limit_size: u32,
}
impl Config {
    fn build(args: Vec<String>) -> Result<Self, ConfigBuildError> {
        let mut path: PathBuf = PathBuf::new();
        let pattern: String = match args.get(1) {
            Some(value) => value.clone(),
            None => {
                eprintln!("No pattern to search for provided");
                process::exit(1);
            }
        };
        let mut github: bool = true;
        let mut limit_size = 50_000;
        for (index, arg) in args.iter().enumerate() {
            if arg == "-path" {
                if let Some(value) = args.get(index + 1) {
                    path.push(value);
                } else {
                    eprintln!("You tried to overwrite the path but didn't set a value");
                    process::exit(1);
                }
            } else if arg == "-gitignore" {
                github = false;
            } else if arg == "-limit-size" {
                if let Some(value) = args.get(index + 1) {
                    match value.parse() {
                        Ok(result) => limit_size = result,
                        Err(e) => {
                            eprintln!("The value you passed could not be parsed to a number");
                            return Err(ConfigBuildError::Test);
                        }
                    }
                } else {
                    eprintln!("You tried to overwrite the limit size but didn't set a value");
                    process::exit(1);
                }
            }
        }
        if path == PathBuf::new() {
            path = match current_dir() {
                Ok(res) => res,
                Err(_) => panic!(""),
            };
        }
        Ok(Config {
            path,
            pattern,
            github,
            limit_size,
        })
    }
}

fn run(config: Config) {
    let ignored_files = match config.github {
        true => read_gitignore_file(&config.path),
        false => vec![],
    };
    if config.path.is_dir() {
        let dir = match fs::read_dir(config.path) {
            Ok(d) => d,
            Err(err) => {
                eprintln!("Application error: {}", err);
                process::exit(1);
            }
        };
        read_dir(dir, &config.pattern, &ignored_files, config.limit_size);
    } else {
        let file = match config.path.as_os_str().to_str() {
            Some(value) => value,
            None => {
                eprintln!("Could not read file");
                process::exit(1);
            }
        };
        search_on_file(file, &config.pattern, config.limit_size);
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let config = match Config::build(args) {
        Ok(res) => res,
        Err(_) => panic!(""),
    };
    run(config);
}
