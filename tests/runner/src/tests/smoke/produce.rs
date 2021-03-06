use std::time::Duration;
use flv_future_aio::timer::sleep;
use flv_client::profile::ScConfig;
use flv_client::SpuController;
use flv_client::ReplicaLeader;

use crate::util::CommandUtil;


/// produce message
#[allow(unused)]
pub fn produce_message_with_api() {

    use flv_future_aio::task::run_block_on;
    
    run_block_on(async {

        sleep(Duration::from_secs(2)).await;

        let config = ScConfig::new(Some("localhost:9003".into()),None).expect("connect");
        let mut sc = config.connect().await.expect("should connect");

        let mut leader = sc.find_replica_for_topic_partition("test1",0).await.expect("leader not founded");

        let message = "hello world".to_owned().into_bytes();

        leader.send_record(message).await.expect("message sent");

        println!("message produced");
    });
}

use std::io::Write;
use std::io;
use std::process::Stdio;

use utils::bin::get_fluvio;

pub fn produce_message_with_cli() {

    println!("starting produce");
    let mut child = get_fluvio()
            .expect("no fluvio")
            .stdin(Stdio::piped())
            .arg("produce")
            .arg("test1")
            .print()
            .spawn()
            .expect("no child");
                  

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    stdin.write_all("hello world".as_bytes()).expect("Failed to write to stdin");

    let output = child.wait_with_output().expect("Failed to read stdout");

    io::stdout().write_all(&output.stdout).unwrap();
    io::stderr().write_all(&output.stderr).unwrap();

    println!("produce message: hello world");

}
