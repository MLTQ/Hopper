use hopper_core::{Config, Node};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut config = Config::desktop(Config::desktop_name())?;
    let mut imports = vec![];
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                config.data_dir = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--data-dir needs a path"))?
                    .into()
            }
            "--name" => {
                config.name = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--name needs a name"))?
            }
            "--local-only" => config.local_only = true,
            "--import" => imports.push(
                args.next()
                    .ok_or_else(|| anyhow::anyhow!("--import needs a file or folder"))?
                    .into(),
            ),
            "--help" => {
                println!(
                    "hopper-node [--data-dir PATH] [--name NAME] [--import PATH] [--local-only]\nThe local UI URL contains a private authorization fragment. Do not share it; use Pair device."
                );
                return Ok(());
            }
            _ => anyhow::bail!("unknown option {arg}"),
        }
    }
    let (node, hello) = Node::start(config).await?;
    if !imports.is_empty() {
        eprintln!("{}", serde_json::to_string(&node.import(imports).await?)?);
    }
    println!("{}", serde_json::to_string(&hello)?);
    eprintln!("Open Hopper: {}", hello.url());
    tokio::signal::ctrl_c().await?;
    node.close().await;
    Ok(())
}
