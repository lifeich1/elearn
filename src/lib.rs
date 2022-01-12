use error_chain::error_chain;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tokio::sync::{mpsc, oneshot};

mod exam;
mod www;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        InitLog(log::SetLoggerError);
        ConfigLog(log4rs::config::runtime::ConfigErrors);
        Ron(ron::error::Error);
    }
}

const VAR_PATH: &str = "/var/lifeich1/elearn";
const LOGYML_PATH: &str = "/var/lifeich1/elearn/log4rs.yml";

fn test_data_path<T: ToString, U: ToString>(typ: T, name: U) -> String {
    format!(
        "{}/data/test/{}/{}.ron",
        VAR_PATH,
        typ.to_string(),
        name.to_string()
    )
}

fn commit_test_data<T: ToString, U: ToString, V: Serialize>(
    typ: T,
    name: U,
    value: &V,
) -> Result<()> {
    let out = File::create(test_data_path(typ, name))?;
    ron::ser::to_writer(out, value)?;
    Ok(())
}

fn load_test_data<T: ToString, U: ToString, V: DeserializeOwned>(typ: T, name: U) -> Result<V> {
    let p = test_data_path(typ, name);
    log::info!("opening {}", &p);
    let input = File::open(p)?;
    Ok(ron::de::from_reader(input)?)
}

fn history_root() -> String {
    format!("{}/data/history", VAR_PATH)
}

fn history_dir_path<T: ToString, U: ToString>(typ: T, name: U) -> String {
    let s = format!(
        "{}/{}/{}",
        history_root(),
        typ.to_string(),
        name.to_string()
    );
    let p = Path::new(&s);
    if !p.exists() {
        std::fs::create_dir_all(p)
            .unwrap_or_else(|e| panic!("Create test history dir {:?} error: {}", p, e));
    }
    s
}

fn history_path<T: ToString, U: ToString>(typ: T, name: U, tag: &str) -> String {
    format!("{}/{}.html", history_dir_path(typ, name), tag)
}

fn commit_history<T: ToString, U: ToString>(typ: T, name: U, data: &str) -> Result<()> {
    let tag = chrono::Local::now().format("%F_%Hh%Mm%Ss").to_string();
    log::info!(
        "commiting new history {}/{}/{}",
        typ.to_string(),
        name.to_string(),
        &tag
    );
    let mut out = File::create(history_path(typ, name, &tag))?;
    write!(&mut out, "{}", data)?;
    Ok(())
}

fn list_history_of_kind<T: ToString, U: ToString>(typ: T, name: U) -> Vec<String> {
    let s = history_dir_path(typ, name);
    let p = Path::new(&s);
    let d = p
        .read_dir()
        .unwrap_or_else(|e| panic!("Open test history dir {:?} error: {}", p, e));
    d.filter_map(|result| match result {
        Ok(entry) => entry
            .file_name()
            .into_string()
            .map_err(|e| log::error!("history name {:?} cannot convert to utf8", e))
            .ok()
            .and_then(|s| s.strip_suffix(".html").map(String::from)),
        Err(e) => {
            log::error!("read dir {:?} error: {}", p, e);
            None
        }
    })
    .collect()
}

fn list_names_of_test_type<T: ToString>(typ: T) -> Vec<String> {
    let datapath = test_data_path(typ.to_string(), "none");
    let p = Path::new(&datapath)
        .parent()
        .unwrap_or_else(|| panic!("Cannot get dir of test type {}", typ.to_string()));
    if !p.exists() {
        std::fs::create_dir_all(p)
            .unwrap_or_else(|e| panic!("Create test type data dir {:?} error: {}", p, e));
    }
    let d = p
        .read_dir()
        .unwrap_or_else(|e| panic!("Open test type data dir {:?} error: {}", p, e));
    d.filter_map(|result| match result {
        Ok(entry) => entry
            .file_name()
            .into_string()
            .map_err(|e| {
                log::error!(
                    "test name {:?} of test type {} cannot to utf string",
                    e,
                    typ.to_string()
                )
            })
            .ok()
            .and_then(|s| s.strip_suffix(".ron").map(String::from)),
        Err(e) => {
            log::error!("read dir {:?} error: {}", p, e);
            None
        }
    })
    .collect()
}

fn str_dump2file<P: AsRef<Path>, T: ToString>(path: P, text: T, is_force: bool) -> Result<()> {
    let p = path.as_ref();
    if p.exists() && !is_force {
        return Ok(());
    }
    let d = p.parent().unwrap_or_else(|| panic!("path {:?} no parent", p));
    if !d.exists() {
        std::fs::create_dir_all(d)
            .unwrap_or_else(|e| panic!("Create data dir {:?} error: {}", d, e));
    }
    let mut out = File::create(p)?;
    write!(&mut out, "{}", text.to_string())?;
    Ok(())
}

fn prepare_log<T: ToString>(tag: T) -> Result<()> {
    str_dump2file(Path::new(LOGYML_PATH), include_str!("../assets/log4rs.yml"), false)?;

    log4rs::init_file(LOGYML_PATH, Default::default()).unwrap_or_else(|e| panic!("prepare log fatal error(s): {}", e));

    log::info!(
        "[tag {}] {} version {}; logger prepared",
        tag.to_string(),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    Ok(())
}

fn expect_log<T: ToString>(tag: T) {
    prepare_log(tag).unwrap_or_else(|e| panic!("prepare log fatal error(s): {}", e));
}

macro_rules! graceful_shutdown {
    ($done_rx:ident) => {{
        println!("\nDoing graceful shutdown ...");
        println!("Press ^C again to halt");

        tokio::select! {
            _ = $done_rx.recv() => {},
            _ = tokio::signal::ctrl_c() => {
                println!("\nforced halt");
                log::error!("forced halt");
            },
        }
    }};
}

pub async fn run_editor() -> Result<()> {
    expect_log("editor");

    let (done_tx, mut done_rx) = mpsc::channel::<i32>(1);
    {
        let (_shutdown0, rx) = oneshot::channel::<i32>();

        let done = done_tx.clone();
        tokio::spawn(async move {
            let _done = done;
            www::run_editor(rx).await;
        });
        let _ = tokio::signal::ctrl_c().await?;
        log::info!("Caught ^C, quiting");
    }
    drop(done_tx);

    graceful_shutdown!(done_rx);

    log::info!("editor stopped");
    Ok(())
}

pub async fn run() -> Result<()> {
    expect_log("elearn");

    let (done_tx, mut done_rx) = mpsc::channel::<i32>(1);
    {
        let (_shutdown0, rx) = oneshot::channel::<i32>();

        let done = done_tx.clone();
        tokio::spawn(async move {
            let _done = done;
            www::run(rx).await;
        });
        let _ = tokio::signal::ctrl_c().await?;
        log::info!("Caught ^C, quiting");
    }
    drop(done_tx);

    graceful_shutdown!(done_rx);

    log::info!("elearn stopped");
    Ok(())
}
