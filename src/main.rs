use std::io::{stderr, IsTerminal};

use colored::Colorize;
use crates_io_api::CratesQuery;
use kdam::{term, tqdm, BarExt};
use rand::Rng;

async fn get_crates(number: usize) -> Vec<String> {
    let client = crates_io_api::AsyncClient::new(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36", 
        std::time::Duration::from_millis(50)
    ).unwrap();

    let mut crates = Vec::new();

    for i in 0..(number / 50) {
        let summary = client
            .crates(
                CratesQuery::builder()
                    .sort(crates_io_api::Sort::Downloads)
                    .page((i + 1) as u64)
                    .page_size(50)
                    .build(),
            )
            .await
            .unwrap();

        crates.extend(
            summary
                .crates
                .into_iter()
                .map(|c| format!("{} v{}", &c.name, &c.max_version)),
        );
    }

    crates
}

fn generate_postfix(project_name: &str, crates: Vec<String>) -> String {
    let teminal_width = termsize::get().unwrap().cols;
    let avil_width = teminal_width - (teminal_width / 3);

    let mut postfix = String::new();
    postfix.push_str(format!("{}(build)", project_name).as_str());

    for cr in crates {
        let mut clone = postfix.clone();
        clone.push_str(format!(", {}", cr).as_str());

        if clone.len() > avil_width.into() {
            postfix.push_str(format!(", {}...", cr.chars().take(3).collect::<String>()).as_str());
            break;
        } else {
            postfix = clone;
        }
    }

    postfix
}

#[tokio::main]
async fn main() {
    term::init(stderr().is_terminal());
    term::hide_cursor().unwrap();

    let current_dir = std::env::current_dir().unwrap();
    // C:/projects/test => test
    let current_dir_name = current_dir.file_name().unwrap().to_str().unwrap();

    println!(
        "{} waiting for file lock on package cache",
        "Blocking".cyan()
    );

    let crates = get_crates(200).await;

    let text = format!("{}", "Compiling".green());

    let mut pb = tqdm!(
        total = crates.len() + 1,
        desc = text,
        animation = "arrow",
        position = 0,
        force_refresh = true,
        ncols = 20,
        bar_format = "{desc} |{animation}| {count}/{total}: {postfix}"
    );
    pb.postfix = generate_postfix(current_dir_name, crates.clone());

    for (done_crates, package) in (0_i32..).zip(crates.iter()) {
        let text = format!("\t{} {}", "Compiling".green(), package);
        pb.write(text).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(
            rand::thread_rng().gen_range(100..1000),
        ));

        pb.update(1).unwrap();
        pb.postfix = generate_postfix(
            current_dir_name,
            crates
                .clone()
                .into_iter()
                .skip(done_crates as usize)
                .collect(),
        );
    }

    let text = format!(
        "\t{} {} v0.1.0 ({})",
        "Compiling".green(),
        current_dir_name,
        current_dir.display()
    );
    pb.write(text).unwrap();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(10000));
    }
}
