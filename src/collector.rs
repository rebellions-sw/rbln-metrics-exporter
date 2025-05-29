use anyhow::Result;
use log::{debug, error};
use metrics::gauge;
use phf::phf_map;
use tokio::task::JoinSet;

mod rblnservices {
    include!("./rblnservices.rs");
}

#[derive(strum_macros::Display, Debug)]
enum RBLNMetric {
    #[strum(to_string = "RBLN_DEVICE_STATUS:TEMPERATURE")]
    Temperature,
    #[strum(to_string = "RBLN_DEVICE_STATUS:CARD_POWER")]
    Power,
    #[strum(to_string = "RBLN_DEVICE_STATUS:DRAM_TOTAL")]
    DramTotal,
    #[strum(to_string = "RBLN_DEVICE_STATUS:DRAM_USED")]
    DramUsed,
    #[strum(to_string = "RBLN_DEVICE_STATUS:UTILIZATION")]
    Utilization,
}

static CARD_NAME_MAP: phf::Map<&str, &str> = phf_map! {
    "1020" => "RBLN-CA02",
    "1021" => "RBLN-CA02",
    "1120" => "RBLN-CA12",
    "1121" => "RBLN-CA12",
    "1150" => "RBLN-CA15",
    "1220" => "RBLN-CA22",
    "1221" => "RBLN-CA22",
    "1250" => "RBLN-CA25",
};

#[derive(Debug, Clone)]
pub struct RBLNMetricsCollector {
    daemon_url: String,
}

impl RBLNMetricsCollector {
    pub fn new(daemon_url: &str) -> Self {
        Self {
            daemon_url: daemon_url.into(),
        }
    }

    pub async fn collect(&self) -> Result<()> {
        let daemon_client_result = rblnservices::rbln_services_client::RblnServicesClient::connect(
            self.daemon_url.clone(),
        )
        .await;

        if let Err(e) = daemon_client_result {
            error!("Failed to initialize daemon client: {}", e);
            return Ok(());
        }

        let mut daemon_client = daemon_client_result.unwrap();

        let mut tasks = JoinSet::new();
        for device in Self::get_devices(&mut daemon_client).await {
            tasks.spawn({
                let mut daemon_client = daemon_client.clone();
                let device = device.clone();
                async move { Self::collect_hw_info(&mut daemon_client, &device).await }
            });
            tasks.spawn({
                let mut daemon_client = daemon_client.clone();
                let device = device.clone();
                async move { Self::collect_memory_info(&mut daemon_client, &device).await }
            });
            tasks.spawn({
                let mut daemon_client = daemon_client.clone();
                let device = device.clone();
                async move { Self::collect_utilization(&mut daemon_client, &device).await }
            });
        }
        tasks.join_all().await;

        Ok(())
    }

    async fn get_devices(
        daemon_client: &mut rblnservices::rbln_services_client::RblnServicesClient<
            tonic::transport::Channel,
        >,
    ) -> Vec<rblnservices::Device> {
        let mut devices = Vec::<rblnservices::Device>::new();
        match daemon_client
            .get_serviceable_device_list(rblnservices::Empty {})
            .await
        {
            Ok(mut resp) => {
                while let Some(device) = resp.get_mut().message().await.unwrap() {
                    devices.push(device);
                }
            }
            Err(e) => {
                error!("Failed to get serviceable devices: {}", e);
            }
        }
        devices
    }

    fn get_device_labels(device: &rblnservices::Device) -> Vec<(String, String)> {
        vec![
            (
                "card".into(),
                CARD_NAME_MAP
                    .get(device.dev_id.as_str())
                    .unwrap()
                    .to_string(),
            ),
            ("uuid".into(), device.uuid.clone()),
            ("device".into(), device.name.clone()),
        ]
    }

    fn report_metric(metric: RBLNMetric, value: f32, device: &rblnservices::Device) {
        let labels = Self::get_device_labels(device);
        gauge!(metric.to_string(), &labels).set(value);
        debug!("[{}] {}: {}", device.name, metric, value);
    }

    async fn collect_hw_info(
        daemon_client: &mut rblnservices::rbln_services_client::RblnServicesClient<
            tonic::transport::Channel,
        >,
        device: &rblnservices::Device,
    ) -> Result<()> {
        match daemon_client.get_hw_info(device.clone()).await {
            Ok(resp) => {
                let temperature = resp.get_ref().temperature;
                let power = resp.get_ref().watt;
                Self::report_metric(RBLNMetric::Temperature, temperature, device);
                Self::report_metric(RBLNMetric::Power, power, device);
            }
            Err(e) => {
                error!("Failed to get hw info of {}: {}", device.name, e);
            }
        }
        Ok(())
    }

    async fn collect_memory_info(
        daemon_client: &mut rblnservices::rbln_services_client::RblnServicesClient<
            tonic::transport::Channel,
        >,
        device: &rblnservices::Device,
    ) -> Result<()> {
        match daemon_client.get_memory_info(device.clone()).await {
            Ok(resp) => {
                let dram_total = resp.get_ref().total_mem;
                let dram_used = resp.get_ref().used_mem;
                Self::report_metric(RBLNMetric::DramTotal, dram_total, device);
                Self::report_metric(RBLNMetric::DramUsed, dram_used, device);
            }
            Err(e) => {
                error!("Failed to get memory info of {}: {}", device.name, e);
            }
        }
        Ok(())
    }

    async fn collect_utilization(
        daemon_client: &mut rblnservices::rbln_services_client::RblnServicesClient<
            tonic::transport::Channel,
        >,
        device: &rblnservices::Device,
    ) -> Result<()> {
        match daemon_client.get_utilization(device.clone()).await {
            Ok(resp) => {
                let util = resp.get_ref().utilization;
                Self::report_metric(RBLNMetric::Utilization, util, device);
            }
            Err(e) => {
                error!("Failed to get utilization of {}: {}", device.name, e);
            }
        }
        Ok(())
    }
}
