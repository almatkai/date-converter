#[tokio::main(worker_threads = 2)]
async fn main() {
    if let Err(e) =
        tezbar_extension_sdk::serve(date_converter_v2::adapter::Adapter::default()).await
    {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
