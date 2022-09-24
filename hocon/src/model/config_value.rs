use std::collections::HashMap;
use std::env;
use std::rc::Rc;

use crate::model::config_array_value::ConfigArrayValue;
use crate::model::config_duration_value::ConfigDurationValue;
use crate::model::config_include_value::ConfigIncludeValue;
use crate::model::config_number_value::ConfigNumberValue;
use crate::model::config_object_value::ConfigObjectValue;
use crate::model::config_value_link::ConfigValueLink;
use crate::model::{ConfigFactory, ConfigMergeable, Monoid};

#[derive(Clone, Debug, PartialEq)]
pub enum ConfigValue {
  Null,
  Bool(bool),
  String(String),
  Number(ConfigNumberValue),
  Duration(ConfigDurationValue),
  Array(ConfigArrayValue),
  Object(ConfigObjectValue),
  Reference {
    prev: Option<Rc<ConfigValue>>,
    path: String,
    missing: bool,
  },
  Include(ConfigIncludeValue),
  Link(Rc<ConfigValueLink>),
}

impl Monoid for ConfigValue {
  fn combine(&mut self, other: &Self) {
    match other {
      cv @ ConfigValue::Link(..) => {
        for child_cv in cv.to_vec() {
          self.push(child_cv);
        }
      }
      _ => {}
    }
  }
}

impl ConfigMergeable for ConfigValue {
  fn merge_with(&mut self, other: Self) {
    match (self, other) {
      (ConfigValue::Object(l), ConfigValue::Object(r)) => {
        l.merge_with(r);
      }
      (ConfigValue::Array(l), ConfigValue::Array(r)) => {
        l.merge_with(r);
      }
      (ConfigValue::Link(..), ConfigValue::Link(..)) => {}
      (re @ ConfigValue::Link(..), r) => {
        let mut n = re.get_value_link().unwrap().value.clone();
        n.merge_with(r);
        re.push(n);
      }
      (..) => {}
    }
  }
}

impl ConfigValue {
  pub fn of_reference(prev: Option<Rc<ConfigValue>>, path: String, missing: bool) -> Self {
    ConfigValue::Reference { prev, path, missing }
  }

  pub fn to_vec(&self) -> Vec<ConfigValue> {
    match self {
      ConfigValue::Link(cv) => {
        let mut cur = cv.clone();
        let mut result = vec![cur.value.clone()];
        while let ConfigValue::Link(prev_cur) = &*cur.prev {
          cur = prev_cur.clone();
          result.push(cur.value.clone());
        }
        result.push((*cur.prev).clone());
        result.reverse();
        result
      }
      cv => vec![cv.clone()],
    }
  }

  pub fn push(&mut self, cv: ConfigValue) {
    let to = Rc::new(self.clone());
    let cv = match cv {
      ConfigValue::Reference {
        path: ref_name,
        missing,
        ..
      } => ConfigValue::of_reference(Some(to.clone()), ref_name.clone(), missing),
      _ => cv,
    };
    let cvl = ConfigValueLink::new(to, cv);
    *self = ConfigValue::Link(Rc::new(cvl))
  }

  pub fn latest(&self) -> &Self {
    match self {
      ConfigValue::Link(cv) => &cv.value,
      cv => cv,
    }
  }

  pub fn prev_latest(&self) -> &Self {
    println!("self = {:?}", self);
    match self {
      ConfigValue::Link(cv) => match &*(cv.prev) {
        ConfigValue::Link(prev_cv) => {
          let ret = &(prev_cv.value);
          ret
        }
        cv => cv,
      },
      cv => cv,
    }
  }

  fn ref_prev(&self) -> &Option<Rc<ConfigValue>> {
    match self {
      ConfigValue::Reference { prev, .. } => prev,
      _ => &None,
    }
  }

  fn ref_name(&self) -> Option<&String> {
    match self {
      ConfigValue::Reference { path: ref_name, .. } => Some(ref_name),
      _ => None,
    }
  }

  fn ref_missing(&self) -> Option<bool> {
    match self {
      ConfigValue::Reference { missing, .. } => Some(*missing),
      _ => None,
    }
  }

  fn get_include_value(&self) -> Option<&ConfigIncludeValue> {
    match self {
      ConfigValue::Include(civ) => Some(civ),
      _ => None,
    }
  }

  fn get_object_value(&self) -> Option<&ConfigObjectValue> {
    match self {
      ConfigValue::Object(cov) => Some(cov),
      _ => None,
    }
  }

  fn get_array_value(&self) -> Option<&ConfigArrayValue> {
    match self {
      ConfigValue::Array(cav) => Some(cav),
      _ => None,
    }
  }

  fn get_value_link(&self) -> Option<&ConfigValueLink> {
    match self {
      ConfigValue::Link(cvl) => Some(&*cvl),
      _ => None,
    }
  }

  pub fn resolve(&mut self, source: Option<&Self>) {
    match (self, source) {
      (cvi @ ConfigValue::Include(..), ..) => {
        let mut config_factory = ConfigFactory::new();
        let c = config_factory
          .load_from_file(&cvi.get_include_value().unwrap().file_name)
          .unwrap();
        *cvi = c.to_config_value().clone();
      }
      (cvl @ ConfigValue::Link(..), Some(..)) => {
        let cvs = cvl.to_vec();
        let mut head = cvs[0].clone();
        head.resolve(source);

        for e in &cvs[1..] {
          let mut ee = e.clone();
          ee.resolve(source);
          head.push(ee.clone());
        }

        *cvl = head;
      }
      (cva @ ConfigValue::Array(..), Some(..)) => {
        let av = cva.get_array_value().unwrap();
        let mut m = vec![];
        for mut cv in av.0.clone().into_iter() {
          cv.resolve(source);
          m.push(cv);
        }
        *cva = ConfigValue::Array(ConfigArrayValue::new(m));
      }
      (cvo @ ConfigValue::Object(..), Some(..)) => {
        let ov = cvo.get_object_value().unwrap();
        let mut m = HashMap::new();
        for (k, mut cv) in ov.0.clone().into_iter() {
          cv.resolve(source);
          m.insert(k, cv);
        }
        *cvo = ConfigValue::Object(ConfigObjectValue::new(m));
      }
      (cvr @ ConfigValue::Reference { .. }, Some(src)) => {
        let ref_value = src
          .get_value(cvr.ref_name().unwrap())
          .cloned()
          .or_else(|| env::var(cvr.ref_name().unwrap()).ok().map(|s| ConfigValue::String(s)));
        if cvr.ref_missing().unwrap() {
          if ref_value.is_some() {
            *cvr = ref_value.unwrap();
          } else {
            *cvr = cvr.ref_prev().clone().unwrap().prev_latest().clone();
          }
        } else {
          if ref_value.is_none() {
            panic!("Cannot resolve the reference: {}", cvr.ref_name().unwrap())
          }
          *cvr = ref_value.unwrap();
        }
      }
      _ => {}
    }
  }

  pub fn get_value(&self, path: &str) -> Option<&ConfigValue> {
    let keys = path.split(".").collect::<Vec<_>>();
    let key = keys[0];
    let child_count = keys.len() - 1;
    match self {
      ConfigValue::Object(map) => match map.0.get(key) {
        Some(cv) if child_count > 0 => {
          let next_path = &path[(key.len() + 1) as usize..];
          cv.latest().get_value(next_path)
        }
        Some(cv) => Some(cv.latest()),
        None => None,
      },
      _ => None,
    }
  }

  pub fn contains(&self, key: &str) -> bool {
    match self {
      ConfigValue::Object(map) => map.0.contains_key(key),
      _ => false,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::model::config_value::ConfigValue;
  use crate::model::Monoid;

  #[test]
  fn test_push() {
    let mut config_value = ConfigValue::Bool(true);
    config_value.push(ConfigValue::String("ABC".to_string()));
    config_value.push(ConfigValue::Null);

    println!("{:?}", config_value);
    println!("{:?}", config_value.to_vec());

    assert_eq!(config_value.latest().clone(), ConfigValue::Null);
    assert_eq!(
      config_value.prev_latest().clone(),
      ConfigValue::String("ABC".to_string())
    );
  }

  #[test]
  fn test_combine() {
    let mut first = ConfigValue::Bool(true);
    first.push(ConfigValue::String("ABC".to_string()));
    first.push(ConfigValue::Null);
    let mut second = ConfigValue::Bool(false);
    second.push(ConfigValue::String("XYZ".to_string()));
    second.push(ConfigValue::Reference {
      prev: None,
      path: "ABC".to_string(),
      missing: false,
    });

    let mut t = first.clone();

    t.combine(&second);

    // [Bool(true), "ABC", Null]
    println!("{:?}", t);
    println!("{:?}", t.to_vec());
  }
}
