use aws_sdk_ec2::model::{Instance, InstanceStateName, ReservedInstances, ReservedInstanceState};
use tui::layout::Constraint;

pub trait EC2Instance {
  fn to_vec(&self) -> Vec<String>;
}

#[derive(Clone)]
pub struct EC2InstanceInstance {
  product_description: String,
  instance_type: String,
  running_count: i32,
  reserved_active_count: i32,
  reserved_expired_count: i32,
}

impl EC2Instance for EC2InstanceInstance {
  fn to_vec(&self) -> Vec<String> {
    return vec![
      self.product_description.to_owned(),
      self.instance_type.to_owned(),
      format!("{}", self.running_count),
      format!("{}", self.reserved_active_count),
      format!("{}", self.reserved_expired_count),
    ];
  }
}
impl EC2InstanceInstance {
  pub fn instance_family(&self) -> String {
    let splitted: Vec<&str> = self.instance_type.split(".").collect();
    return splitted[0].to_owned();
  }

  pub fn instance_size(&self) -> String {
    let splitted: Vec<&str> = self.instance_type.split(".").collect();
    return splitted[1].to_owned();
  }

  pub fn running_count_normalization_factor(&self) -> f32 {
    let nf = get_normalization_factor(self.instance_size().as_str());
    return self.running_count as f32 * nf;
  }

  pub fn reserved_active_normalization_factor(&self) -> f32 {
    let nf = get_normalization_factor(self.instance_size().as_str());
    return self.reserved_active_count as f32 * nf;
  }

}

#[derive(Clone)]
pub struct EC2InstanceNormalizationFactor {
  product_description: String,
  instance_family: String,
  running_count_normalization_factor: f32,
  reserved_active_normalization_factor: f32,
}
impl EC2Instance for EC2InstanceNormalizationFactor {
  fn to_vec(&self) -> Vec<String> {
    return vec![
      self.product_description.to_owned(),
      self.instance_family.to_owned(),
      format!("{}", self.running_count_normalization_factor),
      format!("{}", self.reserved_active_normalization_factor),
      format!("{}", self.normalization_factor_diff()),
      format!("{:.1}%", self.normalization_factor_coverage() * 100.0),
    ];
  }
}
impl EC2InstanceNormalizationFactor {
  pub fn normalization_factor_coverage(&self) -> f32 {
    if self.running_count_normalization_factor == 0.0 {
      return 0.0;
    }
    return self.reserved_active_normalization_factor / self.running_count_normalization_factor;
  }
  pub fn normalization_factor_diff(&self) -> f32 {
    return self.running_count_normalization_factor - self.reserved_active_normalization_factor;
  }
}

pub fn get_normalization_factor(s: &str) -> f32 {
  return match s {
    "nano" => 0.25,
    "micro" => 0.5,
    "small" => 1.0,
    "medium" => 2.0,
    "large" => 4.0,
    "xlarge" => 8.0,
    "2xlarge" => 16.0,
    "3xlarge" => 24.0,
    "4xlarge" => 32.0,
    "6xlarge" => 48.0,
    "8xlarge" => 64.0,
    "9xlarge" => 72.0,
    "10xlarge" => 80.0,
    "12xlarge" => 96.0,
    "16xlarge" => 128.0,
    "18xlarge" => 144.0,
    "24xlarge" => 192.0,
    "32xlarge" => 256.0,
    "56xlarge" => 448.0,
    "112xlarge" => 896.0,
    _ => panic!("this normalization factor table does't have given instance family type"),
  }
}

pub struct EC2Instances {
  items: Vec<EC2InstanceInstance>,
  view_mode: ViewMode,
}

pub enum ViewMode {
  Instance,
  NormalizationFactor
}

impl EC2Instances {
  pub fn new() -> EC2Instances {
    EC2Instances {
      items: vec![],
      view_mode: ViewMode::Instance,
    }
  }
  pub fn set_view_mode(&mut self, view_mode: ViewMode) {
    self.view_mode = view_mode;
  }
  pub fn title<'a>(&self) -> &'a str {
    return match self.view_mode {
      ViewMode::Instance => "EC2 Instance",
      ViewMode::NormalizationFactor => "EC2 RI NormalizationFactor",
    }
  }
  pub fn widths<'a>(&self) -> &'a [Constraint]{
    return match self.view_mode {
      ViewMode::Instance => &[Constraint::Percentage(15); 5],
      ViewMode::NormalizationFactor => &[Constraint::Percentage(15); 6],
    }
  }
  pub fn header<'a>(&self) -> Vec<&'a str> {
    match self.view_mode {
      ViewMode::Instance => vec![
        "product_description",
        "instance_type",
        "running_count",
        "reserved_active_count",
        "reserved_expired_count(all)",
      ],
      ViewMode::NormalizationFactor => vec![
        "product_description",
        "instance_family",
        "running_count_normalization_factor",
        "reserved_active_normalization_factor",
        "normalization_factor_diff",
        "normalization_factor_coverage",
      ]
    }
  }
  pub fn push_from_instance(&mut self, instance: &Instance) {
    let mut found = false;
    for o in &mut self.items {
      if o.product_description == instance.platform_details().map(|x| x.to_owned()).unwrap_or("".to_owned()) &&
      o.instance_type == instance.instance_type().map(|x| x.as_str().to_owned()).unwrap_or("".to_owned()) {
        found =true ;
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
      self.items.push(EC2InstanceInstance{
        product_description: instance.platform_details().map(|x| x.to_owned()).unwrap_or("".to_owned()),
        instance_type: instance.instance_type().map(|x| x.as_str().to_owned()).unwrap_or("".to_owned()),
        running_count: running_count,
        reserved_active_count: 0,
        reserved_expired_count: 0,
      })
    }
  }
  pub fn push_from_reserved_instance(&mut self, ri: &ReservedInstances) {
    if let Some(s) = ri.state() {
      let mut found = false;
      let mut reserved_active_count = 0;
      let mut reserved_expired_count = 0;
      match s {
        ReservedInstanceState::Active => {reserved_active_count = ri.instance_count().unwrap_or(0)},
        ReservedInstanceState::Retired => {reserved_expired_count = ri.instance_count().unwrap_or(0)},
        _ => return
      }
      for o in &mut self.items {
        if o.product_description == ri.product_description().map(|x| x.as_str()).unwrap_or("") &&
          o.instance_type == ri.instance_type().map(|x| x.as_str()).unwrap_or("") {
          found =true;
          o.reserved_active_count += reserved_active_count;
          o.reserved_expired_count += reserved_expired_count;
        }
      }
      if !found {
        self.items.push(EC2InstanceInstance{
          product_description: ri.product_description().map(|x| x.as_str().to_owned()).unwrap_or("".to_owned()),
          instance_type: ri.instance_type().map(|x| x.as_str().to_owned()).unwrap_or("".to_owned()),
          running_count: 0,
          reserved_active_count: reserved_active_count,
          reserved_expired_count: reserved_expired_count,
        })
      }
    }
  }
  pub fn to_vec(&self) -> Vec<Vec<String>> {
    return match self.view_mode {
      ViewMode::Instance => self.items.iter().map(|x| x.to_vec()).collect(),
      ViewMode::NormalizationFactor => self.agg_by_instance_family().iter().map(|x| x.to_vec()).collect(),
    }
  }
  pub fn agg_by_instance_family(&self) -> Vec<EC2InstanceNormalizationFactor> {
    let mut ret: Vec<EC2InstanceNormalizationFactor> = vec![];
    for item in &self.items {
      let mut found = false;
      for o in &mut ret {
        if item.instance_family() == o.instance_family && item.product_description == o.product_description {
          found = true;
          o.running_count_normalization_factor += item.running_count_normalization_factor();
          o.reserved_active_normalization_factor += item.reserved_active_normalization_factor();
          break;
        }
      }

      if !found {
        if item.running_count_normalization_factor() == 0.0 && item.reserved_active_normalization_factor() == 0.0 {
          continue;
        }
        ret.push(EC2InstanceNormalizationFactor{
          product_description: item.product_description.to_owned(),
          instance_family: item.instance_family().to_owned(),
          running_count_normalization_factor: item.running_count_normalization_factor(),
          reserved_active_normalization_factor: item.reserved_active_normalization_factor(),
        });
      }
    }

    return ret;
  }
}
