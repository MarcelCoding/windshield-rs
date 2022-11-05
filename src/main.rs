use windshield_rs::run;

#[tokio::main(flavor = "current_thread")]
async fn main() {
  tracing_subscriber::fmt::init();
  run().await;
}
