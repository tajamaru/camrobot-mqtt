mod robo;

use crate::robo::{Action,Robo};
use tokio::sync::mpsc::{channel,Sender};
use tokio::{task};

use futures_util::stream::StreamExt;
use rumq_client::{MqttOptions,eventloop,MqttEventLoop,Request,QoS,Notification};

use std::fs;

#[tokio::main(basic_scheduler)]
async fn main() {
    let  mqttoptions = set_mqtt_options();
    let (requests_tx,requests_rx) = channel(100);
    let mut eventloop = eventloop(mqttoptions, requests_rx);
    subsclibe_action(requests_tx.clone()).await;

    let (tx,rx) = channel(10);

    task::spawn(async move{
    stream_it(&mut eventloop, tx).await;
    });
    if let Ok(mut robo) = Robo::new(){
        robo.wakeup(rx).await;
    }

}
async fn subsclibe_action(mut request: Sender<Request>){
    task::spawn(async move {
        let topic = "camrobot/action".to_owned();
        let subsclibe = rumq_client::subscribe(&topic, QoS::AtLeastOnce);
        let _ = request.send(Request::Subscribe(subsclibe)).await;
    });
}


fn set_mqtt_options() -> MqttOptions{
    let ca = fs::read("certs/AmazonRootCA1.pem").unwrap();
    let client_cert = fs::read("certs/xxx.pem.crt").unwrap();
    let client_key = fs::read("certs/xxx.pem.key").unwrap();

    let mut mqttoptions = MqttOptions::new("raspai","xxx.amazonaws.com", 8883);
    mqttoptions.set_ca(ca);
    mqttoptions.set_client_auth(client_cert, client_key);
    mqttoptions.set_keep_alive(100);
    mqttoptions
}

async fn stream_it(eventloop: &mut MqttEventLoop,mut sender: Sender<Action>) {
    let mut stream = eventloop.stream();

    while let Some(item) = stream.next().await {
        if let Notification::Publish(data) = item{
            let a:Action = serde_json::from_slice(&data.payload).unwrap();
            println!("{:?}",data);
            let _ = sender.send(a).await;
        }else{
            println!("{:?}",item);
        }
    }
    
    println!("Stream done");
}

