#![cfg_attr(
all(not(debug_assertions), target_os = "windows"),
windows_subsystem = "windows"
)]

use std::time::Duration;
use lazy_static::lazy_static;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::producer::{BaseProducer, BaseRecord};


use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance, StreamConsumer};
use rdkafka::{ClientContext, Message, TopicPartitionList};
use rdkafka::error::KafkaResult;
use serde_json::json;
use tauri::{Manager, Window};


#[derive(Clone, serde::Serialize)]
struct Payload {
  message: String,
}


struct CustomContext;

impl ClientContext for CustomContext {}

impl ConsumerContext for CustomContext {
  fn pre_rebalance(&self, rebalance: &Rebalance) {
    println!("Pre rebalance {:?}", rebalance);
  }

  fn post_rebalance(&self, rebalance: &Rebalance) {
    println!("Post rebalance {:?}", rebalance);
  }

  fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
    println!("Committing offsets: {:?}", result);
  }
}

type LoggingConsumer = StreamConsumer<CustomContext>;


lazy_static! {

    static ref CONSUMER: LoggingConsumer = ClientConfig::new()
        .set("group.id", "test3")
        .set("bootstrap.servers", "localhost:9092")
        .set("enable.auto.commit", "true")
        // .set("auto.offset.reset", "earliest")
        .set_log_level(RDKafkaLogLevel::Info)
        .create_with_context(CustomContext)

        .expect("Consumer creation failed");

    static ref PRODUCER: BaseProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .create()
        .expect("invalid producer config");
}





fn main() {
  tauri::Builder::default()
      .invoke_handler(tauri::generate_handler![
            greet,
            send_message,
            consume,
            stop_consumer,
            list_topics
        ])
      .run(tauri::generate_context!())
      .expect("error while running tauri application");
}

#[tauri::command]
fn greet(name: &str) -> String {
  format!("Hello, {}!", name)
}

#[tauri::command]
fn send_message(num_of_messages: i32) -> String {
  for i in 1..num_of_messages {
    println!("sending message");
    PRODUCER
        .send(
          BaseRecord::to("rust")
              .key(&format!("key-{}", i))
              .payload(&format!("value-{}", i)),
        )
        .expect("failed to send message");
  }


  return format!("sent {} to rust topic", num_of_messages);
}


//Consume and display incoming message on one topic
#[tauri::command]
async fn consume(window: Window) {
  println!("Created Consumer");
  CONSUMER.subscribe(&["rust"])
      .expect("Can't subscribe to specified topics");
  println!("Subscribed");

  loop {
    match CONSUMER.recv().await {
      Err(e) => println!("Kafka error: {}", e),
      Ok(m) => {
        let payload = match m.payload_view::<str>() {
          None => "",
          Some(Ok(s)) => s,
          Some(Err(e)) => {
            println!("Error while deserializing message payload: {:?}", e);
            ""
          }
        };

        let outbound_message = json!({
                    "key": m.key(),
                    "payload": payload,
                    "topic": m.topic(),
                    "partition": m.partition(),
                    "offset": m.offset(),
                    "timestamp": m.timestamp().to_millis()
                });

        window.emit("update-message", Payload {message: outbound_message.to_string()}).unwrap();
        println!("{}", outbound_message.to_string());
      }
    };
  }
}


#[tauri::command]
fn stop_consumer() {
  CONSUMER.unsubscribe()
}

#[tauri::command]
fn list_topics() -> Vec<String> {
  let mut topic_list: Vec<String> = Vec::new();

  let metadata = CONSUMER.fetch_metadata(None, Duration::from_secs(60))
      .expect("Failed to fetch metadata");

  let topics = metadata.topics();

  println!("  Topics count: {}", topics.len());
  for topic in topics {
    println!("\t{}", topic.name());
    topic_list.push(topic.name().parse().unwrap())
  }

  return topic_list;
}


