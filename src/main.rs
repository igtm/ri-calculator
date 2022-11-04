mod terminal;
mod model_ec2;

use aws_sdk_ec2 as ec2;
use aws_smithy_types;
// use cli_table::{format::Justify, print_stdout, Cell, Style, Table, CellStruct, WithTitle};
use ec2::{Client, model::{ReservedInstanceState, InstanceStateName, Instance, ReservedInstances}, output::{DescribeReservedInstancesOutput, DescribeInstancesOutput}};
use crossterm::{
  event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
  execute,
  terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
  backend::{Backend, CrosstermBackend},
  layout::{Constraint, Layout},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Cell, Row, Table, TableState},
  Frame, Terminal, text::Spans,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>  {
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

    // assert!(print_stdout(out.with_title()).is_ok());

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = terminal::App::new(
      terminal::TopWindowOptions::new(
        vec![
          Spans::from(format!("{}", "aaa")),
        ]
      ),
      terminal::TableOptions::new(
        out,
      )
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

    // let mut vec = vec![];
    // for ri in ret.reserved_instances().unwrap().into_iter() {
    //   vec.push(vec![
    //     get_cell(ri.availability_zone()),
    //     get_cell(ri.duration().and_then(|x| Some(x / (60 * 60 * 24 * 365)))),
    //     get_cell(ri.end().and_then(|x| x.fmt(aws_smithy_types::date_time::Format::DateTime).ok()).or_else(|| None)),
    //     // get_cell(ri.fixed_price()),
    //     get_cell(ri.instance_count()),
    //     get_cell(ri.instance_type().map(|x| x.as_str())),
    //     get_cell(ri.product_description().map(|x| x.as_str())),
    //     // get_cell(ri.reserved_instances_id()),
    //     get_cell(ri.start().and_then(|x| x.fmt(aws_smithy_types::date_time::Format::DateTime).ok()).or_else(|| None)),
    //     get_cell(ri.state()),
    //     // get_cell(ri.usage_price()),
    //     // get_cell(ri.currency_code()),
    //     get_cell(ri.instance_tenancy()),
    //     get_cell(ri.offering_class()),
    //     get_cell(ri.offering_type()),
    //     // get_cell(ri.recurring_charges()),
    //     get_cell(ri.scope()),
    //   ])
    // }

    // // cli-table
    // let table = vec.table()
    // .title(vec![
    //     "availability_zone".cell().bold(true),
    //     "duration".cell().bold(true),
    //     "end".cell().bold(true),
    //     // "fixed_price".cell().bold(true),
    //     "instance_count".cell().bold(true),
    //     "instance_type".cell().bold(true),
    //     "product_description".cell().bold(true),
    //     // "reserved_instances_id".cell().bold(true),
    //     "start".cell().bold(true),
    //     "state".cell().bold(true),
    //     // "usage_price".cell().bold(true),
    //     // "currency_code".cell().bold(true),
    //     "instance_tenancy".cell().bold(true),
    //     "offering_class".cell().bold(true),
    //     "offering_type".cell().bold(true),
    //     // "recurring_charges".cell().bold(true),
    //     "scope".cell().bold(true),
    // ])
    // .bold(true);

    // assert!(print_stdout(table).is_ok());

    return ret;
}

async fn get_ec2_instances(client: &Client) -> DescribeInstancesOutput {

    // RI
    let ret = client.describe_instances().send().await.unwrap();

    // let mut vec = vec![];
    // for ri in ret.reservations().unwrap().into_iter() {
    //   for instance in ri.instances().unwrap() {
    //     vec.push(vec![
    //       get_cell(instance.instance_type().map(|x| x.as_str())),
    //       get_cell(instance.state().and_then(|x| x.name.as_ref())),
    //       // get_cell(instance.platform()),
    //       get_cell(instance.platform_details()),
    //       get_cell(instance.placement().and_then(|x| x.availability_zone())),
    //       get_cell(instance.placement().and_then(|x| x.tenancy())),
    //     ])
    //   }
    // }

    // // cli-table
    // let table = vec
    // .table()
    // .title(vec![
    //     "instance_type".cell().bold(true),
    //     "state".cell().bold(true),
    //     // "platform".cell().bold(true),
    //     "platform_details".cell().bold(true),
    //     "availability_zone".cell().bold(true),
    //     "tenancy".cell().bold(true),
    // ])
    // .bold(true);

    // assert!(print_stdout(table).is_ok());

    return ret;
}

// fn get_cell<T: std::fmt::Debug>(s: std::option::Option<T>) -> CellStruct {
//   let str = match s {
//     Some(x) => format!("{:?}", x),
//     None => "".to_owned(),
//   };
//   return str.cell()
// }
