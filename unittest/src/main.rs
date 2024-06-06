use candid::{Encode, Principal};
use ic_agent::Agent;
use tempdir::TempDir;
use std::{fs::{read_to_string, File}, path::{Path, PathBuf}, process::Command};
use like_shell::{run_successful_command, temp_dir_from_template, Capture, TemporaryChild};
use anyhow::Context;
use serde_json::Value;

struct Test {
    dir: TempDir,
    // cargo_manifest_dir: PathBuf,
    workspace_dir: PathBuf,
}

impl Test {
    pub async fn new(tmpl_dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let cargo_manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_dir = cargo_manifest_dir.join("..").join("..");
        let dir = temp_dir_from_template(tmpl_dir)?;

        let res = Self {
            dir,
            // cargo_manifest_dir: cargo_manifest_dir.to_path_buf(),
            workspace_dir: workspace_dir,
        };

        Ok(res)
    }
}

// TODO: Should have more abstract DFXDir.
struct OurDFX<'a> {
    pub base: &'a Test,
    pub test_canister_id: Principal,
    pub agent: Agent,
}

impl<'a> OurDFX<'a> {
    pub async fn new(base: &'a Test, additional_args: &[&str]) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Specifying a specific port is a hack.
        run_successful_command(&mut Command::new(
            "/root/.local/share/dfx/bin/dfx"
        ).args([&["start", "--host", "127.0.0.1:8007", "--background"] as &[&str], additional_args].concat()).current_dir(base.dir.path()))
            .context("Starting DFX")?;

        let port_str = read_to_string(
            base.dir.path().join(".dfx").join("network").join("local").join("webserver-port"),
        ).context("Reading port.")?;
        let port: u16 = port_str.parse().context("Parsing port number.")?;

        println!("Connecting to DFX (port {port})");
        run_successful_command(Command::new(
            "/root/.local/share/dfx/bin/dfx" // TODO: Split base.dir.path().
        ).args(["deploy"]))?;
        // dotenv().ok();

        let canister_ids: Value = {
            let dir = base.dir.path().join(".dfx").join("local").join("canister_ids.json");
            let file = File::open(dir).with_context(|| format!("Opening canister_ids.json"))?;
            serde_json::from_reader(file).expect("Error parsing JSON")
        };
        let test_canister_id = canister_ids.as_object().unwrap()["test"].as_object().unwrap()["local"].as_str().unwrap();

        let agent = Agent::builder().with_url(format!("http://127.0.0.1:{port}")).build().context("Creating Agent")?;
        agent.fetch_root_key().await.context("Fetching root keys.")?; // DON'T USE this on mainnet

        Ok(Self {
            base: &base,
            test_canister_id: Principal::from_text(test_canister_id)
                .context("Parsing principal")?,
            agent,
        })
    }
}

impl<'a> Drop for OurDFX<'a> {
    fn drop(&mut self) {
        run_successful_command(&mut Command::new(
            "/root/.local/share/dfx/bin/dfx" // TODO: Split path.
        ).args(["stop"]).current_dir(self.base.dir.path()))
            .context("Stopping DFX").expect("can't stop DFX");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test = Test::new(&Path::new(".")).await?;
    let dfx = OurDFX::new(&test, &[]).await?;
    run_successful_command(Command::new("dfx").arg("deploy"))?.context("running dfx deploy");

    let res = dfx.agent.update(&dfx.test_canister_id, "test").with_arg(Encode!()?)
        .call_and_wait().await.context("Call to IC.")?;
    let res = Decode!(res, String)?;
    let count = res.matches("x-my").count();
    assert_eq!(count, 2);

    Ok(())
}
