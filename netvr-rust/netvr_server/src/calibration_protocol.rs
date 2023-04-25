use std::vec;

use anyhow::{anyhow, Result};
use netvr_data::net::{
    CalibrationSample, ClientId,
    ConfigurationDown::{StopCalibration, TriggerCalibration},
};
use tokio::sync::mpsc;

use self::CalibrationProtocolMessage::*;
use crate::server::Server;

pub(crate) struct CalibrationProtocol {
    recv: mpsc::UnboundedReceiver<CalibrationProtocolMessage>,
}

pub(crate) type CalibrationSender = mpsc::UnboundedSender<CalibrationProtocolMessage>;

#[derive(Debug, Clone)]
pub(crate) enum CalibrationProtocolMessage {
    Begin {
        clients: ((ClientId, String), (ClientId, String)),
        sample_count: usize,
    },
    Sample {
        client: ClientId,
        sample: CalibrationSample,
    },
}

impl CalibrationProtocol {
    pub(crate) fn new() -> (Self, CalibrationSender) {
        let (sender, recv) = mpsc::unbounded_channel();
        (Self { recv }, sender)
    }

    pub(crate) async fn run(self, server: Server) -> Result<()> {
        run(self.recv, server).await
    }
}

async fn run(
    mut recv: mpsc::UnboundedReceiver<CalibrationProtocolMessage>,
    server: Server,
) -> Result<()> {
    loop {
        let Some(instruction) = recv.recv().await else { break; };
        let Begin { clients, sample_count } = instruction else { continue; };

        let Some(client1) = server.get_client(clients.0.0).await else { continue; };
        let Some(client2) = server.get_client(clients.1.0).await else { continue; };
        if let Err(err) = client1.send_configuration_down(TriggerCalibration(clients.0 .1)) {
            println!(
                "Failed to send trigger calibration to client {:?}: {:?}",
                clients.0 .0, err
            );
            continue;
        }
        if let Err(err) = client2.send_configuration_down(TriggerCalibration(clients.1 .1)) {
            println!(
                "Failed to send trigger calibration to client {:?}: {:?}",
                clients.1 .0, err
            );
            let _ = client1.send_configuration_down(StopCalibration);
            continue;
        }

        let clients = (clients.0 .0, clients.1 .0);
        let samples_result = collect_samples(sample_count, clients, &mut recv).await;
        // Send end to clients
        for id in [clients.0, clients.1] {
            if let Some(client) = server.get_client(id).await {
                if let Err(err) = client.send_configuration_down(StopCalibration) {
                    println!(
                        "Failed to send stop calibration to client {:?}: {:?}",
                        id, err
                    );
                }
            }
        }
        let (samples1, samples2) = match samples_result {
            Ok(v) => v,
            Err(err) => {
                println!("Calibration failed: {:?}", err);
                continue;
            }
        };

        // Samples collected. Calibrate and apply
        println!("Samples 1: {:?}", samples1);
        println!("Samples 2: {:?}", samples2);
    }
    Ok(())
}

async fn collect_samples(
    sample_count: usize,
    clients: (ClientId, ClientId),
    recv: &mut mpsc::UnboundedReceiver<CalibrationProtocolMessage>,
) -> Result<(Vec<CalibrationSample>, Vec<CalibrationSample>)> {
    let mut samples1 = vec![];
    let mut samples2 = vec![];
    let time_start = std::time::Instant::now();
    loop {
        let Some(sample) = recv.recv().await else { break; };
        let Sample { client, sample } = sample else { continue; };
        if client == clients.0 {
            samples1.push(sample);
        } else if client == clients.1 {
            samples2.push(sample);
        }
        if samples1.len() >= sample_count && samples2.len() >= sample_count {
            break;
        }
        if time_start.elapsed().as_secs() > 60 {
            Err(anyhow!("Timed out waiting for samples"))?
        }
    }
    Ok((samples1, samples2))
}
