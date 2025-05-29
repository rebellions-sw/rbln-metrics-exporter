mod collector;

use chrono::{Duration, Local};
use clap::Parser;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddrV4;
use tokio::time::sleep;

use collector::RBLNMetricsCollector;

#[derive(Parser, Debug)]
#[command(version, about = "RBLN Metrics Exporter", long_about = None)]
struct CLIArgs {
    #[arg(
        long,
        value_name = "rbln-daemon-url",
        env = "RBLN_METRICS_EXPORTER_RBLN_DAEMON_URL",
        default_value = "http://[::1]:50051",
        help = "Endpoint to RBLN daemon grpc server"
    )]
    rbln_daemon_url: String,

    #[arg(
        long,
        value_name = "port",
        env = "RBLN_METRICS_EXPORTER_PORT",
        default_value = "9090",
        help = "Port to listen for requests"
    )]
    port: u32,

    #[arg(
        long,
        value_name = "seconds",
        env = "RBLN_METRICS_EXPORTER_INTERVAL",
        default_value_t = 5,
        value_parser = clap::value_parser!(u32).range(1..60),
        help = "Interval of collecting metrics (min: 1s, max: 60s)"
    )]
    interval: u32,

    #[arg(long, help = "Collect once and exit")]
    oneshot: bool,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = CLIArgs::parse();
    let collector = RBLNMetricsCollector::new(args.rbln_daemon_url.as_str());

    let socket = format!("0.0.0.0:{}", args.port)
        .parse::<SocketAddrV4>()
        .expect("Invalid socket address");
    PrometheusBuilder::new()
        .with_http_listener(socket)
        .install()
        .expect("Failed to install Prometheus recorder");

    if args.oneshot {
        let _ = collector.collect().await;
        return;
    }

    let interval = Duration::seconds(args.interval as i64);
    let mut next_time = Local::now() + interval;
    loop {
        let _ = collector.collect().await;
        sleep((next_time - Local::now()).to_std().unwrap()).await;
        next_time += interval;
    }
}
