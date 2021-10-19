use error_chain::error_chain;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use tokio::sync::{mpsc, watch, oneshot};
use serde::ser::Serialize;
use serde::de::DeserializeOwned;
use std::fs::File;
use std::path::Path;

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

fn test_data_path<T: ToString, U: ToString>(typ: T, name: U) -> String {
    format!("{}/data/test/{}/{}.ron", VAR_PATH, typ.to_string(), name.to_string())
}

fn commit_test_data<T: ToString, U: ToString, V: Serialize>(typ: T, name: U, value: &V) -> Result<()> {
    let mut out = File::create(test_data_path(typ, name))?;
    ron::ser::to_writer(out, value)?;
    Ok(())
}

fn load_test_data<T: ToString, U: ToString, V: DeserializeOwned>(typ: T, name: U) -> Result<V> {
    let input = File::open(test_data_path(typ, name))?;
    Ok(ron::de::from_reader(input)?)
}

fn list_names_of_test_type<T: ToString>(typ: T) -> Vec<String> {
    let datapath = test_data_path(typ.to_string(), "none");
    let p = Path::new(&datapath).parent()
        .unwrap_or_else(|| panic!("Cannot get dir of test type {}", typ.to_string()));
    if !p.exists() {
        std::fs::create_dir_all(p).unwrap_or_else(|e| panic!("Create test type data dir {:?} error: {}", p, e));
    }
    let d = p.read_dir().unwrap_or_else(|e| panic!("Open test type data dir {:?} error: {}", p, e));
    d.filter_map(|result| match result {
        Ok(entry) => entry.file_name().into_string()
            .map_err(|e| log::error!("test name {:?} of test type {} cannot to utf string", e, typ.to_string()))
            .ok().and_then(|s| s.strip_suffix(".ron").map(String::from)),
        Err(e) => {
            log::error!("read dir {:?} error: {}", p, e);
            None
        }
    })
        .collect()
}

fn prepare_log<T: ToString>(tag: T) -> Result<()> {
        let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} # {M}/{l} - {P}:{I} # {m}{n}",
        )))
        .build(format!("{}/{}.log", VAR_PATH, tag.to_string()))?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    log::info!(
        "{} version {}; logger prepared",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    Ok(())
}

fn expect_log<T: ToString>(tag: T) {
    prepare_log(tag).unwrap_or_else(|e| panic!("prepare log fatal error(s): {}", e));
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

    println!("\nDoing graceful shutdown ...");
    println!("Press ^C again to halt");

    tokio::select! {
        _ = done_rx.recv() => {},
        _ = tokio::signal::ctrl_c() => {
            println!("\nforced halt");
            log::error!("forced halt");
        },
    };

    log::info!("editor stopped");
    Ok(())
}
