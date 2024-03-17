use std::collections::HashSet;
use std::fs;
use std::collections::HashMap;
use clap::{App, Arg};
use futures::future::join_all;
use reqwest;
use warp::{http::Response, Filter, path::Tail};

#[tokio::main]
async fn main() {
    let matches = App::new("DNS Program")
        .version("1.0")
        .author("Author Name")
        .about("Does awesome things")
        .arg(
            Arg::with_name("local_dns")
                .short('l')
                .long("local")
                .takes_value(true)
                .help("Sets a custom local DNS"),
        )
        .arg(
            Arg::with_name("remote_dns")
                .short('r')
                .long("remote")
                .takes_value(true)
                .help("Sets a custom remote DNS"),
        )
        .arg(
            Arg::with_name("server")
                .short('s')
                .long("server")
                .help("listen on the server"),
        )
        .get_matches();

    let local_dns = matches.value_of("local_dns").unwrap_or("127.0.0.1").to_string();
    let remote_dns = matches.value_of("remote_dns").unwrap_or("127.0.0.1:1053").to_string();

    let urls = vec![
        "https://raw.githubusercontent.com/blackmatrix7/ios_rule_script/master/rule/Clash/ChinaMaxNoIP/ChinaMaxNoIP_Domain.txt",
        "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/apple-cn.txt",
        "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/direct-list.txt",
        "https://raw.githubusercontent.com/Loyalsoldier/v2ray-rules-dat/release/google-cn.txt",
        "https://raw.githubusercontent.com/blackmatrix7/ios_rule_script/master/rule/Surge/China/China_Domain.list",
    ];

    if matches.is_present("server") {
        let get_route = warp::get()
            .and(warp::path::tail())
            .and(warp::query::<HashMap<String, String>>())
            .then(move |_tail: Tail, params: HashMap<String, String>| { // 注意这里添加了Tail参数
                let urls_clone = urls.clone();
                let local_dns_clone = local_dns.clone();
                let remote_dns_clone = remote_dns.clone();
                async move {
                    let domains = get_domains(urls_clone).await;
                    let content = if params.get("min").is_some() {
                        generate_content(&domains, &local_dns_clone, &remote_dns_clone, true)
                    } else {
                        generate_content(&domains, &local_dns_clone, &remote_dns_clone, false)
                    };
                    Response::builder().body(content).unwrap()
                }
            });
        warp::serve(get_route).run(([127, 0, 0, 1], 3030)).await;
    } else {
        let domains = get_domains(urls).await;
        write_files(&domains, &local_dns, &remote_dns).await;
    }
}

async fn get_domains(urls: Vec<&str>) -> HashSet<String> {
    let clients = urls.into_iter().map(|url| reqwest::get(url));
    let responses = join_all(clients).await;
    let mut domains = HashSet::new();

    for response in responses {
        if let Ok(resp) = response {
            if let Ok(text) = resp.text().await {
                text.lines().for_each(|line| {
                    if !line.starts_with('#') {
                        if let Some(domain) = line.split_whitespace().next() {
                            domains.insert(domain.to_string());
                        }
                    }
                });
            }
        }
    }

    domains
}

fn generate_content(domains: &HashSet<String>, local_dns: &str, remote_dns: &str, min: bool) -> String {
    let mut domain_list = domains.iter().collect::<Vec<&String>>();
    domain_list.sort();

    if min {
        format!(
            "{}\n[/{}/]{}",
            remote_dns,
            domain_list.iter().map(|s| s.as_str()).collect::<Vec<&str>>().join("/"),
            local_dns
        )
    } else {
        format!(
            "{}\n{}",
            remote_dns,
            domain_list
                .iter()
                .map(|domain| format!("[/{}/]{}", domain, local_dns))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

async fn write_files(domains: &HashSet<String>, local_dns: &str, remote_dns: &str) {
    let path = "./AdGuardHome";
    let _ = fs::remove_dir_all(path);
    fs::create_dir_all(path).unwrap();
    let mut domain_list = domains.iter().collect::<Vec<&String>>();
    domain_list.sort();
    let content = generate_content(domains, local_dns, remote_dns, false);
    fs::write(format!("{}/domains.txt", path), content).unwrap();
    let content_min = generate_content(domains, local_dns, remote_dns, true);
    fs::write(format!("{}/domains-min.txt", path), content_min).unwrap();
}