use error_chain::error_chain;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use tokio::sync::{mpsc, watch, oneshot};

mod exam;
mod www;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        InitLog(log::SetLoggerError);
        ConfigLog(log4rs::config::runtime::ConfigErrors);
    }
}

const VAR_PATH: &str = "/var/lifeich1/elearn";

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
