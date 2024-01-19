// read configuration from environment variables or from command line args
#[derive(clap::Parser, Debug)]
pub struct Config {
    #[clap(long, env)]
    pub dynamo_table_name: String,
    #[clap(long, env)]
    pub aws_region: String,
    #[clap(long, env)]
    pub aws_profile: Option<String>,
}