use std::time::Duration;

use actix_web::{get, App, HttpRequest, HttpResponse, HttpServer, Responder};
use bangumi_rss_proxy::mods::{request::async_get_bytes, types::Config};
use futures::join;
use log::info;
use qstring::QString;

lazy_static::lazy_static! {
    static ref CONFIG: Config = serde_json::from_str(
        &std::fs::read_to_string("config.json").expect("Failed to read config.json")
    ).expect("Failed to parse config.json");
    static ref DOMAIN: String = CONFIG.domain.clone();
    static ref HTTP_PORT: u16 = CONFIG.http_port;
}

#[get("/RSS/MyBangumi")]
// /RSS/MyBangumi?token=
async fn rss_mybangumi(req: HttpRequest) -> impl Responder {
    let query_string = req.query_string();
    let query = QString::from(query_string);
    let token = match query.get("token") {
        Some(token) => token,
        None => {
            return HttpResponse::BadRequest()
                .content_type("application/xml")
                .body("[Error] Invalid Request: Missing token");
        }
    };
    let raw_rss_data = match bangumi_rss_proxy::mods::rss::get_mybangumi_rss(token).await {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("application/xml")
                .body(format!("[Error] Internal Server Error: {:?}", e));
        }
    };

    let rss_data =
        match bangumi_rss_proxy::mods::rss::edit_mybangumi_rss(&raw_rss_data, &DOMAIN).await {
            Ok(data) => data,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .content_type("application/xml")
                    .body(format!("[Error] Internal Server Error: {:?}", e));
            }
        };
    HttpResponse::Ok()
        .content_type("application/xml")
        .body(rss_data)
}

#[get("/Download/{date}/{filename:.+}")]
async fn torrent_download(req: HttpRequest) -> impl Responder {
    let date = req.match_info().get("date").unwrap();
    let filename = req.match_info().get("filename").unwrap();

    let torrent_url = format!("https://mikanani.me/Download/{}/{}", date, filename);

    let torrent_data = match async_get_bytes(&torrent_url).await {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .content_type("plaintext")
                .body(format!("[Error] Internal Server Error: {:?}", e));
        }
    };

    HttpResponse::Ok()
        .content_type("application/octet-stream")
        .body(torrent_data)
}

fn main() -> std::io::Result<()> {
    // init log
    use chrono::Local;
    use std::io::Write;
    let rt = tokio::runtime::Runtime::new().unwrap();
    let env = env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}][{:>5}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                buf.default_styled_level(record.level()),
                &record.args()
            )
        })
        .init();

    info!("Starting bangumi-rss-server");

    let web_main =
        HttpServer::new(move || App::new().service(rss_mybangumi).service(torrent_download))
            .bind(("0.0.0.0", HTTP_PORT.clone()))
            .unwrap()
            .keep_alive(Duration::from_secs(20))
            .run();
    rt.block_on(async { join!(web_main).0 }) // 以后可能会加些其它的东西
}
