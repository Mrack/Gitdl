use std::{env, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::string::String;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use threadpool::ThreadPool;

struct GitApi {
    url: String,
    proj: String,
    ref_: String,

    path: Vec<String>,
    client: reqwest::blocking::Client,
}

impl GitApi {
    fn new(url: String, proxy: String, ref_: String) -> anyhow::Result<GitApi> {
        let url_git = url::Url::parse(&url)?;
        let host = url_git.host_str().ok_or(anyhow::anyhow!("url error"))?;
        if !host.eq("github.com") {
            return Err(anyhow::anyhow!("not github url"));
        }
        let mut paths = url_git.path_segments().ok_or(anyhow::anyhow!("url error"))?;
        let repo = format!("{}/{}",
                           paths.nth(0).unwrap(),
                           paths.nth(0).unwrap()).replace(".git", "");
        println!("repo: {} ref:{}", repo, ref_);
        let client = if proxy.len() == 0 {
            reqwest::blocking::Client::builder().gzip(true)
        } else {
            reqwest::blocking::Client::builder().gzip(true).proxy(reqwest::Proxy::all(proxy).unwrap())
        };

        Ok(GitApi {
            proj: repo.to_string(),
            url: format!("https://api.github.com/repos/{}", repo),
            path: Vec::new(),
            client: client.build()?,
            ref_,
        })
    }

    fn get<T: DeserializeOwned>(&self, url: String) -> anyhow::Result<T> {
        let resp = self.client.get(url)
            .header("User-Agent", "Mozilla/5.0 (Linux; Android 13; SM-G981B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Mobile Safari/537.36")
            .header("Accept", "application/vnd.github.v3+json")
            .header("Accept-Encoding", "gzip")
            .send()?;
        Ok(serde_json::from_str(resp.text()?.as_str())?)
    }

    fn get_files(&self) -> anyhow::Result<Vec<String>> {
        let temp_file = env::temp_dir().join(format!("{}_{}.json", self.proj.replace("/", "_"), self.ref_));
        Ok(if !temp_file.exists() {
            let url = format!("{}/git/tree-file-list/{}", self.url, self.ref_);
            let res: Vec<String> = self.get::<serde_json::Value>(url)?.as_array().ok_or(anyhow::anyhow!("get files failed"))?.iter().map(|v| {
                v.as_str().unwrap().to_string()
            }).collect();
            let mut file = File::create(&temp_file)?;
            let json_data = serde_json::to_string(&res)?;
            file.write_all(json_data.as_bytes())?;
            res
        } else {
            let mut file = File::open(&temp_file)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            serde_json::from_str(&contents)?
        })
    }


    fn download(&self, path: &String) -> anyhow::Result<String> {
        let url = format!("https://raw.githubusercontent.com/{}/{}/{}", self.proj, self.ref_, path);
        let resp = self.client.get(url)
            .header("User-Agent", "Mozilla/5.0 (Linux; Android 13; SM-G981B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome")
            .header("Accept-Encoding", "gzip")
            .send()?;
        let save_name = format!("{}/{}", &self.proj, path);
        let save_path = std::path::Path::new(&save_name);
        if let Some(dir_path) = save_path.parent() {
            fs::create_dir_all(dir_path)?;
        }
        if !save_path.exists() {
            let mut file = File::create(save_path)?;
            let bytes = resp.bytes()?;
            file.write_all(&bytes)?;
        }
        Ok(save_name)
    }
}


fn main() -> anyhow::Result<()> {
    let matches = clap::Command::new("gitdl")
        .author("mrack")
        .version("0.2")
        .about("download github repo")
        .arg(clap::Arg::new("url")
            .required(true)
            .help("github url")
            .id("url"))
        .arg(clap::Arg::new("pattern")
            .help("file pattern")
            .default_value("*")
            .id("pattern"))
        .arg(clap::Arg::new("proxy")
            .short('p')
            .help("proxy")
            .default_value("")
            .id("proxy"))
        .arg(clap::Arg::new("ref")
            .short('r')
            .help("ref")
            .default_value("HEAD")
            .id("ref"))
        .get_matches();

    let url = matches.get_one::<String>("url").unwrap();
    let pattern = matches.get_one::<String>("pattern").unwrap();
    let proxy = matches.get_one::<String>("proxy").unwrap();
    let _ref = matches.get_one::<String>("ref").unwrap();
    let api = Arc::new(GitApi::new(url.to_string(), proxy.to_string(), _ref.to_string())?);
    let files = api.get_files()?;
    let pool = ThreadPool::new(num_cpus::get() * 2);
    for file in files {
        if glob::Pattern::new(pattern)?.matches(&file) {
            let api = api.clone();
            pool.execute(move || {
                if api.download(&file).is_ok() {
                    println!("downloaded {}", file);
                } else {
                    eprintln!("download failed {}", file);
                }
            });
        }
    }
    pool.join();
    println!("done!");
    Ok(())
}
