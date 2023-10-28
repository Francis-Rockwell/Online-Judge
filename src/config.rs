use clap::{App, Arg, ArgMatches};
use serde_derive::{Deserialize, Serialize};
use std::io::Read;
#[derive(Debug, Serialize, Deserialize)]
pub struct MyServer {
    pub bind_address: String,
    pub bind_port: i32,
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemType {
    Standard,
    Strict,
    Spj,
    DynamicRanking,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemCase {
    pub score: f64,
    pub input_file: String,
    pub answer_file: String,
    pub time_limit: u128,
    pub memory_limit: i32,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Misc {
    pub packing: Option<Vec<Vec<usize>>>,
    pub special_judge: Option<Vec<String>>,
    pub dynamic_ranking_ratio: Option<f64>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Problem {
    pub id: usize,
    pub name: String,
    #[serde(rename = "type")]
    pub ty: ProblemType,
    pub misc: Option<Misc>,
    pub cases: Vec<ProblemCase>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Language {
    pub name: String,
    pub file_name: String,
    pub command: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: MyServer,
    pub problems: Vec<Problem>,
    pub languages: Vec<Language>,
    pub flush: Option<bool>,
}
/*
function: to read all content in the file
input: path: a &str of the file's location
       name: a &str of the file's name
output: a Ok(string) with all content in the file, or an err with the file's name
*/
pub(crate) fn fread(path: &str, name: &str) -> Result<String, String> {
    let mut text: String = String::new();
    let mut file = std::fs::File::open(path).unwrap();
    if let Err(_) = file.read_to_string(&mut text) {
        let err = String::from(name);
        return Err(err + " open error!");
    }
    Ok(text)
}
/*
function: to collect env args from the command line
input: None
output: env args
*/
pub fn args() -> ArgMatches<'static> {
    let args = App::new("oj")
        .arg(
            Arg::with_name("config")
                .long("config")
                .short("c")
                .takes_value(true),
        )
        .arg(Arg::with_name("flush-data").long("flush-data").short("f"))
        .get_matches();
    args
}
/*
function: to transform args into a Config struct
input: args: env args
output: a Ok(Config) , or an err with related reason
*/
pub fn config(args: &ArgMatches) -> Result<Config, String> {
    match args.value_of("config") {
        Some(path) => {
            let text = fread(path, "config")?;
            let config: Result<Config, serde_json::Error> = serde_json::from_str(&text);
            match config {
                Ok(_) => {
                    let mut config = config.unwrap();
                    if args.is_present("flush-data") {
                        config.flush = Some(true);
                    } else {
                        config.flush = Some(false);
                    }
                    for problem in &config.problems {
                        for case in &problem.cases {
                            fread(&case.input_file, "inputfile?")?;
                            fread(&case.answer_file, "ansfile?")?;
                        }
                    } // check if every input and answer file valid
                    Ok(config)
                }
                Err(_) => Err(String::from("Config Error")),
            }
        }
        None => Err(String::from("No Config")),
    }
}
#[derive(Serialize, Deserialize)]
pub struct Error {
    pub code: i32,
    pub reason: String,
    pub message: String,
}
