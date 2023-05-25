use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use netvr_calibrate::{invert_quaternion, rotate_vector, CalibrationInput};
use netvr_data::{
    net::{
        BaseSpace, CalibrationConfiguration, CalibrationSample, ClientId,
        ConfigurationDown::{
            RequestSample, SetServerSpacePose, StopCalibration, TriggerCalibration,
        },
    },
    Pose, Vec3,
};
use std::io::Write;
use std::{
    fs::File,
    time::{Duration, SystemTime},
    vec,
};
use tokio::{
    select,
    sync::{broadcast, mpsc},
};

use self::CalibrationProtocolMessage::*;
use crate::{client::Client, dashboard::DashboardMessage, server::Server};

pub(crate) struct CalibrationProtocol {
    recv: mpsc::UnboundedReceiver<CalibrationProtocolMessage>,
}

pub(crate) type CalibrationSender = mpsc::UnboundedSender<CalibrationProtocolMessage>;

#[derive(Debug, Clone)]
pub(crate) enum CalibrationProtocolMessage {
    Begin {
        client_target: (ClientId, String),
        client_reference: (ClientId, String),
        conf: CalibrationConfiguration,
    },
    Hijack {
        client_target: (ClientId, String),
        client_reference: (ClientId, String),
    },
    FinishCalibration,
    Sample {
        client: ClientId,
        sample: CalibrationSample,
    },
    Reapply {
        client_target: (ClientId, String),
        client_reference: (ClientId, String),
        data: CalibrationInput,
    },
    ByHeadset,
}

impl CalibrationProtocol {
    pub(crate) fn new() -> (Self, CalibrationSender) {
        let (sender, recv) = mpsc::unbounded_channel();
        (Self { recv }, sender)
    }

    pub(crate) async fn run(
        self,
        server: Server,
        tx: broadcast::Sender<DashboardMessage>,
    ) -> Result<()> {
        run(self.recv, server, tx).await
    }
}

async fn run(
    mut recv: mpsc::UnboundedReceiver<CalibrationProtocolMessage>,
    server: Server,
    tx: broadcast::Sender<DashboardMessage>,
) -> Result<()> {
    loop {
        let Some(instruction) = recv.recv().await else { break; };
        println!("Received calibration instruction {:?}", instruction);
        let (client_target, client_reference, conf) = match instruction {
            Begin {
                client_target,
                client_reference,
                conf,
            } => (client_target, client_reference, conf),
            Hijack {
                client_target,
                client_reference,
            } => {
                let conf = CalibrationConfiguration {
                    sample_count: 50 * 3600,
                    sample_interval_nanos: 20_000_000,
                };
                run_calibration(
                    &mut recv,
                    client_target,
                    client_reference,
                    server.clone(),
                    tx.clone(),
                    conf,
                    false,
                )
                .await;
                continue;
            }
            Sample { .. } => continue,
            FinishCalibration => continue,
            Reapply {
                client_target,
                client_reference,
                data,
            } => {
                let Some(client_target) = server.get_client(client_target.0).await else { continue; };
                let Some(client_reference) = server.get_client(client_reference.0).await else { continue; };
                finish(tx.clone(), data, &client_target, &client_reference, true).await;
                continue;
            }
            ByHeadset => {
                let clients = server.get_clients().await;
                match calibrate_by_headset(clients, &mut recv).await {
                    Ok(_) => println!("Calibrated by headset"),
                    Err(err) => println!("Failed to calibrate by headset: {:?}", err),
                }

                continue;
            }
        };
        run_calibration(
            &mut recv,
            client_target,
            client_reference,
            server.clone(),
            tx.clone(),
            conf,
            true,
        )
        .await;
    }
    println!("Calibration protocol stopped");
    Ok(())
}

async fn run_calibration(
    recv: &mut mpsc::UnboundedReceiver<CalibrationProtocolMessage>,
    client_target: (ClientId, String),
    client_reference: (ClientId, String),
    server: Server,
    tx: broadcast::Sender<DashboardMessage>,
    conf: CalibrationConfiguration,
    apply: bool,
) {
    let client_target_id = client_target.0;
    let client_reference_id = client_reference.0;
    let client_target_path = client_target.1;
    let client_reference_path = client_reference.1;

    let Some(client_target) = server.get_client(client_target.0).await else { return ; };
    let Some(client_reference) = server.get_client(client_reference.0).await else { return ; };
    if let Err(err) = client_target.send_configuration_down(TriggerCalibration(
        client_target_path,
        conf,
        BaseSpace::Stage,
    )) {
        println!(
            "Failed to send trigger calibration to client {:?}: {:?}",
            client_target_id, err
        );
        return;
    }
    if let Err(err) = client_reference.send_configuration_down(TriggerCalibration(
        client_reference_path,
        conf,
        BaseSpace::Server,
    )) {
        println!(
            "Failed to send trigger calibration to client {:?}: {:?}",
            client_reference_id, err
        );
        let _ = client_target.send_configuration_down(StopCalibration);
        return;
    }

    let samples_result = collect_samples(
        conf.sample_count,
        client_target_id,
        client_reference_id,
        recv,
    )
    .await;
    // Send end to clients
    for id in [client_target_id, client_reference_id] {
        if let Some(client) = server.get_client(id).await {
            if let Err(err) = client.send_configuration_down(StopCalibration) {
                println!(
                    "Failed to send stop calibration to client {:?}: {:?}",
                    id, err
                );
            }
        }
    }
    let (samples_target, samples_reference) = match samples_result {
        Ok(v) => v,
        Err(err) => {
            println!("Calibration failed: {:?}", err);
            return;
        }
    };

    // Samples collected. Calibrate and apply
    println!("samples_target: {:?}", samples_target.len());
    println!("samples_reference: {:?}", samples_target.len());
    let configuration = server.latest_configuration().await;
    let configuration = configuration.borrow();
    let calibration = CalibrationInput {
        target: samples_target,
        target_name: configuration
            .clients
            .get(&client_target_id)
            .map(|v| v.name.clone())
            .unwrap_or_default(),
        reference: samples_reference,
        reference_name: configuration
            .clients
            .get(&client_reference_id)
            .map(|v| v.name.clone())
            .unwrap_or_default(),
    };
    match serde_json::to_string(&calibration) {
        Ok(data) => {
            let dt: DateTime<Utc> = SystemTime::now().into();
            let fname = format!("calibration-data-{}.json", dt.format("%Y-%m-%dT%H-%M-%S"));
            match File::create(fname.clone()) {
                Ok(mut f) => match f.write_all(data.as_bytes()) {
                    Ok(()) => println!("Calibration data written to file: {}", fname),
                    Err(err) => println!("Failed to write calibration data to file: {:?}", err),
                },
                Err(err) => println!("Failed to create calibration file: {:?}", err),
            }
        }
        Err(err) => println!("Failed to serialize calibration data: {:?}", err),
    };
    finish(
        tx.clone(),
        calibration,
        &client_target,
        &client_reference,
        apply,
    )
    .await;
}

async fn calibrate_by_headset(
    clients: Vec<(u32, Client)>,
    recv: &mut mpsc::UnboundedReceiver<CalibrationProtocolMessage>,
) -> Result<()> {
    for (client_id, client) in clients {
        client
            .send_configuration_down(RequestSample("/user/head".to_string(), BaseSpace::Stage))?;
        let Some(sample) = select!(
            sample = recv_sample(client_id, recv) => sample,
            _ = tokio::time::sleep(Duration::from_secs(1)) => None,
        ) else { continue; };
        client.send_configuration_down(SetServerSpacePose(Pose {
            position: Vec3 {
                y: 0.0,
                ..sample.pose.position
            },
            orientation: sample.pose.orientation,
        }))?;
    }
    Ok(())
}

async fn recv_sample(
    client_id: ClientId,
    recv: &mut mpsc::UnboundedReceiver<CalibrationProtocolMessage>,
) -> Option<CalibrationSample> {
    loop {
        let message = recv.recv().await;
        let Some(message) = message else { break; };
        if let CalibrationProtocolMessage::Sample { client, sample } = message {
            if client == client_id {
                return Some(sample);
            }
        };
    }
    None
}

async fn finish(
    tx: broadcast::Sender<DashboardMessage>,
    input: CalibrationInput,
    target: &Client,
    reference: &Client,
    apply: bool,
) {
    let result = netvr_calibrate::calibrate(&input);
    println!("Calibration result: {:?}", result);
    let _ = tx.send(DashboardMessage::Info {
        message: format!("Calibration finished: {:?}", result),
    });
    let Ok(data) = result else { return; };
    if apply {
        if let Err(res) = target.send_configuration_down(SetServerSpacePose(Pose {
            position: rotate_vector(
                Vec3 {
                    x: -data.translation.x,
                    y: -data.translation.y,
                    z: -data.translation.z,
                },
                invert_quaternion(data.rotation.clone()),
            ),
            orientation: invert_quaternion(data.rotation),
        })) {
            println!("Failed to send stage pose to target: {:?}", res);
        }
    }
}

async fn collect_samples(
    sample_count: usize,
    client_target: ClientId,
    client_reference: ClientId,
    recv: &mut mpsc::UnboundedReceiver<CalibrationProtocolMessage>,
) -> Result<(Vec<CalibrationSample>, Vec<CalibrationSample>)> {
    let mut samples_target = vec![];
    let mut samples_reference = vec![];
    let time_start = std::time::Instant::now();
    loop {
        let Some(sample) = recv.recv().await else { break; };
        if let FinishCalibration = sample {
            break;
        }
        let Sample { client, sample } = sample else { continue; };
        if client == client_target {
            samples_target.push(sample);
        } else if client == client_reference {
            samples_reference.push(sample);
        }
        if samples_target.len() >= sample_count && samples_reference.len() >= sample_count {
            break;
        }
        if time_start.elapsed().as_secs() > 60 {
            Err(anyhow!("Timed out waiting for samples"))?
        }
    }
    Ok((samples_target, samples_reference))
}
