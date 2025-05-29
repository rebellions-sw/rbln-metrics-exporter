# RBLN Metrics Exporter

This repository contains Rebellions' NPU metrics exporter for [Prometheus](https://prometheus.io/).

## Prerequisites

- RBLN driver >= 1.3.40

## Command line interface

```bash
$ rbln-metrics-exporter --help
RBLN Metrics Exporter

Usage: rbln-metrics-exporter [OPTIONS]

Options:
      --rbln-daemon-url <rbln-daemon-url>
          Endpoint to RBLN daemon grpc server [env: RBLN_METRICS_EXPORTER_RBLN_DAEMON_URL=] [default: http://[::1]:50051]
      --port <port>
          Port to listen for requests [env: RBLN_METRICS_EXPORTER_PORT=] [default: 9090]
      --interval <seconds>
          Interval of collecting metrics (min: 1s, max: 60s) [env: RBLN_METRICS_EXPORTER_INTERVAL=] [default: 5]
      --oneshot
          Collect once and exit
  -h, --help
          Print help
  -V, --version
          Print version
```

## Metrics

The below metrics are exported for each NPU devices, labeled with device UUID, card name and character device node (`rblnN`).

|Name|Description|
|----|-----------|
|`RBLN_DEVICE_STATUS:TEMPERATURE`|temperature (°C)|
|`RBLN_DEVICE_STATUS:CARD_POWER`|power usages (W)|
|`RBLN_DEVICE_STATUS:DRAM_USED`|DRAM in use (GiB)|
|`RBLN_DEVICE_STATUS:DRAM_TOTAL`|DRAM total (GiB)|
|`RBLN_DEVICE_STATUS:UTILIZATION`|utilization (%)|
