mod model_ec2;
mod terminal;

use aws_sdk_ec2 as ec2;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ec2::{
    output::{DescribeInstancesOutput, DescribeReservedInstancesOutput},
    Client,
};
use std::{error::Error, io};
use tui::{backend::CrosstermBackend, text::Spans, Terminal};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = aws_config::load_from_env().await;
    let client = ec2::Client::new(&config);

    let ri = get_ec2_ri(&client).await;
    let instances = get_ec2_instances(&client).await;

    // calc
    let mut out: model_ec2::EC2Instances = model_ec2::EC2Instances::new();
    for ri in instances.reservations().unwrap().into_iter() {
        for instance in ri.instances().unwrap() {
            out.push_from_instance(&instance);
        }
    }

    for i in ri.reserved_instances().unwrap().into_iter() {
        out.push_from_reserved_instance(i);
    }

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = terminal::App::new(
        terminal::TopWindowOptions::new(vec![Spans::from(format!("{}", "aaa"))]),
        terminal::TableOptions::new(out),
    );
    let res = terminal::run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn get_ec2_ri(client: &Client) -> DescribeReservedInstancesOutput {
    // RI
    let ret = client.describe_reserved_instances().send().await.unwrap();

    return ret;
}

async fn get_ec2_instances(client: &Client) -> DescribeInstancesOutput {
    // RI
    let ret = client.describe_instances().send().await.unwrap();

    return ret;
}
