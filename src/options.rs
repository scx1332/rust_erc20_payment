use std::fmt::Debug;

use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about = "Payment admin tool - run options")]
pub struct RunOptions {
    #[structopt(
        long = "keep-running",
        help = "Set to keep running when finished processing transactions"
    )]
    pub keep_running: bool,

    #[structopt(
        long = "generate-tx-only",
        help = "Do not send or process transactions, only generate stubs"
    )]
    pub generate_tx_only: bool,

    #[structopt(
        long = "skip-multi-contract-check",
        help = "Skip multi contract check when generating txs"
    )]
    pub skip_multi_contract_check: bool,

    #[structopt(
        long = "service-sleep",
        help = "Sleep time between service loops in seconds",
        default_value = "10"
    )]
    pub service_sleep: u64,

    #[structopt(
        long = "process-sleep",
        help = "Sleep time between process loops in seconds",
        default_value = "10"
    )]
    pub process_sleep: u64,

    #[structopt(long = "http", help = "Enable http server")]
    pub http: bool,

    #[structopt(
        long = "http-threads",
        help = "Number of threads to use for the server",
        default_value = "2"
    )]
    pub http_threads: u64,

    #[structopt(
        long = "http-port",
        help = "Port number of the server",
        default_value = "8080"
    )]
    pub http_port: u16,

    #[structopt(
        long = "http-addr",
        help = "Bind address of the server",
        default_value = "127.0.0.1"
    )]
    pub http_addr: String,

    #[structopt(long = "faucet", help = "Enabled faucet for the server")]
    pub faucet: bool,

    #[structopt(long = "debug", help = "Enabled debug endpoint for the server")]
    pub debug: bool,

    #[structopt(long = "frontend", help = "Enabled frontend serving for the server")]
    pub frontend: bool,
}

#[derive(StructOpt)]
#[structopt(about = "Import payment list")]
pub struct ImportOptions {
    #[structopt(long = "file", help = "File to import", default_value = "payments.csv")]
    pub file: String,
    #[structopt(long = "separator", help = "Separator", default_value = "|")]
    pub separator: char,

    //default is Mumbai for safety
    #[structopt(long = "chain-name", default_value = "mumbai")]
    pub chain_name: String,
}

#[derive(StructOpt)]
#[structopt(about = "Import payment list")]
pub struct DecryptKeyStoreOptions {
    #[structopt(
        short = "f",
        long = "file",
        help = "File to import",
        default_value = "payments.csv"
    )]
    pub file: String,
    #[structopt(short = "p", long = "password", help = "Password")]
    pub password: Option<String>,
}

#[derive(StructOpt)]
#[structopt(about = "Payment admin tool")]
pub enum PaymentCommands {
    Run {
        #[structopt(flatten)]
        run_options: RunOptions,
    },
    ImportPayments {
        #[structopt(flatten)]
        import_options: ImportOptions,
    },
    DecryptKeyStore {
        #[structopt(flatten)]
        decrypt_options: DecryptKeyStoreOptions,
    },
}

#[derive(StructOpt)]
#[structopt(about = "Payment admin tool")]
pub struct PaymentOptions {
    #[structopt(subcommand)]
    pub commands: PaymentCommands,
}

#[derive(Debug, StructOpt)]
pub struct CliOptions {}
