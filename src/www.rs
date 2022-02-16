use crate::exam;
use serde_derive::{Deserialize, Serialize};
use tera::{Context as TeraContext, Tera};
use tokio::sync::oneshot;
use warp::Filter;

macro_rules! load_templates {
    (@inner $v:ident) => {};
    (@inner $v:ident, ) => {};
    (@inner $v:ident, $name:literal, $($tail:tt)*) => {
        {
            $v.push((concat!($name, ".html"), include_str!(concat!("../templates/", $name, ".html"))));
            load_templates!(@inner $v, $($tail)*);
        }
    };
    [$name:literal, $($tail:tt)*] => {
        {
            let mut v = Vec::default();
            load_templates!(@inner v, $name, $($tail)*);
            v
        }
    };
}

lazy_static::lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        let files = load_templates![
            "editor_clustering",
            "editor_index",
            "failure",
            "index",
            "list_history",
            "test_clustering",
            "test_name_option",
            "welcome",
            ];
        if let Err(e) = tera.add_raw_templates(files) {
            log::error!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
        tera
    };
}

macro_rules! render {
    (@errhtml $kind:expr, $reason:expr) => {
        warp::reply::html(render!(@err TEMPLATES, $kind, $reason))
    };
    (@err $tera:ident, $kind:expr, $reason:expr) => {
        {
            let mut ctx = TeraContext::new();
            ctx.insert("kind", $kind);
            ctx.insert("reason", $reason);
            log::error!("render error: kind {}, reason {}", $kind, $reason);
            $tera.render("failure.html", &ctx).unwrap()
        }
    };

    ($name:expr, $ctx:expr) => {
        render!(TEMPLATES, $name, $ctx)
    };
    ($tera:ident, $name:expr, $ctx:expr) => {
        warp::reply::html($tera.render($name, $ctx).unwrap_or_else(|e|
            render!(@err $tera, "Tera engine", &format!("Error: tera: {}", e))
        ))
    };
}

macro_rules! str_decode {
    (@assign $expr:expr, $e:ident, $return:expr) => {
        match urlencoding::decode(&$expr) {
            Ok(v) => v,
            Err($e) => {
                log::error!("urldecode {} error: {}", &$expr, &$e);
                return $return;
            }
        }
    };
    (@inner $ident:ident, $e:ident, $return:expr) => {
        let $ident = str_decode!(@assign $ident, $e, $return);
    };

    (@ajsn $expr:expr) => {
        str_decode!(@assign $expr, e, warp::reply::json(&format!("URLDecode错误：{}", &e)))
    };
    ($ident:ident) => {
        str_decode!(@inner $ident, e, render!(@errhtml "URLDecode", &e.to_string()))
    };
    (@jsn $ident:ident) => {
        str_decode!(@inner $ident, e, warp::reply::json(&format!("URLDecode错误：{}", &e)))
    };
}

macro_rules! load_exam {
    ($typ:expr, $name:expr) => {
        match super::load_test_data($typ, $name) {
            Ok(v) => v,
            Err(e) => {
                log::error!("load exam data {}/{} error: {}", $typ, $name, &e);
                return render!(@errhtml "加载题库", &format!("加载数据错误：{}", e));
            }
        }
    };
}

#[derive(Deserialize, Serialize)]
struct SaveHistory {
    pub typ: String,
    pub name: String,
    pub data: String,
}

pub async fn run_editor(shutdown: oneshot::Receiver<i32>) {
    let index = warp::path::end().map(|| render!("editor_index.html", &TeraContext::new()));

    let editor = warp::path("editor");
    let editor_clustering = warp::path!("1" / String).map(|name: String| {
        str_decode!(name);
        let mut ctx = TeraContext::new();
        let data: exam::ClusteringExam = super::load_test_data("1", &name).unwrap_or_else(|e| {
            log::error!("load ClusteringExam of {} error: {}", name, e);
            Default::default()
        });
        let col = data.column_count();
        let tbl = data.table();
        ctx.insert("data", &tbl);
        ctx.insert("column", &col);
        render!("editor_clustering.html", &ctx)
    });

    let card = warp::path("card");
    let card_testnameopts = warp::path!("test_name_option" / String).map(|typ| {
        let mut ctx = TeraContext::new();
        let names = super::list_names_of_test_type(typ);
        ctx.insert("opts", &names);
        render!("test_name_option.html", &ctx)
    });

    let submit = warp::path("submit");
    let submit_clustering = warp::path!("1" / String)
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|name: String, opt: exam::ClusteringExam| {
            str_decode!(@jsn name);
            let desc = match super::commit_test_data("1", &name, &opt) {
                Ok(_) => {
                    log::info!("save test type clustering {}", &name);
                    String::from("保存成功")
                }
                Err(e) => {
                    log::error!("save test type clustering {} error: {}", &name, &e);
                    format!("保存出错：{}", e)
                }
            };
            warp::reply::json(&desc)
        });

    let static_files = warp::path("static").and(warp::fs::dir(var_path!(@static)));
    let favicon = warp::path!("favicon.ico").and(warp::fs::file(concat!(var_path!(@static), "/favicon.ico")));

    let app = index
        .or(editor.and(editor_clustering))
        .or(card.and(card_testnameopts))
        .or(submit.and(submit_clustering))
        .or(static_files)
        .or(favicon);

    log::info!("www running");
    let (_, run) = warp::serve(app).bind_with_graceful_shutdown(([0, 0, 0, 0], 3732), async move {
        shutdown.await.ok();
        log::info!("graceful shutdown");
    });
    run.await;
    log::info!("www stopped");
}

pub async fn run(shutdown: oneshot::Receiver<i32>) {
    let index = warp::path::end().map(|| render!("index.html", &TeraContext::new()));

    let card = warp::path("card");
    let card_testnameopts = warp::path!("test_name_option" / String).map(|typ| {
        let mut ctx = TeraContext::new();
        let names = super::list_names_of_test_type(typ);
        ctx.insert("opts", &names);
        render!("test_name_option.html", &ctx)
    });

    let welcome = warp::path!("welcome" / String / String).map(|typ, name| {
        let mut ctx = TeraContext::new();
        ctx.insert("type", &typ);
        ctx.insert("name", &name);
        render!("welcome.html", &ctx)
    });

    let exam = warp::path("exam");
    let exam_clustering = warp::path!("1" / String / usize).map(|name: String, count| {
        str_decode!(name);
        let data: exam::ClusteringExam = load_exam!("1", &name);
        let data = data.gen_probs(count);
        let mut ctx = TeraContext::new();
        ctx.insert("item_count", &count);
        ctx.insert("data", &data);
        render!("test_clustering.html", &ctx)
    });

    let save_history = warp::path!("save_history")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 1024 * 15))
        .and(warp::body::json())
        .map(|opt: SaveHistory| {
            let name = str_decode!(@ajsn opt.name);
            let desc = match super::commit_history(&opt.typ, &name, &opt.data) {
                Ok(_) => String::from("保存成功"),
                Err(e) => {
                    log::error!("commit history of {}/{} error: {}", &opt.typ, &name, &e);
                    format!("保存历史出错：{}", e)
                }
            };
            warp::reply::json(&desc)
        });

    let list_history =
        warp::path!("list_history" / String / String).map(|typ: String, name: String| {
            str_decode!(name);
            str_decode!(typ);
            let mut ctx = TeraContext::new();
            let mut v = super::list_history_of_kind(&typ, &name);
            v.sort();
            ctx.insert("data", &v);
            render!("list_history.html", &ctx)
        });

    let static_history = warp::path("history").and(warp::fs::dir(super::history_root()));

    let static_files = warp::path("static").and(warp::fs::dir(var_path!(@static)));
    let favicon = warp::path!("favicon.ico").and(warp::fs::file(concat!(var_path!(@static), "/favicon.ico")));

    let app = index
        .or(welcome)
        .or(card.and(card_testnameopts))
        .or(exam.and(exam_clustering))
        .or(save_history)
        .or(list_history)
        .or(static_history)
        .or(static_files)
        .or(favicon);

    log::info!("www running");
    let (_, run) = warp::serve(app).bind_with_graceful_shutdown(([0, 0, 0, 0], 3733), async move {
        shutdown.await.ok();
        log::info!("graceful shutdown");
    });
    run.await;
    log::info!("www stopped");
}
