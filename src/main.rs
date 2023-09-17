use exsb::Cli;

#[tokio::main]
async fn main() -> exsb::Result<()> {
    Cli::execute().await
}
