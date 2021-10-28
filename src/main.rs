use clap::{Arg, App};
use dirs;
use std::{fs, io};
use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct Translations {
    translations: Vec<Translation>
}

#[derive(Deserialize, Debug)]
struct Translation {
    detected_source_language: String,
    text: String
}

struct Input {
    text: String,
    api_key: String,
    // TODO: Add enum for languages
    source_language: Option<String>,
    target_language: String,
    search_string: String
}

impl Input {
    fn new(text: String, api_key: String, target_language: String, source_language: Option<String>) -> Input {
        Input {
            text: text,
            api_key: api_key,
            target_language: target_language,
            source_language: source_language,
            search_string: String::new()
        }
    }

    fn build_search_string(&mut self) {
        self.search_string.push_str("auth_key=");
        self.search_string.push_str(&self.api_key);
        self.search_string.push_str("&text=");
        self.search_string.push_str(&self.text);
        self.search_string.push_str("&target_lang=");
        self.search_string.push_str(&self.target_language);
        if let Some(source_language) = &self.source_language {
            self.search_string.push_str("&source_lang=");
            self.search_string.push_str(&source_language);
        }
    }
}

fn send_request(search_string: String) -> Option<Translations> {
    let client = reqwest::blocking::Client::new();
    let resp = client.post("https://api-free.deepl.com/v2/translate")
        .header("User-Agent", "deeprs cli v0.1")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(search_string)
        .send().ok()?;

    match resp.status() {
        reqwest::StatusCode::OK => {
            let translations: Translations = resp.json::<Translations>().unwrap();
            Some(translations)
        },
        _ => {
            println!("Status Code: {}", resp.status());
            None
        }
    }
}

fn get_api_key() -> Result<String, String>{
    let home_dir = dirs::home_dir();
    match home_dir{
        Some(mut home_dir) => {
            home_dir.push(".deeprs");
            match fs::read_to_string(&home_dir) {
                Ok(api_key) => Ok(api_key),
                Err(_) => {
                    println!("No API key was found on your system.");
                    println!("If you don't have one yet, you can register here for free:");
                    println!("https://www.deepl.com/pro-api?cta=header-pro-api/");
                    println!("Please enter a valid API key: ");
                    let mut api_key = String::new();
                    io::stdin().read_line(&mut api_key).expect("Failed to read input");
                    fs::write(home_dir, &api_key).expect("Failed writing key file");
                    Ok(api_key)
                }
            }
        },
        None => Err(String::from("Faild getting home directory"))
    }
}

fn main() {
    let matches = App::new("deeprs: CLI for DeepL")
        .version("0.1.0")
        .arg(Arg::with_name("source language")
             .required(false)
             .short("s")
             .takes_value(true)
             .help(concat!("Define the source language. ",
                           "This is not needed in the most cases ",
                           "since DeepL can detect the source language by its own")))
        .arg(Arg::with_name("target language")
             .required(true)
             .short("t")
             .takes_value(true)
             .help("target language")
        ).arg(Arg::with_name("text")
              .takes_value(true)
              .required(true)
              .help("Text you want to translate")
        ).get_matches();

    let api_key = get_api_key().unwrap();

    let mut user_input = Input::new(matches.value_of_lossy("text").unwrap().to_string(),
                                    api_key,
                                    matches.value_of("target language").unwrap().to_string(),
                                    match matches.value_of("source language") {
                                        Some(sl) => Some(sl.to_string()),
                                        None => None
                                    });

    user_input.build_search_string();

    if let Some(resp) = send_request(user_input.search_string) {
        for translation in &resp.translations {
            println!("From Language: {}", translation.detected_source_language);
            println!("Text: {}", translation.text);
        }
    }
}
