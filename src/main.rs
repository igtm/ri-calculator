use aws_sdk_ec2 as ec2;
use aws_smithy_types;
use cli_table::{format::Justify, print_stdout, Cell, Style, Table, CellStruct, WithTitle};
use ec2::{Client, model::{ReservedInstanceState, InstanceStateName}, output::{DescribeReservedInstancesOutput, DescribeInstancesOutput}};
use std::{io};

#[derive(Table)]
struct EC2Instance {
    #[table(title = "product_description")]
    product_description: String,
    #[table(title = "instance_type")]
    instance_type: String,
    #[table(title = "instance_count", justify = "Justify::Right")]
    instance_count: i32,
    #[table(title = "running_count", justify = "Justify::Right")]
    running_count: i32,
    #[table(title = "reserved_active_count", justify = "Justify::Right")]
    reserved_active_count: i32,
    #[table(title = "reserved_expired_count", justify = "Justify::Right")]
    reserved_expired_count: i32,
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let config = aws_config::load_from_env().await;
    let client = ec2::Client::new(&config);

    let ri = get_ec2_ri(&client).await;
    let instances = get_ec2_instances(&client).await;

    // calc
    let mut out: Vec<EC2Instance> = vec![];
    for ri in instances.reservations().unwrap().into_iter() {
      for instance in ri.instances().unwrap() {
        let mut found = false;
        for o in &mut out {
          if o.product_description == instance.platform_details().map(|x| x.to_owned()).unwrap_or("".to_owned()) &&
          o.instance_type == instance.instance_type().map(|x| x.as_str().to_owned()).unwrap_or("".to_owned()) {
            found =true ;
            o.instance_count += 1;
            if let Some(state) = instance.state().and_then(|x| x.name.as_ref()) {
              match state {
                InstanceStateName::Running => {o.running_count += 1},
                _ => {},
              }
            }
            break;
          }
        }
        if !found {
          let mut running_count = 0;
          if let Some(state) = instance.state().and_then(|x| x.name.as_ref()) {
            match state {
              InstanceStateName::Running => {running_count = 1},
              _ => {},
            }
          }
          out.push(EC2Instance{
            product_description: instance.platform_details().map(|x| x.to_owned()).unwrap_or("".to_owned()),
            instance_type: instance.instance_type().map(|x| x.as_str().to_owned()).unwrap_or("".to_owned()),
            instance_count: 1,
            running_count: running_count,
            reserved_active_count: 0,
            reserved_expired_count: 0,
          })
        }
      }
    }

    for i in ri.reserved_instances().unwrap().into_iter() {
      if let Some(s) = i.state() {
        let mut found = false;
        let mut reserved_active_count = 0;
        let mut reserved_expired_count = 0;
        match s {
          ReservedInstanceState::Active => {reserved_active_count = i.instance_count().unwrap_or(0)},
          ReservedInstanceState::Retired => {reserved_expired_count = i.instance_count().unwrap_or(0)},
          _ => continue
        }
        for o in &mut out {
          if o.product_description == i.product_description().map(|x| x.as_str()).unwrap_or("") &&
            o.instance_type == i.instance_type().map(|x| x.as_str()).unwrap_or("") {
            found =true;
            o.reserved_active_count += reserved_active_count;
            o.reserved_expired_count += reserved_expired_count;
          }
        }
        if !found {
          out.push(EC2Instance{
            product_description: i.product_description().map(|x| x.as_str().to_owned()).unwrap_or("".to_owned()),
            instance_type: i.instance_type().map(|x| x.as_str().to_owned()).unwrap_or("".to_owned()),
            instance_count: 0,
            running_count: 0,
            reserved_active_count: reserved_active_count,
            reserved_expired_count: reserved_expired_count,
          })
        }
      }
    }

    assert!(print_stdout(out.with_title()).is_ok());

    Ok(())
}

async fn get_ec2_ri(client: &Client) -> DescribeReservedInstancesOutput {

    // RI
    let ret = client.describe_reserved_instances().send().await.unwrap();

    let mut vec = vec![];
    for ri in ret.reserved_instances().unwrap().into_iter() {
      vec.push(vec![
        get_cell(ri.availability_zone()),
        get_cell(ri.duration().and_then(|x| Some(x / (60 * 60 * 24 * 365)))),
        get_cell(ri.end().and_then(|x| x.fmt(aws_smithy_types::date_time::Format::DateTime).ok()).or_else(|| None)),
        // get_cell(ri.fixed_price()),
        get_cell(ri.instance_count()),
        get_cell(ri.instance_type().map(|x| x.as_str())),
        get_cell(ri.product_description().map(|x| x.as_str())),
        // get_cell(ri.reserved_instances_id()),
        get_cell(ri.start().and_then(|x| x.fmt(aws_smithy_types::date_time::Format::DateTime).ok()).or_else(|| None)),
        get_cell(ri.state()),
        // get_cell(ri.usage_price()),
        // get_cell(ri.currency_code()),
        get_cell(ri.instance_tenancy()),
        get_cell(ri.offering_class()),
        get_cell(ri.offering_type()),
        // get_cell(ri.recurring_charges()),
        get_cell(ri.scope()),
      ])
    }

    // cli-table
    let table = vec.table()
    .title(vec![
        "availability_zone".cell().bold(true),
        "duration".cell().bold(true),
        "end".cell().bold(true),
        // "fixed_price".cell().bold(true),
        "instance_count".cell().bold(true),
        "instance_type".cell().bold(true),
        "product_description".cell().bold(true),
        // "reserved_instances_id".cell().bold(true),
        "start".cell().bold(true),
        "state".cell().bold(true),
        // "usage_price".cell().bold(true),
        // "currency_code".cell().bold(true),
        "instance_tenancy".cell().bold(true),
        "offering_class".cell().bold(true),
        "offering_type".cell().bold(true),
        // "recurring_charges".cell().bold(true),
        "scope".cell().bold(true),
    ])
    .bold(true);

    assert!(print_stdout(table).is_ok());

    return ret;
}

async fn get_ec2_instances(client: &Client) -> DescribeInstancesOutput {

    // RI
    let ret = client.describe_instances().send().await.unwrap();

    let mut vec = vec![];
    for ri in ret.reservations().unwrap().into_iter() {
      for instance in ri.instances().unwrap() {
        vec.push(vec![
          get_cell(instance.instance_type().map(|x| x.as_str())),
          get_cell(instance.state().and_then(|x| x.name.as_ref())),
          // get_cell(instance.platform()),
          get_cell(instance.platform_details()),
          get_cell(instance.placement().and_then(|x| x.availability_zone())),
          get_cell(instance.placement().and_then(|x| x.tenancy())),
        ])
      }
    }

    // cli-table
    let table = vec
    .table()
    .title(vec![
        "instance_type".cell().bold(true),
        "state".cell().bold(true),
        // "platform".cell().bold(true),
        "platform_details".cell().bold(true),
        "availability_zone".cell().bold(true),
        "tenancy".cell().bold(true),
    ])
    .bold(true);

    assert!(print_stdout(table).is_ok());

    return ret;
}

fn get_cell<T: std::fmt::Debug>(s: std::option::Option<T>) -> CellStruct {
  let str = match s {
    Some(x) => format!("{:?}", x),
    None => "".to_owned(),
  };
  return str.cell()
}
